import SwiftUI

private enum ReconnectTarget {
    case preview(HostProfile)
    case password(profile: HostProfile, password: String, startupCommand: String)
    case privateKey(profile: HostProfile, privateKeyPEM: String, passphrase: String?, startupCommand: String)
}

private enum ShellowRoute: Hashable {
    case terminal
}

struct AppShell: View {
    @State private var path: [ShellowRoute] = []
    @State private var coreSession = ShellowCoreSession()
    @State private var session = TerminalSession.preview
    @State private var profiles = HostProfileStore.load()
    @State private var settings = ShellowSettingsStore.load()
    @State private var reconnectTarget: ReconnectTarget?
    @State private var isSettingsPresented = false
    @State private var terminalRenderTick = 0

    var body: some View {
        NavigationStack(path: $path) {
            HostsScreen(
                profiles: $profiles,
                onOpenSettings: {
                    isSettingsPresented = true
                },
                connectPreview: { profile in
                    reconnectTarget = .preview(profile)
                    updateSession(coreSession.connectPreview(to: profile))
                    showTerminal()
                },
                connectPassword: { profile, password, command in
                    reconnectTarget = .password(profile: profile, password: password, startupCommand: command)
                    session = .connecting(to: profile)
                    showTerminal()
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
                    showTerminal()
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
            .navigationDestination(for: ShellowRoute.self) { route in
                switch route {
                case .terminal:
                    terminalScreen
                }
            }
            .sheet(isPresented: $isSettingsPresented) {
                SettingsScreen(settings: $settings)
            }
        }
        .tint(ShellowTheme.accent)
        .preferredColorScheme(settings.colorScheme.preferredSwiftUIColorScheme)
        .task {
            updateSession(coreSession.snapshot())
            var lastLiveRevision = coreSession.liveShellEventRevision()
            var idleRenderTicks = 0
            while !Task.isCancelled {
                try? await Task.sleep(nanoseconds: 50_000_000)
                let revision = coreSession.liveShellEventRevision()
                if revision != lastLiveRevision {
                    lastLiveRevision = revision
                    idleRenderTicks = 0
                    let next = coreSession.pollLiveShell()
                    if next != session {
                        updateSession(next)
                    } else if next.state == .connected {
                        advanceTerminalRenderTick()
                    }
                } else if session.state == .connected {
                    idleRenderTicks += 1
                    if idleRenderTicks >= 20 {
                        idleRenderTicks = 0
                        advanceTerminalRenderTick()
                    }
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

    private var terminalScreen: some View {
        TerminalScreen(
            session: $session,
            settings: settings,
            renderTick: terminalRenderTick,
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
                _ = coreSession.attachCoreAnimationLayer(
                    rawHandle: rawHandle,
                    width: width,
                    height: height
                )
            },
            onSetRendererOverlay: { overlayJSON in
                _ = coreSession.setRendererOverlayJSON(overlayJSON)
            },
            onRenderRendererSurface: { width, height, firstRow, rowCount in
                coreSession.renderRendererSurfaceFrame(
                    width: width,
                    height: height,
                    firstRow: firstRow,
                    rowCount: rowCount
                )
            },
            onDetachRendererSurface: {
                _ = coreSession.detachRendererSurface()
            },
            onClearTerminal: {
                updateSession(coreSession.clearTerminal())
            },
            onResetTerminal: {
                updateSession(coreSession.resetTerminal())
            }
        )
        .navigationBarTitleDisplayMode(.inline)
        .toolbar(.hidden, for: .navigationBar)
    }

    private func showTerminal() {
        guard path.last != .terminal else { return }
        path.append(.terminal)
    }

    private func reconnect() {
        guard let reconnectTarget else { return }

        switch reconnectTarget {
        case .preview(let profile):
            updateSession(coreSession.connectPreview(to: profile))
            showTerminal()
        case .password(let profile, let password, let startupCommand):
            session = .connecting(to: profile)
            showTerminal()
            Task {
                await connectPasswordShell(
                    profile: profile,
                    password: password,
                    startupCommand: startupCommand
                )
            }
        case .privateKey(let profile, let privateKeyPEM, let passphrase, let startupCommand):
            session = .connecting(to: profile)
            showTerminal()
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
        advanceTerminalRenderTick()
        captureObservedHostKeyIfNeeded(from: next)
    }

    private func advanceTerminalRenderTick() {
        terminalRenderTick &+= 1
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
