import SwiftUI
import OSLog

private enum ReconnectTarget {
    case preview(HostProfile)
    case password(profile: HostProfile, password: String, startupCommand: String)
    case privateKey(profile: HostProfile, privateKeyPEM: String, passphrase: String?, startupCommand: String)
}

private let rendererSurfaceLogger = Logger(subsystem: "xyz.zinglix.shellow", category: "renderer")

struct AppShell: View {
    @State private var selectedTab: AppTab = .terminal
    @State private var coreSession = ShellowCoreSession()
    @State private var session = TerminalSession.preview
    @State private var profiles = HostProfileStore.load()
    @State private var settings = ShellowSettingsStore.load()
    @State private var reconnectTarget: ReconnectTarget?

    var body: some View {
        TabView(selection: $selectedTab) {
            TerminalScreen(
                session: $session,
                settings: settings,
                onTerminalInput: { input in
                    updateSession(coreSession.sendTerminalInput(input))
                },
                onReconnect: reconnectTarget == nil ? nil : {
                    reconnect()
                },
                onDisconnect: {
                    updateSession(coreSession.disconnectLiveShell())
                },
                onResizeTerminal: { cols, rows in
                    updateSession(coreSession.resizeTerminal(cols: cols, rows: rows))
                },
                onAttachRendererSurface: { rawHandle, width, height in
                    let response = coreSession.attachCoreAnimationLayer(
                        rawHandle: rawHandle,
                        width: width,
                        height: height
                    )
                    #if DEBUG
                    print("Shellow renderer surface attach \(response)")
                    rendererSurfaceLogger.info("Shellow renderer surface attach \(response, privacy: .public)")
                    #endif
                },
                onSetRendererOverlay: { overlayJSON in
                    _ = coreSession.setRendererOverlayJSON(overlayJSON)
                },
                onRenderRendererSurface: { width, height, firstRow, rowCount in
                    let presented = coreSession.renderRendererSurfaceFrame(
                        width: width,
                        height: height,
                        firstRow: firstRow,
                        rowCount: rowCount
                    )
                    #if DEBUG
                    if presented {
                        let message = "Shellow renderer terminal surface frame \(width)x\(height) first=\(firstRow) rows=\(rowCount)"
                        print(message)
                        rendererSurfaceLogger.info("\(message, privacy: .public)")
                    }
                    #endif
                    return presented
                },
                onDetachRendererSurface: {
                    let response = coreSession.detachRendererSurface()
                    #if DEBUG
                    print("Shellow renderer surface detach \(response)")
                    rendererSurfaceLogger.info("Shellow renderer surface detach \(response, privacy: .public)")
                    #endif
                },
                onClearTerminal: {
                    updateSession(coreSession.clearTerminal())
                },
                onResetTerminal: {
                    updateSession(coreSession.resetTerminal())
                }
            )
                .tabItem { AppTab.terminal.label }
                .tag(AppTab.terminal)

            HostsScreen(
                profiles: $profiles,
                selectedTab: $selectedTab,
                connectPreview: { profile in
                    reconnectTarget = .preview(profile)
                    updateSession(coreSession.connectPreview(to: profile))
                    selectedTab = .terminal
                },
                connectPassword: { profile, password, command in
                    reconnectTarget = .password(profile: profile, password: password, startupCommand: command)
                    session = .connecting(to: profile)
                    selectedTab = .terminal
                    Task {
                        await connectPasswordShell(profile: profile, password: password, startupCommand: command)
                    }
                },
                connectPrivateKey: { profile, privateKeyPEM, passphrase, command in
                    reconnectTarget = .privateKey(
                        profile: profile,
                        privateKeyPEM: privateKeyPEM,
                        passphrase: passphrase,
                        startupCommand: command
                    )
                    session = .connecting(to: profile)
                    selectedTab = .terminal
                    Task {
                        await connectPrivateKeyShell(
                            profile: profile,
                            privateKeyPEM: privateKeyPEM,
                            passphrase: passphrase,
                            startupCommand: command
                        )
                    }
                }
            )
                .tabItem { AppTab.hosts.label }
                .tag(AppTab.hosts)

            SettingsScreen(settings: $settings)
                .tabItem { AppTab.settings.label }
                .tag(AppTab.settings)
        }
        .tint(ShellowTheme.accent)
        .preferredColorScheme(settings.colorScheme.preferredSwiftUIColorScheme)
        .task {
            updateSession(coreSession.snapshot())
            while !Task.isCancelled {
                try? await Task.sleep(nanoseconds: 300_000_000)
                let next = coreSession.pollLiveShell()
                if next != session {
                    updateSession(next)
                }
            }
        }
        .onChange(of: profiles) {
            HostProfileStore.save(profiles)
        }
        .onChange(of: settings) {
            ShellowSettingsStore.save(settings)
        }
    }

    private func reconnect() {
        guard let reconnectTarget else { return }

        switch reconnectTarget {
        case .preview(let profile):
            updateSession(coreSession.connectPreview(to: profile))
            selectedTab = .terminal
        case .password(let profile, let password, let startupCommand):
            session = .connecting(to: profile)
            selectedTab = .terminal
            Task {
                await connectPasswordShell(
                    profile: profile,
                    password: password,
                    startupCommand: startupCommand
                )
            }
        case .privateKey(let profile, let privateKeyPEM, let passphrase, let startupCommand):
            session = .connecting(to: profile)
            selectedTab = .terminal
            Task {
                await connectPrivateKeyShell(
                    profile: profile,
                    privateKeyPEM: privateKeyPEM,
                    passphrase: passphrase,
                    startupCommand: startupCommand
                )
            }
        }
    }

    private func updateSession(_ next: TerminalSession) {
        session = next
        captureObservedHostKeyIfNeeded(from: next)
    }

    private func captureObservedHostKeyIfNeeded(from session: TerminalSession) {
        guard
            let observed = session.observedHostKeySha256?.trimmingCharacters(in: .whitespacesAndNewlines),
            !observed.isEmpty,
            let reconnectTarget
        else {
            return
        }

        let profile = reconnectTarget.profile
        let existingPin = profile.trustedHostKeySHA256?.trimmingCharacters(in: .whitespacesAndNewlines)
        guard existingPin?.isEmpty ?? true else {
            return
        }

        var updatedProfile = profile
        updatedProfile.trustedHostKeySHA256 = observed
        updatedProfile.lastConnected = .now

        if let index = profiles.firstIndex(where: { $0.id == profile.id }) {
            profiles[index] = updatedProfile
        }
        self.reconnectTarget = reconnectTarget.replacingProfile(updatedProfile)
    }

    @MainActor
    private func connectPasswordShell(
        profile: HostProfile,
        password: String,
        startupCommand: String
    ) async {
        updateSession(await coreSession.startPasswordShell(
            to: profile,
            password: password
        ))

        let command = startupCommand.trimmingCharacters(in: .whitespacesAndNewlines)
        if session.state != .disconnected && !command.isEmpty {
            updateSession(coreSession.sendTerminalInput(command + "\r"))
        }
    }

    @MainActor
    private func connectPrivateKeyShell(
        profile: HostProfile,
        privateKeyPEM: String,
        passphrase: String?,
        startupCommand: String
    ) async {
        updateSession(await coreSession.startPrivateKeyShell(
            to: profile,
            privateKeyPEM: privateKeyPEM,
            passphrase: passphrase
        ))

        let command = startupCommand.trimmingCharacters(in: .whitespacesAndNewlines)
        if session.state != .disconnected && !command.isEmpty {
            updateSession(coreSession.sendTerminalInput(command + "\r"))
        }
    }
}

private extension ReconnectTarget {
    var profile: HostProfile {
        switch self {
        case .preview(let profile), .password(let profile, _, _), .privateKey(let profile, _, _, _):
            profile
        }
    }

    func replacingProfile(_ profile: HostProfile) -> ReconnectTarget {
        switch self {
        case .preview:
            .preview(profile)
        case .password(_, let password, let startupCommand):
            .password(profile: profile, password: password, startupCommand: startupCommand)
        case .privateKey(_, let privateKeyPEM, let passphrase, let startupCommand):
            .privateKey(
                profile: profile,
                privateKeyPEM: privateKeyPEM,
                passphrase: passphrase,
                startupCommand: startupCommand
            )
        }
    }
}

#Preview {
    AppShell()
}
