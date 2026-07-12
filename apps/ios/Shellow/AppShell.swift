import Foundation
import SwiftUI
import UIKit

private enum ReconnectTarget {
    case preview(HostProfile)
    case password(profile: HostProfile, password: String, startupCommand: String)
    case privateKey(profile: HostProfile, privateKeyPEM: String, passphrase: String?, startupCommand: String)
}

private enum CodexReconnectTarget {
    case password(profile: HostProfile, password: String, cwd: String, threadID: String?)
    case privateKey(profile: HostProfile, privateKeyPEM: String, passphrase: String?, cwd: String, threadID: String?)
}

private enum ClaudeReconnectTarget {
    case password(profile: HostProfile, password: String, cwd: String, sessionID: String?)
    case privateKey(profile: HostProfile, privateKeyPEM: String, passphrase: String?, cwd: String, sessionID: String?)
}

private enum HostConnectMode {
    case terminal
    case codex
    case claude

    var passwordPromptTitle: String {
        switch self {
        case .terminal: "Terminal Password"
        case .codex: "Codex Password"
        case .claude: "Claude Code Password"
        }
    }
}

private struct PasswordPromptRequest: Identifiable {
    let id = UUID()
    var profile: HostProfile
    var mode: HostConnectMode
    var reason: String?
}

private struct ConnectionNotice: Identifiable {
    let id = UUID()
    var title: String
    var message: String
}

private struct PendingHostKeyTrust {
    var fingerprint: String
    var mode: HostConnectMode
}

private let hostKeyConfirmationPrefix = "ssh host key confirmation required: "
private let codexRemoteControlBootstrapCommand = """
PATH="$PATH:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.local/bin:$HOME/.cargo/bin:$HOME/.bun/bin:$HOME/.npm-global/bin:/home/linuxbrew/.linuxbrew/bin"
export PATH
codex app-server daemon bootstrap --remote-control && printf '\n__SHELLOW_CODEX_BOOTSTRAP_OK__\n'
"""

private struct StoredPrivateKeyAuth {
    var credential: SSHKeyCredential
    var privateKeyPEM: String
    var passphrase: String?
}

private enum HostProbeCredential {
    case password(String)
    case privateKey(privateKeyPEM: String, passphrase: String?)
}

private enum ShellowRoute: Hashable {
    case terminal
    case codex
    case claude
}

struct AppShell: View {
    @Environment(\.scenePhase) private var scenePhase
    @State private var path: [ShellowRoute] = []
    @State private var coreSession = ShellowCoreSession()
    @State private var session = TerminalSession.preview
    @State private var codexSession = CodexSnapshot.disconnected()
    @State private var claudeSession = CodexSnapshot.disconnected()
    @State private var profiles = HostProfileStore.load()
    @State private var sshKeys = SSHKeyCredentialStore.load()
    @State private var settings = ShellowSettingsStore.load()
    @State private var reconnectTarget: ReconnectTarget?
    @State private var codexReconnectTarget: CodexReconnectTarget?
    @State private var claudeReconnectTarget: ClaudeReconnectTarget?
    @State private var passwordPrompt: PasswordPromptRequest?
    @State private var connectionNotice: ConnectionNotice?
    @State private var codexBootstrapPromptEndpoint: String?
    @State private var codexBootstrapError: String?
    @State private var pendingHostKeyTrust: PendingHostKeyTrust?
    @State private var isSettingsPresented = false
    @State private var terminalRenderTick = 0
    @State private var reconnectTerminalAfterBackground = false
    @State private var reconnectCodexAfterBackground = false

    private let secretStore = SSHSecretStore.shared

    var body: some View {
        NavigationStack(path: $path) {
            HostsScreen(
                profiles: $profiles,
                sshKeys: $sshKeys,
                onOpenSettings: {
                    isSettingsPresented = true
                },
                connectTerminal: { profile in
                    Task {
                        await connectHost(profile, mode: .terminal)
                    }
                },
                connectCodex: { profile in
                    Task {
                        await connectHost(profile, mode: .codex)
                    }
                },
                connectClaude: { profile in
                    Task {
                        await connectHost(profile, mode: .claude)
                    }
                }
            )
            .navigationDestination(for: ShellowRoute.self) { route in
                switch route {
                case .terminal:
                    terminalScreen
                case .codex:
                    codexScreen
                case .claude:
                    claudeScreen
                }
            }
            .sheet(isPresented: $isSettingsPresented) {
                SettingsScreen(settings: $settings)
            }
            .sheet(item: $passwordPrompt) { request in
                PasswordPromptSheet(
                    profile: request.profile,
                    modeTitle: request.mode.passwordPromptTitle,
                    reason: request.reason,
                    connect: { password in
                        startPasswordConnection(
                            profile: request.profile,
                            password: password,
                            mode: request.mode
                        )
                        passwordPrompt = nil
                    }
                )
                .presentationDetents([.large])
            }
            .alert(item: $connectionNotice) { notice in
                Alert(
                    title: Text(notice.title),
                    message: Text(notice.message),
                    dismissButton: .default(Text("OK"))
                )
            }
        }
        .tint(ShellowTheme.accent)
        .preferredColorScheme(settings.colorScheme.preferredSwiftUIColorScheme)
        .alert("Enable remote Codex?", isPresented: Binding(
            get: { codexBootstrapPromptEndpoint != nil },
            set: { if !$0 { codexBootstrapPromptEndpoint = nil } }
        )) {
            Button("Cancel", role: .cancel) {
                codexBootstrapPromptEndpoint = nil
            }
            Button("Enable and reconnect") {
                codexBootstrapPromptEndpoint = nil
                Task { await bootstrapRemoteCodexAndReconnect() }
            }
        } message: {
            Text("Shellow needs to enable the persistent Codex remote-control daemon on \(codexBootstrapPromptEndpoint ?? "this host"). This runs `codex app-server daemon bootstrap --remote-control` once over SSH. Only continue on a host you trust.")
        }
        .alert("Could not enable remote Codex", isPresented: Binding(
            get: { codexBootstrapError != nil },
            set: { if !$0 { codexBootstrapError = nil } }
        )) {
            Button("OK", role: .cancel) { codexBootstrapError = nil }
        } message: {
            Text(codexBootstrapError ?? "The remote setup command failed.")
        }
        .alert("Trust SSH host key?", isPresented: Binding(
            get: { pendingHostKeyTrust != nil },
            set: { if !$0 { pendingHostKeyTrust = nil } }
        )) {
            Button("Cancel", role: .cancel) {
                cancelPendingHostKeyTrust()
            }
            Button("Trust and connect") {
                trustPendingHostKeyAndReconnect()
            }
        } message: {
            Text("Verify this SHA-256 fingerprint with the server administrator before continuing:\n\n\(pendingHostKeyTrust?.fingerprint ?? "")")
        }
        .alert("New remote port detected", isPresented: Binding(
            get: { session.detectedRemotePorts.first != nil },
            set: { presented in
                guard !presented, let port = session.detectedRemotePorts.first else { return }
                updateSession(coreSession.dismissDetectedRemotePort(port))
            }
        )) {
            Button("Got it") {}
        } message: {
            if let port = session.detectedRemotePorts.first {
                Text("The remote host started listening on port \(port). Shellow has not exposed or forwarded this port.")
            }
        }
        .task {
#if DEBUG
            await handleSimulatorLaunchRequestIfNeeded()
            let isSimulatorCodexUsagePreview = ProcessInfo.processInfo.arguments
                .contains("--shellow-simulator-show-codex-usage")
#else
            let isSimulatorCodexUsagePreview = false
#endif
            _ = coreSession.setTerminalTheme(settings.terminalTheme.rawValue)
            coreSession.setTransportOptions(
                keepAliveSeconds: settings.keepAliveSeconds,
                detectRemotePorts: settings.detectRemotePorts
            )
            updateSession(coreSession.snapshot())
            var lastLiveRevision = coreSession.liveShellEventRevision()
            var lastCodexRevision = coreSession.codexEventRevision()
            var lastClaudeRevision = coreSession.claudeEventRevision()
            var idleRenderTicks = 0
            var codexConnectingPollTicks = 0
            var claudeConnectingPollTicks = 0
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

                if !isSimulatorCodexUsagePreview {
                    let codexRevision = coreSession.codexEventRevision()
                    if codexRevision != lastCodexRevision {
                        lastCodexRevision = codexRevision
                        codexConnectingPollTicks = 0
                        updateCodexSession(coreSession.pollCodex())
                    } else if codexSession.status == .connecting {
                        codexConnectingPollTicks += 1
                        if codexConnectingPollTicks >= 5 {
                            codexConnectingPollTicks = 0
                            updateCodexSession(coreSession.pollCodex())
                        }
                    }
                }

                let claudeRevision = coreSession.claudeEventRevision()
                if claudeSession.status == .connecting {
                    claudeConnectingPollTicks += 1
                } else {
                    claudeConnectingPollTicks = 0
                }
                if claudeRevision != lastClaudeRevision || claudeConnectingPollTicks >= 5 {
                    lastClaudeRevision = claudeRevision
                    claudeConnectingPollTicks = 0
                    updateClaudeSession(coreSession.pollClaude())
                }
            }
        }
        .onChange(of: profiles) {
            HostProfileStore.save(profiles)
        }
        .onChange(of: sshKeys) {
            SSHKeyCredentialStore.save(sshKeys)
        }
        .onChange(of: settings) {
            ShellowSettingsStore.save(settings)
            coreSession.setTransportOptions(
                keepAliveSeconds: settings.keepAliveSeconds,
                detectRemotePorts: settings.detectRemotePorts
            )
        }
        .onChange(of: settings.terminalTheme) {
            _ = coreSession.setTerminalTheme(settings.terminalTheme.rawValue)
            advanceTerminalRenderTick()
        }
        .onChange(of: scenePhase) {
            handleScenePhaseChange(scenePhase)
        }
    }

    private var terminalScreen: some View {
        TerminalScreen(
            session: $session,
            settings: settings,
            renderTick: terminalRenderTick,
            profileName: reconnectTarget?.profile.name ?? session.title,
            persistentTerminal: reconnectTarget?.profile.persistentTerminal,
            loadPersistentSessions: terminalSessionLoader,
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

    private var terminalSessionLoader: (() async -> RemoteTerminalSessionCatalog)? {
        guard
            let profile = reconnectTarget?.profile,
            let configuration = profile.persistentTerminal
        else {
            return nil
        }
        return {
            await loadRemoteTerminalSessions(
                profile: profile,
                configuration: configuration
            )
        }
    }

    private var codexScreen: some View {
        CodexScreen(
            snapshot: codexSession,
            onSendMessage: { message in
                updateCodexSession(coreSession.sendCodexMessage(message))
            },
            onUpdateSettings: { model, reasoningEffort, serviceTier, approvalPolicy, sandbox in
                updateCodexSession(coreSession.updateCodexSettings(
                    model: model,
                    reasoningEffort: reasoningEffort,
                    serviceTier: serviceTier,
                    approvalPolicy: approvalPolicy,
                    sandbox: sandbox
                ))
            },
            onBrowseDirectory: { path in
                updateCodexSession(await coreSession.browseCodexDirectory(path: path))
            },
            onListThreads: { cwd, searchTerm, cursor, archived, append in
                updateCodexSession(await coreSession.listCodexThreadsPage(
                    cwd: cwd,
                    searchTerm: searchTerm,
                    cursor: cursor,
                    archived: archived,
                    append: append
                ))
            },
            onStartThread: { cwd in
                if let target = codexReconnectTarget {
                    codexReconnectTarget = target
                        .replacingCwd(cwd)
                        .replacingThreadID(nil)
                }
                updateCodexSession(await coreSession.startCodexThread(cwd: cwd))
            },
            onStartThreadAndSend: { cwd, message in
                if let target = codexReconnectTarget {
                    codexReconnectTarget = target
                        .replacingCwd(cwd)
                        .replacingThreadID(nil)
                }
                let started = await coreSession.startCodexThread(cwd: cwd)
                updateCodexSession(started)
                guard started.threadId != nil, started.operation.lastError == nil else {
                    return
                }
                updateCodexSession(coreSession.sendCodexMessage(message))
            },
            onResumeThread: { threadId in
                let started = appShellMonotonicNanos()
#if DEBUG
                print("[Shellow Codex] app resume start threadId=\(threadId)")
#endif
                let next = await coreSession.resumeCodexThread(threadId: threadId)
#if DEBUG
                print("[Shellow Codex] app resume received elapsed_ms=\(appShellElapsedMs(since: started)) threadId=\(next.threadId ?? "nil") messages=\(next.messages.count) opError=\(next.operation.lastError ?? "")")
#endif
                updateCodexSession(next)
                if let cwd = codexSession.cwd {
                    codexReconnectTarget = codexReconnectTarget?.replacingCwd(cwd)
                }
            },
            onReadThread: { threadId in
                updateCodexSession(await coreSession.readCodexThread(threadId: threadId))
            },
            onLoadMoreThreadTurns: { threadId, cursor in
                updateCodexSession(await coreSession.loadMoreCodexThreadTurns(threadId: threadId, cursor: cursor))
            },
            onRenameThread: { threadId, name in
                updateCodexSession(await coreSession.renameCodexThread(threadId: threadId, name: name))
            },
            onArchiveThread: { threadId in
                updateCodexSession(await coreSession.archiveCodexThread(threadId: threadId))
            },
            onUnarchiveThread: { threadId in
                updateCodexSession(await coreSession.unarchiveCodexThread(threadId: threadId))
            },
            onDeleteThread: { threadId in
                updateCodexSession(await coreSession.deleteCodexThread(threadId: threadId))
            },
            onForkThread: { threadId, cwd in
                updateCodexSession(await coreSession.forkCodexThread(threadId: threadId, cwd: cwd))
                if let cwd = codexSession.cwd {
                    codexReconnectTarget = codexReconnectTarget?.replacingCwd(cwd)
                }
            },
            onInterruptTurn: {
                updateCodexSession(coreSession.interruptCodexTurn())
            },
            onApprovalDecision: { requestId, decision in
                updateCodexSession(coreSession.answerCodexApproval(requestId: requestId, decision: decision))
            },
            onDisconnect: {
                updateCodexSession(coreSession.disconnectCodex())
            },
            onReconnect: codexReconnectTarget == nil ? nil : {
                reconnectCodex()
            }
        )
        .navigationBarTitleDisplayMode(.inline)
        .toolbar(.hidden, for: .navigationBar)
    }

    private var claudeScreen: some View {
        CodexScreen(
            snapshot: claudeSession,
            onSendMessage: { message in
                updateClaudeSession(coreSession.sendClaudeMessage(message))
            },
            onUpdateSettings: { model, _, _, approvalPolicy, _ in
                let mode = approvalPolicy.isEmpty ? "default" : approvalPolicy
                updateClaudeSession(coreSession.updateClaudeSettings(model: model, permissionMode: mode))
            },
            onBrowseDirectory: { _ in updateClaudeSession(claudeSession) },
            onListThreads: { _, _, _, _, _ in updateClaudeSession(claudeSession) },
            onStartThread: { cwd in
                await startNewClaudeSession(cwd: cwd, initialMessage: nil)
            },
            onStartThreadAndSend: { cwd, message in
                await startNewClaudeSession(cwd: cwd, initialMessage: message)
            },
            onResumeThread: { _ in updateClaudeSession(claudeSession) },
            onReadThread: { _ in updateClaudeSession(claudeSession) },
            onLoadMoreThreadTurns: { _, _ in updateClaudeSession(claudeSession) },
            onRenameThread: { _, _ in updateClaudeSession(claudeSession) },
            onArchiveThread: { _ in updateClaudeSession(claudeSession) },
            onUnarchiveThread: { _ in updateClaudeSession(claudeSession) },
            onDeleteThread: { _ in updateClaudeSession(claudeSession) },
            onForkThread: { _, _ in updateClaudeSession(claudeSession) },
            onInterruptTurn: {
                updateClaudeSession(coreSession.interruptClaudeTurn())
            },
            onApprovalDecision: { requestId, decision in
                updateClaudeSession(coreSession.answerClaudeApproval(requestId: requestId, decision: decision))
            },
            onDisconnect: {
                updateClaudeSession(coreSession.disconnectClaude())
            },
            onReconnect: claudeReconnectTarget == nil ? nil : {
                reconnectClaude()
            }
        )
        .navigationBarTitleDisplayMode(.inline)
        .toolbar(.hidden, for: .navigationBar)
    }

    @MainActor
    private func connectHost(_ profile: HostProfile, mode: HostConnectMode) async {
        let savedPassword = secretStore.loadSecret(for: profile, kind: .password)

        if profile.authentication == .password {
            if let savedPassword {
                startPasswordConnection(profile: profile, password: savedPassword, mode: mode)
                return
            }
            passwordPrompt = PasswordPromptRequest(
                profile: profile,
                mode: mode,
                reason: "Enter the password for this host. You can save it in Keychain for faster connections next time."
            )
            return
        }

        let keys = storedPrivateKeyAuths(for: profile)
        guard !keys.isEmpty else {
            if profile.authentication == .automatic {
                if let savedPassword {
                    startPasswordConnection(profile: profile, password: savedPassword, mode: mode)
                } else {
                    passwordPrompt = PasswordPromptRequest(
                        profile: profile,
                        mode: mode,
                        reason: "No saved SSH key is available. Enter the password for this host."
                    )
                }
            } else {
                connectionNotice = ConnectionNotice(
                    title: "SSH Key Unavailable",
                    message: profile.preferredKeyID == nil
                        ? "This profile only uses SSH keys, but no saved key is available."
                        : "The SSH key selected for this profile is no longer available."
                )
            }
            return
        }

        let didConnect: Bool
        switch mode {
        case .terminal:
            didConnect = await tryPrivateKeysForTerminal(profile: profile, keys: keys)
        case .codex:
            didConnect = await tryPrivateKeysForCodex(profile: profile, keys: keys)
        case .claude:
            didConnect = await tryPrivateKeysForClaude(profile: profile, keys: keys)
        }

        if !didConnect {
            if pendingHostKeyTrust != nil {
                return
            }
            reconnectTarget = nil
            codexReconnectTarget = nil
            claudeReconnectTarget = nil
            path = []
            if profile.authentication == .automatic {
                if let savedPassword {
                    startPasswordConnection(profile: profile, password: savedPassword, mode: mode)
                } else {
                    passwordPrompt = PasswordPromptRequest(
                        profile: profile,
                        mode: mode,
                        reason: "Saved SSH keys did not authenticate. Enter a password to continue."
                    )
                }
            } else {
                connectionNotice = ConnectionNotice(
                    title: "SSH Key Authentication Failed",
                    message: "None of the SSH keys selected for this profile authenticated successfully. Password fallback is disabled."
                )
            }
        }
    }

    private func startPasswordConnection(
        profile: HostProfile,
        password: String,
        mode: HostConnectMode
    ) {
        Task {
            await probeAndStoreCapabilities(
                for: profile,
                credential: .password(password)
            )
        }

        switch mode {
        case .terminal:
            let startupCommand = profile.terminalStartupCommand
            reconnectTarget = .password(
                profile: profile,
                password: password,
                startupCommand: startupCommand
            )
            session = .connecting(to: profile)
            showTerminal()
            Task {
                await connectPasswordShell(
                    profile: profile,
                    password: password,
                    startupCommand: startupCommand
                )
            }
        case .codex:
            codexReconnectTarget = .password(profile: profile, password: password, cwd: "", threadID: nil)
            codexSession = .connecting(to: profile, cwd: "")
            showCodex()
            Task {
                await startCodexPassword(profile: profile, password: password, cwd: "")
            }
        case .claude:
            claudeReconnectTarget = .password(profile: profile, password: password, cwd: "", sessionID: nil)
            claudeSession = .connecting(to: profile, cwd: "")
            showClaude()
            Task {
                updateClaudeSession(await coreSession.startClaudePassword(
                    to: profile,
                    password: password,
                    cwd: ""
                ))
            }
        }
    }

    @MainActor
    private func tryPrivateKeysForTerminal(
        profile: HostProfile,
        keys: [StoredPrivateKeyAuth]
    ) async -> Bool {
        session = .connecting(to: profile)
        showTerminal()

        for key in keys {
            let startupCommand = profile.terminalStartupCommand
            reconnectTarget = .privateKey(
                profile: profile,
                privateKeyPEM: key.privateKeyPEM,
                passphrase: key.passphrase,
                startupCommand: startupCommand
            )
            await connectPrivateKeyShell(
                profile: profile,
                privateKeyPEM: key.privateKeyPEM,
                passphrase: key.passphrase,
                startupCommand: startupCommand
            )

            let result = await waitForTerminalConnectionResult()
            if result.state == .connected {
                Task {
                    await probeAndStoreCapabilities(
                        for: profile,
                        credential: .privateKey(
                            privateKeyPEM: key.privateKeyPEM,
                            passphrase: key.passphrase
                        )
                    )
                }
                return true
            }

            if pendingHostKeyTrust != nil {
                return true
            }

            _ = coreSession.disconnectLiveShell()
        }

        return false
    }

    @MainActor
    private func tryPrivateKeysForCodex(
        profile: HostProfile,
        keys: [StoredPrivateKeyAuth]
    ) async -> Bool {
        for key in keys {
            codexReconnectTarget = .privateKey(
                profile: profile,
                privateKeyPEM: key.privateKeyPEM,
                passphrase: key.passphrase,
                cwd: "",
                threadID: nil
            )
            codexSession = .connecting(to: profile, cwd: "")
            showCodex()
            updateCodexSession(
                await coreSession.startCodexPrivateKey(
                    to: profile,
                    privateKeyPEM: key.privateKeyPEM,
                    passphrase: key.passphrase,
                    cwd: ""
                )
            )

            let result = await waitForCodexConnectionResult()
            if result.status == .connected {
                Task {
                    await probeAndStoreCapabilities(
                        for: profile,
                        credential: .privateKey(
                            privateKeyPEM: key.privateKeyPEM,
                            passphrase: key.passphrase
                        )
                    )
                }
                return true
            }

            if pendingHostKeyTrust != nil {
                return true
            }

            _ = coreSession.disconnectCodex()
        }

        return false
    }

    @MainActor
    private func tryPrivateKeysForClaude(
        profile: HostProfile,
        keys: [StoredPrivateKeyAuth]
    ) async -> Bool {
        claudeSession = .connecting(to: profile, cwd: "")
        showClaude()

        for key in keys {
            claudeReconnectTarget = .privateKey(
                profile: profile,
                privateKeyPEM: key.privateKeyPEM,
                passphrase: key.passphrase,
                cwd: "",
                sessionID: nil
            )
            updateClaudeSession(await coreSession.startClaudePrivateKey(
                to: profile,
                privateKeyPEM: key.privateKeyPEM,
                passphrase: key.passphrase,
                cwd: ""
            ))
            let result = await waitForClaudeConnectionResult()
            if result.status == .connected {
                return true
            }
            _ = coreSession.disconnectClaude()
        }
        return false
    }

    @MainActor
    private func waitForTerminalConnectionResult() async -> TerminalSession {
        let deadline = Date().addingTimeInterval(8)
        var current = session

        while current.state == .connecting && Date() < deadline {
            try? await Task.sleep(nanoseconds: 200_000_000)
            current = coreSession.pollLiveShell()
            updateSession(current)
        }

        return current
    }

    @MainActor
    private func waitForCodexConnectionResult() async -> CodexSnapshot {
        let deadline = Date().addingTimeInterval(10)
        var current = codexSession

        while current.status == .connecting && Date() < deadline {
            try? await Task.sleep(nanoseconds: 250_000_000)
            current = coreSession.pollCodex()
            updateCodexSession(current)
        }

        return current
    }

    @MainActor
    private func waitForClaudeConnectionResult() async -> CodexSnapshot {
        let deadline = Date().addingTimeInterval(15)
        var current = claudeSession
        while current.status == .connecting && Date() < deadline {
            try? await Task.sleep(nanoseconds: 250_000_000)
            current = coreSession.pollClaude()
            updateClaudeSession(current)
        }
        return current
    }

    private func storedPrivateKeyAuths(for profile: HostProfile? = nil) -> [StoredPrivateKeyAuth] {
        sshKeys.compactMap { credential in
            if let profile,
               profile.authentication == .privateKey,
               let preferredKeyID = profile.preferredKeyID,
               credential.id != preferredKeyID {
                return nil
            }
            guard
                let privateKeyPEM = secretStore.loadSecret(forKeyID: credential.id, kind: .privateKey),
                privateKeyLooksUsable(privateKeyPEM)
            else {
                return nil
            }

            return StoredPrivateKeyAuth(
                credential: credential,
                privateKeyPEM: privateKeyPEM,
                passphrase: secretStore.loadSecret(forKeyID: credential.id, kind: .passphrase)
            )
        }
    }

    @MainActor
    private func probeHostCapabilities(_ profile: HostProfile) async -> RemoteHostProbeOutcome {
        if let password = secretStore.loadSecret(for: profile, kind: .password) {
            return await runHostCapabilityProbe(
                profile: profile,
                credential: .password(password)
            )
        }

        let keys = storedPrivateKeyAuths()
        guard !keys.isEmpty else {
            return .failure("Save a password or private key before checking this host.")
        }

        var lastFailure = "No saved private key authenticated."
        for key in keys {
            let outcome = await runHostCapabilityProbe(
                profile: profile,
                credential: .privateKey(
                    privateKeyPEM: key.privateKeyPEM,
                    passphrase: key.passphrase
                )
            )
            if outcome.report != nil {
                return outcome
            }
            lastFailure = outcome.errorMessage ?? lastFailure
        }
        return .failure(lastFailure)
    }

    @MainActor
    private func loadRemoteTerminalSessions(
        profile: HostProfile,
        configuration: PersistentTerminalConfiguration
    ) async -> RemoteTerminalSessionCatalog {
        var credentials: [HostProbeCredential] = []

        if let reconnectTarget, reconnectTarget.profile.id == profile.id {
            switch reconnectTarget {
            case .preview:
                break
            case .password(_, let password, _):
                credentials.append(.password(password))
            case .privateKey(_, let privateKeyPEM, let passphrase, _):
                credentials.append(.privateKey(privateKeyPEM: privateKeyPEM, passphrase: passphrase))
            }
        }

        if let savedPassword = secretStore.loadSecret(for: profile, kind: .password) {
            credentials.append(.password(savedPassword))
        }
        credentials.append(contentsOf: storedPrivateKeyAuths().map {
            .privateKey(privateKeyPEM: $0.privateKeyPEM, passphrase: $0.passphrase)
        })

        guard !credentials.isEmpty else {
            return RemoteTerminalSessionCatalog(
                sessions: [],
                errorMessage: "Save an SSH credential to load remote sessions."
            )
        }

        let command = RemoteTerminalSessionProbe.command(for: configuration.backend)
        var lastError = "The host did not return a recognizable session list."
        for credential in credentials {
            let probeSession = ShellowCoreSession()
            let result: TerminalSession
            switch credential {
            case .password(let password):
                result = await probeSession.connectPasswordExec(
                    to: profile,
                    password: password,
                    command: command
                )
            case .privateKey(let privateKeyPEM, let passphrase):
                result = await probeSession.connectPrivateKeyExec(
                    to: profile,
                    privateKeyPEM: privateKeyPEM,
                    passphrase: passphrase,
                    command: command
                )
            }

            let output = result.rows.map(\.text).joined(separator: "\n")
            if let catalog = RemoteTerminalSessionProbe.parse(output) {
                return catalog
            }
            if let detail = result.rows.reversed()
                .map(\.text)
                .first(where: { !$0.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty }) {
                lastError = detail
            }
        }

        return RemoteTerminalSessionCatalog(sessions: [], errorMessage: lastError)
    }

    private func runHostCapabilityProbe(
        profile: HostProfile,
        credential: HostProbeCredential
    ) async -> RemoteHostProbeOutcome {
        let probeSession = ShellowCoreSession()
        let result: TerminalSession
        switch credential {
        case .password(let password):
            result = await probeSession.connectPasswordExec(
                to: profile,
                password: password,
                command: RemoteHostCapabilityProbe.command
            )
        case .privateKey(let privateKeyPEM, let passphrase):
            result = await probeSession.connectPrivateKeyExec(
                to: profile,
                privateKeyPEM: privateKeyPEM,
                passphrase: passphrase,
                command: RemoteHostCapabilityProbe.command
            )
        }

        let output = result.rows.map(\.text).joined(separator: "\n")
        if let report = RemoteHostCapabilityProbe.parse(output) {
            return .success(report)
        }

        let detail = result.rows.reversed()
            .map(\.text)
            .first { !$0.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty }
        return .failure(detail ?? "The host returned an unreadable capability report.")
    }

    @MainActor
    private func probeAndStoreCapabilities(
        for profile: HostProfile,
        credential: HostProbeCredential
    ) async {
        guard profile.capabilityReport == nil || profile.capabilityReport?.isStale == true else { return }
        let outcome = await runHostCapabilityProbe(profile: profile, credential: credential)
        guard let report = outcome.report,
              let index = profiles.firstIndex(where: { $0.id == profile.id }) else { return }
        profiles[index].capabilityReport = report
    }

    private func showTerminal() {
        guard path.last != .terminal else { return }
        path.append(.terminal)
    }

    private func showCodex() {
        guard path.last != .codex else { return }
        path.append(.codex)
    }

    private func showClaude() {
        guard path.last != .claude else { return }
        path.append(.claude)
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
        captureHostKeyConfirmationIfNeeded(from: next)
        captureObservedHostKeyIfNeeded(from: next)
    }

    private func updateCodexSession(_ next: CodexSnapshot) {
#if DEBUG
        print("[Shellow Codex] app update snapshot status=\(next.status.rawValue) threadId=\(next.threadId ?? "nil") messages=\(next.messages.count) threads=\(next.threads.threads.count) opRunning=\(next.operation.isRunning) opError=\(next.operation.lastError ?? "")")
#endif
        var resolved = next
        if !next.messagesReplaceAll, next.messagesStartIndex <= codexSession.messages.count {
            var messages = codexSession.messages
            messages.replaceSubrange(next.messagesStartIndex..., with: next.messages)
            resolved.messages = messages
        }
        codexSession = resolved
        captureHostKeyConfirmationIfNeeded(from: resolved)
        rememberCodexResumePoint(from: resolved)
        captureObservedHostKeyIfNeeded(from: resolved)
        if resolved.lastError?.contains("daemon bootstrap --remote-control") == true,
           codexBootstrapPromptEndpoint == nil,
           codexBootstrapError == nil {
            codexBootstrapPromptEndpoint = codexReconnectTarget?.profile.endpoint ?? resolved.endpoint
        }
    }

    private func updateClaudeSession(_ next: CodexSnapshot) {
        claudeSession = next
        captureHostKeyConfirmationIfNeeded(from: next, mode: .claude)
        if let sessionID = next.threadId?.trimmingCharacters(in: .whitespacesAndNewlines),
           !sessionID.isEmpty,
           let target = claudeReconnectTarget {
            switch target {
            case .password(let profile, let password, let cwd, _):
                claudeReconnectTarget = .password(
                    profile: profile,
                    password: password,
                    cwd: next.cwd ?? cwd,
                    sessionID: sessionID
                )
            case .privateKey(let profile, let key, let passphrase, let cwd, _):
                claudeReconnectTarget = .privateKey(
                    profile: profile,
                    privateKeyPEM: key,
                    passphrase: passphrase,
                    cwd: next.cwd ?? cwd,
                    sessionID: sessionID
                )
            }
        }
    }

    private func hostKeyFingerprint(in message: String?) -> String? {
        guard let message,
              let range = message.range(of: hostKeyConfirmationPrefix) else { return nil }
        let fingerprint = message[range.upperBound...]
            .split(whereSeparator: { $0.isWhitespace })
            .first
            .map(String.init)?
            .trimmingCharacters(in: .whitespacesAndNewlines)
        return fingerprint?.isEmpty == false ? fingerprint : nil
    }

    private func captureHostKeyConfirmationIfNeeded(from snapshot: TerminalSession) {
        guard pendingHostKeyTrust == nil,
              let fingerprint = snapshot.rows.reversed().compactMap({ hostKeyFingerprint(in: $0.text) }).first,
              reconnectTarget?.profile.trustedHostKeySHA256?.isEmpty ?? true else { return }
        pendingHostKeyTrust = PendingHostKeyTrust(fingerprint: fingerprint, mode: .terminal)
    }

    private func captureHostKeyConfirmationIfNeeded(
        from snapshot: CodexSnapshot,
        mode: HostConnectMode = .codex
    ) {
        let trustedHostKey: String?
        switch mode {
        case .terminal:
            trustedHostKey = reconnectTarget?.profile.trustedHostKeySHA256
        case .codex:
            trustedHostKey = codexReconnectTarget?.profile.trustedHostKeySHA256
        case .claude:
            trustedHostKey = claudeReconnectTarget?.profile.trustedHostKeySHA256
        }
        guard pendingHostKeyTrust == nil,
              let fingerprint = hostKeyFingerprint(in: snapshot.lastError)
                ?? snapshot.messages.reversed().compactMap({ hostKeyFingerprint(in: $0.text) }).first,
              trustedHostKey?.isEmpty ?? true else { return }
        pendingHostKeyTrust = PendingHostKeyTrust(fingerprint: fingerprint, mode: mode)
    }

    private func cancelPendingHostKeyTrust() {
        guard let pending = pendingHostKeyTrust else { return }
        pendingHostKeyTrust = nil
        switch pending.mode {
        case .terminal:
            reconnectTarget = nil
            updateSession(coreSession.disconnectLiveShell())
        case .codex:
            codexReconnectTarget = nil
            updateCodexSession(coreSession.disconnectCodex())
        case .claude:
            claudeReconnectTarget = nil
            updateClaudeSession(coreSession.disconnectClaude())
        }
    }

    private func trustPendingHostKeyAndReconnect() {
        guard let pending = pendingHostKeyTrust else { return }
        pendingHostKeyTrust = nil
        switch pending.mode {
        case .terminal:
            guard let target = reconnectTarget else { return }
            var profile = target.profile
            profile.trustedHostKeySHA256 = pending.fingerprint
            if let index = profiles.firstIndex(where: { $0.id == profile.id }) {
                profiles[index] = profile
            }
            reconnectTarget = target.replacingProfile(profile)
            reconnect()
        case .codex:
            guard let target = codexReconnectTarget else { return }
            var profile = target.profile
            profile.trustedHostKeySHA256 = pending.fingerprint
            if let index = profiles.firstIndex(where: { $0.id == profile.id }) {
                profiles[index] = profile
            }
            codexReconnectTarget = target.replacingProfile(profile)
            reconnectCodex()
        case .claude:
            guard let target = claudeReconnectTarget else { return }
            var profile = target.profile
            profile.trustedHostKeySHA256 = pending.fingerprint
            if let index = profiles.firstIndex(where: { $0.id == profile.id }) {
                profiles[index] = profile
            }
            claudeReconnectTarget = target.replacingProfile(profile)
            reconnectClaude()
        }
    }

    @MainActor
    private func bootstrapRemoteCodexAndReconnect() async {
        guard let target = codexReconnectTarget else { return }
        _ = coreSession.disconnectCodex()
        let setupSession = ShellowCoreSession()
        let result: TerminalSession
        switch target {
        case .password(let profile, let password, _, _):
            result = await setupSession.connectPasswordExec(
                to: profile,
                password: password,
                command: codexRemoteControlBootstrapCommand
            )
        case .privateKey(let profile, let privateKeyPEM, let passphrase, _, _):
            result = await setupSession.connectPrivateKeyExec(
                to: profile,
                privateKeyPEM: privateKeyPEM,
                passphrase: passphrase,
                command: codexRemoteControlBootstrapCommand
            )
        }
        let output = result.rows.map(\.text).joined(separator: "\n")
        guard output.contains("__SHELLOW_CODEX_BOOTSTRAP_OK__") else {
            codexBootstrapError = result.rows.reversed()
                .map(\.text)
                .first { !$0.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty }
                ?? "The remote setup command did not complete successfully."
            return
        }
        reconnectCodex()
    }

    @MainActor
    private func startNewClaudeSession(cwd: String, initialMessage: String?) async {
        guard let target = claudeReconnectTarget else { return }
        switch target {
        case .password(let profile, let password, _, _):
            claudeReconnectTarget = .password(profile: profile, password: password, cwd: cwd, sessionID: nil)
            updateClaudeSession(await coreSession.startClaudePassword(
                to: profile,
                password: password,
                cwd: cwd
            ))
        case .privateKey(let profile, let key, let passphrase, _, _):
            claudeReconnectTarget = .privateKey(
                profile: profile,
                privateKeyPEM: key,
                passphrase: passphrase,
                cwd: cwd,
                sessionID: nil
            )
            updateClaudeSession(await coreSession.startClaudePrivateKey(
                to: profile,
                privateKeyPEM: key,
                passphrase: passphrase,
                cwd: cwd
            ))
        }
        _ = await waitForClaudeConnectionResult()
        if let initialMessage, claudeSession.status == .connected {
            updateClaudeSession(coreSession.sendClaudeMessage(initialMessage))
        }
    }

    private func reconnectClaude() {
        guard let target = claudeReconnectTarget else { return }
        showClaude()
        Task {
            switch target {
            case .password(let profile, let password, let cwd, let sessionID):
                updateClaudeSession(await coreSession.startClaudePassword(
                    to: profile,
                    password: password,
                    cwd: cwd,
                    sessionId: sessionID ?? ""
                ))
            case .privateKey(let profile, let key, let passphrase, let cwd, let sessionID):
                updateClaudeSession(await coreSession.startClaudePrivateKey(
                    to: profile,
                    privateKeyPEM: key,
                    passphrase: passphrase,
                    cwd: cwd,
                    sessionId: sessionID ?? ""
                ))
            }
        }
    }

    private func advanceTerminalRenderTick() {
        terminalRenderTick &+= 1
    }

    private func handleScenePhaseChange(_ phase: ScenePhase) {
        switch phase {
        case .background:
            reconnectTerminalAfterBackground = session.state != .disconnected && reconnectTarget != nil
            reconnectCodexAfterBackground = codexSession.status != .disconnected && codexReconnectTarget != nil
        case .active:
            let terminal = coreSession.pollLiveShell()
            updateSession(terminal)
#if DEBUG
            if ProcessInfo.processInfo.arguments.contains("--shellow-simulator-show-codex-usage") {
                return
            }
#endif
            let codex = coreSession.pollCodex()
            updateCodexSession(codex)
            if reconnectTerminalAfterBackground, terminal.state == .disconnected {
                reconnect()
            }
            if reconnectCodexAfterBackground, codex.status == .disconnected || codex.status == .failed {
                reconnectCodex()
            }
            reconnectTerminalAfterBackground = false
            reconnectCodexAfterBackground = false
        default:
            break
        }
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

    private func captureObservedHostKeyIfNeeded(from snapshot: CodexSnapshot) {
        guard
            let observed = snapshot.observedHostKeySha256?.trimmingCharacters(in: .whitespacesAndNewlines),
            !observed.isEmpty,
            let codexReconnectTarget
        else {
            return
        }

        let profile = codexReconnectTarget.profile
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
        self.codexReconnectTarget = codexReconnectTarget.replacingProfile(updatedProfile)
    }

    private func rememberCodexResumePoint(from snapshot: CodexSnapshot) {
        guard var target = codexReconnectTarget else {
            return
        }

        if let cwd = snapshot.cwd?.trimmingCharacters(in: .whitespacesAndNewlines), !cwd.isEmpty {
            target = target.replacingCwd(cwd)
        }
        if let threadID = snapshot.threadId?.trimmingCharacters(in: .whitespacesAndNewlines), !threadID.isEmpty {
            target = target.replacingThreadID(threadID)
        }

        codexReconnectTarget = target
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
        await sendTerminalStartupCommand(startupCommand)
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
        await sendTerminalStartupCommand(startupCommand)
    }

    @MainActor
    private func sendTerminalStartupCommand(_ startupCommand: String) async {
        let command = startupCommand.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !command.isEmpty else { return }

        let connected = await waitForTerminalConnectionResult()
        guard connected.state == .connected else { return }

        // The SSH channel can report connected just before the login banner and
        // first prompt finish rendering. Let that initial output settle so the
        // the multiplexer attach command is entered at a clean prompt.
        try? await Task.sleep(nanoseconds: 450_000_000)
        let settled = coreSession.pollLiveShell()
        updateSession(settled)
        guard settled.state == .connected else { return }

        updateSession(coreSession.sendTerminalInput(command + "\r"))
    }

    @MainActor
    private func startCodexPassword(
        profile: HostProfile,
        password: String,
        cwd: String
    ) async {
        updateCodexSession(await coreSession.startCodexPassword(
            to: profile,
            password: password,
            cwd: cwd
        ))
    }

    @MainActor
    private func startCodexPrivateKey(
        profile: HostProfile,
        privateKeyPEM: String,
        passphrase: String?,
        cwd: String
    ) async {
        updateCodexSession(await coreSession.startCodexPrivateKey(
            to: profile,
            privateKeyPEM: privateKeyPEM,
            passphrase: passphrase,
            cwd: cwd
        ))
    }

    @MainActor
    private func resumeCodexThreadAfterReconnect(_ threadID: String?) async {
        guard
            let threadID = threadID?.trimmingCharacters(in: .whitespacesAndNewlines),
            !threadID.isEmpty
        else {
            return
        }

        if codexSession.status == .connecting {
            _ = await waitForCodexConnectionResult()
        }

        guard codexSession.status == .connected else {
            return
        }

        updateCodexSession(await coreSession.resumeCodexThread(threadId: threadID))
    }

    private func reconnectCodex() {
        guard let codexReconnectTarget else { return }

        switch codexReconnectTarget {
        case .password(let profile, let password, let cwd, let threadID):
            let resumeThreadID = threadID ?? codexSession.threadId
            showCodex()
            Task {
                await startCodexPassword(profile: profile, password: password, cwd: cwd)
                await resumeCodexThreadAfterReconnect(resumeThreadID)
            }
        case .privateKey(let profile, let privateKeyPEM, let passphrase, let cwd, let threadID):
            let resumeThreadID = threadID ?? codexSession.threadId
            showCodex()
            Task {
                await startCodexPrivateKey(
                    profile: profile,
                    privateKeyPEM: privateKeyPEM,
                    passphrase: passphrase,
                    cwd: cwd
                )
                await resumeCodexThreadAfterReconnect(resumeThreadID)
            }
        }
    }

#if DEBUG
    @MainActor
    private func handleSimulatorLaunchRequestIfNeeded() async {
        let arguments = ProcessInfo.processInfo.arguments
        let knownRequests = [
            "--shellow-simulator-seed-local",
            "--shellow-simulator-show-password",
            "--shellow-simulator-show-settings",
            "--shellow-simulator-show-codex-usage",
            "--shellow-simulator-connect-terminal",
            "--shellow-simulator-connect-codex"
        ]
        guard arguments.contains(where: knownRequests.contains) else {
            return
        }

        if arguments.contains("--shellow-simulator-show-settings") {
            isSettingsPresented = true
            return
        }

        if arguments.contains("--shellow-simulator-show-codex-usage") {
            let resetBase = UInt64(Date().timeIntervalSince1970)
            var preview = CodexSnapshot.disconnected()
            preview.title = "Usage Preview"
            preview.endpoint = "preview.local"
            preview.cwd = "/Users/demo/Shellow"
            preview.status = .connected
            preview.threadId = "preview-thread"
            preview.messages = [
                CodexMessage(
                    id: "preview-user",
                    role: .user,
                    text: "Show the current Codex usage.",
                    kind: .userMessage
                ),
                CodexMessage(
                    id: "preview-assistant",
                    role: .assistant,
                    text: "Context and account limits are available from the usage ring in the header.",
                    kind: .finalAnswer
                )
            ]
            preview.usage = CodexUsageState(
                thread: CodexThreadTokenUsage(
                    last: CodexTokenUsageBreakdown(
                        cachedInputTokens: 18_240,
                        inputTokens: 36_500,
                        outputTokens: 2_800,
                        reasoningOutputTokens: 1_120,
                        totalTokens: 39_300
                    ),
                    total: CodexTokenUsageBreakdown(
                        cachedInputTokens: 42_600,
                        inputTokens: 81_200,
                        outputTokens: 7_400,
                        reasoningOutputTokens: 3_180,
                        totalTokens: 88_600
                    ),
                    modelContextWindow: 128_000
                ),
                rateLimits: CodexRateLimitSnapshot(
                    limitId: "codex",
                    limitName: "Codex",
                    planType: "plus",
                    primary: CodexRateLimitWindow(
                        usedPercent: 24,
                        resetsAt: resetBase + 3_600,
                        windowDurationMins: 300
                    ),
                    secondary: CodexRateLimitWindow(
                        usedPercent: 61,
                        resetsAt: resetBase + 172_800,
                        windowDurationMins: 10_080
                    ),
                    credits: CodexCreditsSnapshot(
                        hasCredits: true,
                        unlimited: false,
                        balance: "12.50"
                    ),
                    individualLimit: nil,
                    rateLimitReachedType: nil
                ),
                isLoadingRateLimits: false,
                rateLimitsError: nil
            )
            updateCodexSession(preview)
            showCodex()
            return
        }

        let profileID = UUID(uuidString: "E30DB0E4-3931-4D48-9919-84CB6FFAF54A")!
        var profile = profiles.first(where: { $0.id == profileID || $0.host == "10.248.1.102" }) ?? HostProfile(
            id: profileID,
            name: "Mac mini",
            host: "10.248.1.102",
            port: 22,
            username: "zinglix",
            authentication: .password,
            trustedHostKeySHA256: nil,
            lastConnected: nil
        )
        profile.name = "Mac mini"
        profile.host = "10.248.1.102"
        profile.port = 22
        profile.username = "zinglix"
        profile.authentication = .password

        if let index = profiles.firstIndex(where: { $0.id == profile.id || $0.host == profile.host }) {
            profiles[index] = profile
        } else {
            profiles.insert(profile, at: 0)
        }

        if arguments.contains("--shellow-simulator-seed-local") {
            let password = ProcessInfo.processInfo.environment["SHELLOW_SIMULATOR_PASSWORD"]
                ?? UIPasteboard.general.string
            if let password, !password.isEmpty {
                do {
                    try secretStore.saveSecret(password, for: profile, kind: .password)
                    UserDefaults.standard.set("saved", forKey: "shellow.simulatorCredentialStatus")
                    print("[Shellow Simulator] credential saved=\(secretStore.hasSecret(for: profile, kind: .password))")
                } catch {
                    UserDefaults.standard.set("failed", forKey: "shellow.simulatorCredentialStatus")
                    print("[Shellow Simulator] credential save failed: \(error)")
                }
                UIPasteboard.general.items = []
            } else {
                UserDefaults.standard.set("missing", forKey: "shellow.simulatorCredentialStatus")
                print("[Shellow Simulator] no credential was supplied")
            }
            return
        }

        if arguments.contains("--shellow-simulator-show-password") {
            passwordPrompt = PasswordPromptRequest(
                profile: profile,
                mode: .terminal,
                reason: "Authentication is required before the first connection."
            )
            return
        }

        guard let password = secretStore.loadSecret(for: profile, kind: .password) else {
            passwordPrompt = PasswordPromptRequest(
                profile: profile,
                mode: arguments.contains("--shellow-simulator-connect-codex") ? .codex : .terminal,
                reason: "Enter the saved password to continue."
            )
            return
        }

        if arguments.contains("--shellow-simulator-connect-codex") {
            startPasswordConnection(profile: profile, password: password, mode: .codex)
        } else if arguments.contains("--shellow-simulator-connect-terminal") {
            startPasswordConnection(profile: profile, password: password, mode: .terminal)
        }
    }
#endif
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

private extension CodexReconnectTarget {
    var profile: HostProfile {
        switch self {
        case .password(let profile, _, _, _), .privateKey(let profile, _, _, _, _):
            profile
        }
    }

    func replacingProfile(_ profile: HostProfile) -> CodexReconnectTarget {
        switch self {
        case .password(_, let password, let cwd, let threadID):
            .password(profile: profile, password: password, cwd: cwd, threadID: threadID)
        case .privateKey(_, let privateKeyPEM, let passphrase, let cwd, let threadID):
            .privateKey(profile: profile, privateKeyPEM: privateKeyPEM, passphrase: passphrase, cwd: cwd, threadID: threadID)
        }
    }

    func replacingCwd(_ cwd: String) -> CodexReconnectTarget {
        switch self {
        case .password(let profile, let password, _, let threadID):
            .password(profile: profile, password: password, cwd: cwd, threadID: threadID)
        case .privateKey(let profile, let privateKeyPEM, let passphrase, _, let threadID):
            .privateKey(profile: profile, privateKeyPEM: privateKeyPEM, passphrase: passphrase, cwd: cwd, threadID: threadID)
        }
    }

    func replacingThreadID(_ threadID: String?) -> CodexReconnectTarget {
        switch self {
        case .password(let profile, let password, let cwd, _):
            .password(profile: profile, password: password, cwd: cwd, threadID: threadID)
        case .privateKey(let profile, let privateKeyPEM, let passphrase, let cwd, _):
            .privateKey(profile: profile, privateKeyPEM: privateKeyPEM, passphrase: passphrase, cwd: cwd, threadID: threadID)
        }
    }
}

private extension ClaudeReconnectTarget {
    var profile: HostProfile {
        switch self {
        case .password(let profile, _, _, _), .privateKey(let profile, _, _, _, _):
            profile
        }
    }

    func replacingProfile(_ profile: HostProfile) -> ClaudeReconnectTarget {
        switch self {
        case .password(_, let password, let cwd, let sessionID):
            .password(profile: profile, password: password, cwd: cwd, sessionID: sessionID)
        case .privateKey(_, let privateKeyPEM, let passphrase, let cwd, let sessionID):
            .privateKey(
                profile: profile,
                privateKeyPEM: privateKeyPEM,
                passphrase: passphrase,
                cwd: cwd,
                sessionID: sessionID
            )
        }
    }
}

private func appShellMonotonicNanos() -> UInt64 {
    DispatchTime.now().uptimeNanoseconds
}

private func appShellElapsedMs(since start: UInt64) -> String {
    let now = DispatchTime.now().uptimeNanoseconds
    let elapsed = now >= start ? now - start : 0
    return String(format: "%.1f", Double(elapsed) / 1_000_000.0)
}

#Preview {
    AppShell()
}
