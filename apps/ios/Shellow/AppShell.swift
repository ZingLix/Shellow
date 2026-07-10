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

private enum HostConnectMode {
    case terminal
    case codex

    var passwordPromptTitle: String {
        switch self {
        case .terminal: "Terminal Password"
        case .codex: "Codex Password"
        }
    }
}

private struct PasswordPromptRequest: Identifiable {
    let id = UUID()
    var profile: HostProfile
    var mode: HostConnectMode
    var reason: String?
}

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
}

struct AppShell: View {
    @State private var path: [ShellowRoute] = []
    @State private var coreSession = ShellowCoreSession()
    @State private var session = TerminalSession.preview
    @State private var codexSession = CodexSnapshot.disconnected()
    @State private var profiles = HostProfileStore.load()
    @State private var sshKeys = SSHKeyCredentialStore.load()
    @State private var settings = ShellowSettingsStore.load()
    @State private var reconnectTarget: ReconnectTarget?
    @State private var codexReconnectTarget: CodexReconnectTarget?
    @State private var passwordPrompt: PasswordPromptRequest?
    @State private var isSettingsPresented = false
    @State private var terminalRenderTick = 0

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
                probeCapabilities: { profile in
                    await probeHostCapabilities(profile)
                }
            )
            .navigationDestination(for: ShellowRoute.self) { route in
                switch route {
                case .terminal:
                    terminalScreen
                case .codex:
                    codexScreen
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
        }
        .tint(ShellowTheme.accent)
        .preferredColorScheme(settings.colorScheme.preferredSwiftUIColorScheme)
        .task {
#if DEBUG
            await handleSimulatorLaunchRequestIfNeeded()
#endif
            updateSession(coreSession.snapshot())
            var lastLiveRevision = coreSession.liveShellEventRevision()
            var lastCodexRevision = coreSession.codexEventRevision()
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

                let codexRevision = coreSession.codexEventRevision()
                if codexRevision != lastCodexRevision {
                    lastCodexRevision = codexRevision
                    updateCodexSession(coreSession.pollCodex())
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
        }
    }

    private var terminalScreen: some View {
        TerminalScreen(
            session: $session,
            settings: settings,
            renderTick: terminalRenderTick,
            persistentTerminal: reconnectTarget?.profile.persistentTerminal,
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

    private var codexScreen: some View {
        CodexScreen(
            snapshot: codexSession,
            onSendMessage: { message in
                updateCodexSession(coreSession.sendCodexMessage(message))
            },
            onUpdateSettings: { model, approvalPolicy, sandbox in
                updateCodexSession(coreSession.updateCodexSettings(
                    model: model,
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
                print("[Shellow Codex] app resume start threadId=\(threadId)")
                let next = await coreSession.resumeCodexThread(threadId: threadId)
                print("[Shellow Codex] app resume received elapsed_ms=\(appShellElapsedMs(since: started)) threadId=\(next.threadId ?? "nil") messages=\(next.messages.count) opError=\(next.operation.lastError ?? "")")
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

    @MainActor
    private func connectHost(_ profile: HostProfile, mode: HostConnectMode) async {
        if let savedPassword = secretStore.loadSecret(for: profile, kind: .password) {
            startPasswordConnection(profile: profile, password: savedPassword, mode: mode)
            return
        }

        if profile.authentication == .password {
            passwordPrompt = PasswordPromptRequest(
                profile: profile,
                mode: mode,
                reason: "Enter the password for this host. You can save it in Keychain for faster connections next time."
            )
            return
        }

        let keys = storedPrivateKeyAuths()
        guard !keys.isEmpty else {
            passwordPrompt = PasswordPromptRequest(
                profile: profile,
                mode: mode,
                reason: "No saved SSH key is available. Enter a password to connect instead."
            )
            return
        }

        let didConnect: Bool
        switch mode {
        case .terminal:
            didConnect = await tryPrivateKeysForTerminal(profile: profile, keys: keys)
        case .codex:
            didConnect = await tryPrivateKeysForCodex(profile: profile, keys: keys)
        }

        if !didConnect {
            reconnectTarget = nil
            codexReconnectTarget = nil
            passwordPrompt = PasswordPromptRequest(
                profile: profile,
                mode: mode,
                reason: "Saved SSH keys did not authenticate. Enter a password to continue."
            )
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

            _ = coreSession.disconnectLiveShell()
        }

        return false
    }

    @MainActor
    private func tryPrivateKeysForCodex(
        profile: HostProfile,
        keys: [StoredPrivateKeyAuth]
    ) async -> Bool {
        codexSession = .connecting(to: profile, cwd: "")
        showCodex()

        for key in keys {
            codexReconnectTarget = .privateKey(
                profile: profile,
                privateKeyPEM: key.privateKeyPEM,
                passphrase: key.passphrase,
                cwd: "",
                threadID: nil
            )
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

            _ = coreSession.disconnectCodex()
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

    private func storedPrivateKeyAuths() -> [StoredPrivateKeyAuth] {
        sshKeys.compactMap { credential in
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

    private func updateCodexSession(_ next: CodexSnapshot) {
        print("[Shellow Codex] app update snapshot status=\(next.status.rawValue) threadId=\(next.threadId ?? "nil") messages=\(next.messages.count) threads=\(next.threads.threads.count) opRunning=\(next.operation.isRunning) opError=\(next.operation.lastError ?? "")")
        codexSession = next
        rememberCodexResumePoint(from: next)
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
