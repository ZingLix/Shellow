import Foundation
import ShellowCore

final class ShellowCoreSession: @unchecked Sendable {
    private var engine: UnsafeMutableRawPointer?
    private let decoder: JSONDecoder
    private let lock = NSLock()

    init() {
        engine = shellow_engine_create()
        decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
    }

    deinit {
        if let engine {
            shellow_engine_destroy(engine)
        }
    }

    func snapshot() -> TerminalSession {
        withLockedEngine {
            decode(shellow_engine_snapshot_json(engine))
        }
    }

    func renderFrameJSON(width: Int, height: Int) -> String {
        withLockedEngine {
            takeString(
                shellow_engine_render_frame_json(
                    engine,
                    UInt32(clamping: width),
                    UInt32(clamping: height)
                )
            )
        }
    }

    func renderFrameJSON(width: Int, height: Int, firstRow: Int, rowCount: Int) -> String {
        withLockedEngine {
            takeString(
                shellow_engine_render_frame_viewport_json(
                    engine,
                    UInt32(clamping: width),
                    UInt32(clamping: height),
                    UInt32(clamping: firstRow),
                    UInt32(clamping: rowCount)
                )
            )
        }
    }

    func renderRendererSurfaceFrame(width: Int, height: Int) -> Bool {
        let json = renderFrameJSON(width: width, height: height)
        guard let data = json.data(using: .utf8),
              let report = try? decoder.decode(RendererFrameReport.self, from: data)
        else {
            return false
        }
        return report.nativeSurfaceTerminalFramePresentedThisFrame
    }

    func renderRendererSurfaceFrame(width: Int, height: Int, firstRow: Int, rowCount: Int) -> Bool {
        withLockedEngine {
            shellow_engine_render_surface_frame_presented(
                engine,
                UInt32(clamping: width),
                UInt32(clamping: height),
                UInt32(clamping: firstRow),
                UInt32(clamping: rowCount)
            )
        }
    }

    func rendererInfoJSON() -> String {
        withLockedEngine {
            takeString(shellow_engine_renderer_info_json(engine))
        }
    }

    func liveShellEventRevision() -> UInt64 {
        withLockedEngine {
            shellow_engine_live_shell_event_revision(engine)
        }
    }

    func codexEventRevision() -> UInt64 {
        withLockedEngine {
            shellow_engine_codex_event_revision(engine)
        }
    }

    func claudeEventRevision() -> UInt64 {
        withLockedEngine {
            shellow_engine_claude_event_revision(engine)
        }
    }

    func setRendererOverlayJSON(_ overlayJSON: String) -> String {
        withLockedEngine {
            overlayJSON.withCString { pointer in
                takeString(shellow_engine_set_renderer_overlay_json(engine, pointer))
            }
        }
    }

    func setTerminalTheme(_ themeID: String) -> String {
        withLockedEngine {
            themeID.withCString { pointer in
                takeString(shellow_engine_set_terminal_theme_json(engine, pointer))
            }
        }
    }

    func attachCoreAnimationLayer(rawHandle: UInt64, width: Int, height: Int) -> String {
        withLockedEngine {
            takeString(
                shellow_engine_attach_core_animation_layer_json(
                    engine,
                    rawHandle,
                    UInt32(clamping: width),
                    UInt32(clamping: height)
                )
            )
        }
    }

    func detachRendererSurface() -> String {
        withLockedEngine {
            takeString(shellow_engine_detach_renderer_surface_json(engine))
        }
    }

    func sendCommand(_ command: String) -> TerminalSession {
        withLockedEngine {
            command.withCString { pointer in
                decode(shellow_engine_send_command_json(engine, pointer))
            }
        }
    }

    func sendTerminalInput(_ input: String) -> TerminalSession {
        withLockedEngine {
            input.withCString { pointer in
                decode(shellow_engine_send_terminal_input_json(engine, pointer))
            }
        }
    }

    func resizeTerminal(cols: Int, rows: Int) -> TerminalSession {
        withLockedEngine {
            decode(
                shellow_engine_resize_terminal_json(
                    engine,
                    UInt32(clamping: cols),
                    UInt32(clamping: rows)
                )
            )
        }
    }

    func clearTerminal() -> TerminalSession {
        withLockedEngine {
            decode(shellow_engine_clear_terminal_json(engine))
        }
    }

    func resetTerminal() -> TerminalSession {
        withLockedEngine {
            decode(shellow_engine_reset_terminal_json(engine))
        }
    }

    func connectPreview(to profile: HostProfile) -> TerminalSession {
        withLockedEngine {
            let trustedHostKeySHA256 = profile.trustedHostKeySHA256 ?? ""
            return profile.name.withCString { name in
                profile.host.withCString { host in
                    profile.username.withCString { username in
                        trustedHostKeySHA256.withCString { trustedHostKeySHA256 in
                            decode(
                                shellow_engine_connect_preview_json(
                                    engine,
                                    name,
                                    host,
                                    UInt16(clamping: profile.port),
                                    username,
                                    trustedHostKeySHA256,
                                    profile.authentication.ffiValue
                                )
                            )
                        }
                    }
                }
            }
        }
    }

    func connectPasswordExec(to profile: HostProfile, password: String, command: String) async -> TerminalSession {
        await withCheckedContinuation { continuation in
            DispatchQueue.global(qos: .userInitiated).async {
                let result = self.withLockedEngine {
                    let trustedHostKeySHA256 = profile.trustedHostKeySHA256 ?? ""
                    return profile.name.withCString { name in
                        profile.host.withCString { host in
                            profile.username.withCString { username in
                                trustedHostKeySHA256.withCString { trustedHostKeySHA256 in
                                    password.withCString { password in
                                        command.withCString { command in
                                            self.decode(
                                                shellow_engine_connect_password_exec_json(
                                                    self.engine,
                                                    name,
                                                    host,
                                                    UInt16(clamping: profile.port),
                                                    username,
                                                    trustedHostKeySHA256,
                                                    password,
                                                    command
                                                )
                                            )
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                continuation.resume(returning: result)
            }
        }
    }

    func connectPrivateKeyExec(
        to profile: HostProfile,
        privateKeyPEM: String,
        passphrase: String?,
        command: String
    ) async -> TerminalSession {
        await withCheckedContinuation { continuation in
            DispatchQueue.global(qos: .userInitiated).async {
                let result = self.withLockedEngine {
                    let trustedHostKeySHA256 = profile.trustedHostKeySHA256 ?? ""
                    let passphrase = passphrase ?? ""
                    return profile.name.withCString { name in
                        profile.host.withCString { host in
                            profile.username.withCString { username in
                                trustedHostKeySHA256.withCString { trustedHostKeySHA256 in
                                    privateKeyPEM.withCString { privateKeyPEM in
                                        passphrase.withCString { passphrase in
                                            command.withCString { command in
                                                self.decode(
                                                    shellow_engine_connect_private_key_exec_json(
                                                        self.engine,
                                                        name,
                                                        host,
                                                        UInt16(clamping: profile.port),
                                                        username,
                                                        trustedHostKeySHA256,
                                                        privateKeyPEM,
                                                        passphrase,
                                                        command
                                                    )
                                                )
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                continuation.resume(returning: result)
            }
        }
    }

    func startPasswordShell(to profile: HostProfile, password: String) async -> TerminalSession {
        await withCheckedContinuation { continuation in
            DispatchQueue.global(qos: .userInitiated).async {
                let result = self.withLockedEngine {
                    let trustedHostKeySHA256 = profile.trustedHostKeySHA256 ?? ""
                    return profile.name.withCString { name in
                        profile.host.withCString { host in
                            profile.username.withCString { username in
                                trustedHostKeySHA256.withCString { trustedHostKeySHA256 in
                                    password.withCString { password in
                                        self.decode(
                                            shellow_engine_start_password_shell_json(
                                                self.engine,
                                                name,
                                                host,
                                                UInt16(clamping: profile.port),
                                                username,
                                                trustedHostKeySHA256,
                                                password
                                            )
                                        )
                                    }
                                }
                            }
                        }
                    }
                }
                continuation.resume(returning: result)
            }
        }
    }

    func startPrivateKeyShell(
        to profile: HostProfile,
        privateKeyPEM: String,
        passphrase: String?
    ) async -> TerminalSession {
        await withCheckedContinuation { continuation in
            DispatchQueue.global(qos: .userInitiated).async {
                let result = self.withLockedEngine {
                    let trustedHostKeySHA256 = profile.trustedHostKeySHA256 ?? ""
                    let passphrase = passphrase ?? ""
                    return profile.name.withCString { name in
                        profile.host.withCString { host in
                            profile.username.withCString { username in
                                trustedHostKeySHA256.withCString { trustedHostKeySHA256 in
                                    privateKeyPEM.withCString { privateKeyPEM in
                                        passphrase.withCString { passphrase in
                                            self.decode(
                                                shellow_engine_start_private_key_shell_json(
                                                    self.engine,
                                                    name,
                                                    host,
                                                    UInt16(clamping: profile.port),
                                                    username,
                                                    trustedHostKeySHA256,
                                                    privateKeyPEM,
                                                    passphrase
                                                )
                                            )
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                continuation.resume(returning: result)
            }
        }
    }

    func pollLiveShell() -> TerminalSession {
        withLockedEngine {
            decode(shellow_engine_poll_live_shell_json(engine))
        }
    }

    func disconnectLiveShell() -> TerminalSession {
        withLockedEngine {
            decode(shellow_engine_disconnect_live_shell_json(engine))
        }
    }

    func codexSnapshot() -> CodexSnapshot {
        withLockedEngine {
            decodeCodex(shellow_engine_codex_snapshot_json(engine), label: "snapshot")
        }
    }

    func startCodexPassword(to profile: HostProfile, password: String, cwd: String) async -> CodexSnapshot {
        await withCheckedContinuation { continuation in
            DispatchQueue.global(qos: .userInitiated).async {
                let result = self.withLockedEngine {
                    let trustedHostKeySHA256 = profile.trustedHostKeySHA256 ?? ""
                    return profile.name.withCString { name in
                        profile.host.withCString { host in
                            profile.username.withCString { username in
                                trustedHostKeySHA256.withCString { trustedHostKeySHA256 in
                                    password.withCString { password in
                                        cwd.withCString { cwd in
                                            self.decodeCodex(
                                                shellow_engine_start_codex_password_json(
                                                    self.engine,
                                                    name,
                                                    host,
                                                    UInt16(clamping: profile.port),
                                                    username,
                                                    trustedHostKeySHA256,
                                                    password,
                                                    cwd
                                                ),
                                                label: "start_password"
                                            )
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                continuation.resume(returning: result)
            }
        }
    }

    func startCodexPrivateKey(
        to profile: HostProfile,
        privateKeyPEM: String,
        passphrase: String?,
        cwd: String
    ) async -> CodexSnapshot {
        await withCheckedContinuation { continuation in
            DispatchQueue.global(qos: .userInitiated).async {
                let result = self.withLockedEngine {
                    let trustedHostKeySHA256 = profile.trustedHostKeySHA256 ?? ""
                    let passphrase = passphrase ?? ""
                    return profile.name.withCString { name in
                        profile.host.withCString { host in
                            profile.username.withCString { username in
                                trustedHostKeySHA256.withCString { trustedHostKeySHA256 in
                                    privateKeyPEM.withCString { privateKeyPEM in
                                        passphrase.withCString { passphrase in
                                            cwd.withCString { cwd in
                                                self.decodeCodex(
                                                    shellow_engine_start_codex_private_key_json(
                                                        self.engine,
                                                        name,
                                                        host,
                                                        UInt16(clamping: profile.port),
                                                        username,
                                                        trustedHostKeySHA256,
                                                        privateKeyPEM,
                                                        passphrase,
                                                        cwd
                                                    ),
                                                    label: "start_private_key"
                                                )
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                continuation.resume(returning: result)
            }
        }
    }

    func pollCodex() -> CodexSnapshot {
        withLockedEngine {
            decodeCodex(shellow_engine_poll_codex_json(engine), label: "poll")
        }
    }

    func sendCodexMessage(_ message: String) -> CodexSnapshot {
        withLockedEngine {
            message.withCString { pointer in
                decodeCodex(shellow_engine_send_codex_message_json(engine, pointer), label: "send_message")
            }
        }
    }

    func updateCodexSettings(
        model: String,
        reasoningEffort: String,
        serviceTier: String,
        approvalPolicy: String,
        sandbox: String
    ) -> CodexSnapshot {
        withLockedEngine {
            model.withCString { model in
                reasoningEffort.withCString { reasoningEffort in
                    serviceTier.withCString { serviceTier in
                        approvalPolicy.withCString { approvalPolicy in
                            sandbox.withCString { sandbox in
                                decodeCodex(
                                    shellow_engine_update_codex_settings_json(
                                        engine,
                                        model,
                                        reasoningEffort,
                                        serviceTier,
                                        approvalPolicy,
                                        sandbox
                                    ),
                                    label: "update_settings"
                                )
                            }
                        }
                    }
                }
            }
        }
    }

    func browseCodexDirectory(path: String) async -> CodexSnapshot {
        await withCheckedContinuation { continuation in
            DispatchQueue.global(qos: .userInitiated).async {
                let result = self.withLockedEngine {
                    path.withCString { path in
                        self.decodeCodex(
                            shellow_engine_browse_codex_directory_json(self.engine, path),
                            label: "browse_directory"
                        )
                    }
                }
                continuation.resume(returning: result)
            }
        }
    }

    func listCodexThreads(cwd: String, searchTerm: String) async -> CodexSnapshot {
        await listCodexThreadsPage(cwd: cwd, searchTerm: searchTerm, cursor: "", archived: false, append: false)
    }

    func listCodexThreadsPage(
        cwd: String,
        searchTerm: String,
        cursor: String,
        archived: Bool,
        append: Bool
    ) async -> CodexSnapshot {
        await withCheckedContinuation { continuation in
            DispatchQueue.global(qos: .userInitiated).async {
                let result = self.withLockedEngine {
                    cwd.withCString { cwd in
                        searchTerm.withCString { searchTerm in
                            cursor.withCString { cursor in
                                self.decodeCodex(
                                    shellow_engine_list_codex_threads_page_json(
                                        self.engine,
                                        cwd,
                                        searchTerm,
                                        cursor,
                                        archived,
                                        append
                                    ),
                                    label: "list_threads_page"
                                )
                            }
                        }
                    }
                }
                continuation.resume(returning: result)
            }
        }
    }

    func startCodexThread(cwd: String) async -> CodexSnapshot {
        await withCheckedContinuation { continuation in
            DispatchQueue.global(qos: .userInitiated).async {
                let result = self.withLockedEngine {
                    cwd.withCString { cwd in
                        self.decodeCodex(
                            shellow_engine_start_codex_thread_json(self.engine, cwd),
                            label: "start_thread"
                        )
                    }
                }
                continuation.resume(returning: result)
            }
        }
    }

    func resumeCodexThread(threadId: String) async -> CodexSnapshot {
        await withCheckedContinuation { continuation in
            DispatchQueue.global(qos: .userInitiated).async {
                let started = monotonicNanos()
                print("[Shellow Codex] bridge resume start threadId=\(threadId)")
                let result = self.withLockedEngine {
                    threadId.withCString { threadId in
                        self.decodeCodex(
                            shellow_engine_resume_codex_thread_json(self.engine, threadId),
                            label: "resume_thread"
                        )
                    }
                }
                print("[Shellow Codex] bridge resume done elapsed_ms=\(elapsedMs(since: started)) snapshotThreadId=\(result.threadId ?? "nil") messages=\(result.messages.count) opError=\(result.operation.lastError ?? "")")
                continuation.resume(returning: result)
            }
        }
    }

    func readCodexThread(threadId: String) async -> CodexSnapshot {
        await withCheckedContinuation { continuation in
            DispatchQueue.global(qos: .userInitiated).async {
                let result = self.withLockedEngine {
                    threadId.withCString { threadId in
                        self.decodeCodex(
                            shellow_engine_read_codex_thread_json(self.engine, threadId),
                            label: "read_thread"
                        )
                    }
                }
                continuation.resume(returning: result)
            }
        }
    }

    func loadMoreCodexThreadTurns(threadId: String, cursor: String) async -> CodexSnapshot {
        await withCheckedContinuation { continuation in
            DispatchQueue.global(qos: .userInitiated).async {
                let result = self.withLockedEngine {
                    threadId.withCString { threadId in
                        cursor.withCString { cursor in
                            self.decodeCodex(
                                shellow_engine_load_more_codex_thread_turns_json(
                                    self.engine,
                                    threadId,
                                    cursor
                                ),
                                label: "load_more_turns"
                            )
                        }
                    }
                }
                continuation.resume(returning: result)
            }
        }
    }

    func renameCodexThread(threadId: String, name: String) async -> CodexSnapshot {
        await withCheckedContinuation { continuation in
            DispatchQueue.global(qos: .userInitiated).async {
                let result = self.withLockedEngine {
                    threadId.withCString { threadId in
                        name.withCString { name in
                            self.decodeCodex(
                                shellow_engine_rename_codex_thread_json(self.engine, threadId, name),
                                label: "rename_thread"
                            )
                        }
                    }
                }
                continuation.resume(returning: result)
            }
        }
    }

    func archiveCodexThread(threadId: String) async -> CodexSnapshot {
        await runCodexThreadAction(threadId: threadId, shellow_engine_archive_codex_thread_json)
    }

    func unarchiveCodexThread(threadId: String) async -> CodexSnapshot {
        await runCodexThreadAction(threadId: threadId, shellow_engine_unarchive_codex_thread_json)
    }

    func deleteCodexThread(threadId: String) async -> CodexSnapshot {
        await runCodexThreadAction(threadId: threadId, shellow_engine_delete_codex_thread_json)
    }

    func forkCodexThread(threadId: String, cwd: String) async -> CodexSnapshot {
        await withCheckedContinuation { continuation in
            DispatchQueue.global(qos: .userInitiated).async {
                let result = self.withLockedEngine {
                    threadId.withCString { threadId in
                        cwd.withCString { cwd in
                            self.decodeCodex(
                                shellow_engine_fork_codex_thread_json(self.engine, threadId, cwd),
                                label: "fork_thread"
                            )
                        }
                    }
                }
                continuation.resume(returning: result)
            }
        }
    }

    func interruptCodexTurn() -> CodexSnapshot {
        withLockedEngine {
            decodeCodex(shellow_engine_interrupt_codex_turn_json(engine), label: "interrupt_turn")
        }
    }

    func answerCodexApproval(requestId: String, decision: String) -> CodexSnapshot {
        withLockedEngine {
            requestId.withCString { requestId in
                decision.withCString { decision in
                    decodeCodex(
                        shellow_engine_answer_codex_approval_json(engine, requestId, decision),
                        label: "answer_approval"
                    )
                }
            }
        }
    }

    func disconnectCodex() -> CodexSnapshot {
        withLockedEngine {
            decodeCodex(shellow_engine_disconnect_codex_json(engine), label: "disconnect")
        }
    }

    func claudeSnapshot() -> CodexSnapshot {
        withLockedEngine {
            decodeCodex(shellow_engine_claude_snapshot_json(engine), label: "claude_snapshot")
        }
    }

    func startClaudePassword(
        to profile: HostProfile,
        password: String,
        cwd: String,
        sessionId: String = ""
    ) async -> CodexSnapshot {
        await withCheckedContinuation { continuation in
            DispatchQueue.global(qos: .userInitiated).async {
                let result = self.withLockedEngine {
                    let trusted = profile.trustedHostKeySHA256 ?? ""
                    return profile.name.withCString { name in
                        profile.host.withCString { host in
                            profile.username.withCString { username in
                                trusted.withCString { trusted in
                                    password.withCString { password in
                                        cwd.withCString { cwd in
                                            sessionId.withCString { sessionId in
                                                self.decodeCodex(
                                                    shellow_engine_start_claude_password_json(
                                                        self.engine,
                                                        name,
                                                        host,
                                                        UInt16(clamping: profile.port),
                                                        username,
                                                        trusted,
                                                        password,
                                                        cwd,
                                                        sessionId
                                                    ),
                                                    label: "start_claude_password"
                                                )
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                continuation.resume(returning: result)
            }
        }
    }

    func startClaudePrivateKey(
        to profile: HostProfile,
        privateKeyPEM: String,
        passphrase: String?,
        cwd: String,
        sessionId: String = ""
    ) async -> CodexSnapshot {
        await withCheckedContinuation { continuation in
            DispatchQueue.global(qos: .userInitiated).async {
                let result = self.withLockedEngine {
                    let trusted = profile.trustedHostKeySHA256 ?? ""
                    let passphrase = passphrase ?? ""
                    return profile.name.withCString { name in
                        profile.host.withCString { host in
                            profile.username.withCString { username in
                                trusted.withCString { trusted in
                                    privateKeyPEM.withCString { key in
                                        passphrase.withCString { passphrase in
                                            cwd.withCString { cwd in
                                                sessionId.withCString { sessionId in
                                                    self.decodeCodex(
                                                        shellow_engine_start_claude_private_key_json(
                                                            self.engine,
                                                            name,
                                                            host,
                                                            UInt16(clamping: profile.port),
                                                            username,
                                                            trusted,
                                                            key,
                                                            passphrase,
                                                            cwd,
                                                            sessionId
                                                        ),
                                                        label: "start_claude_private_key"
                                                    )
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                continuation.resume(returning: result)
            }
        }
    }

    func pollClaude() -> CodexSnapshot {
        withLockedEngine {
            decodeCodex(shellow_engine_poll_claude_json(engine), label: "poll_claude")
        }
    }

    func sendClaudeMessage(_ message: String) -> CodexSnapshot {
        withLockedEngine {
            message.withCString { message in
                decodeCodex(shellow_engine_send_claude_message_json(engine, message), label: "send_claude")
            }
        }
    }

    func updateClaudeSettings(model: String, permissionMode: String) -> CodexSnapshot {
        withLockedEngine {
            model.withCString { model in
                permissionMode.withCString { permissionMode in
                    decodeCodex(
                        shellow_engine_update_claude_settings_json(engine, model, permissionMode),
                        label: "update_claude_settings"
                    )
                }
            }
        }
    }

    func interruptClaudeTurn() -> CodexSnapshot {
        withLockedEngine {
            decodeCodex(shellow_engine_interrupt_claude_turn_json(engine), label: "interrupt_claude")
        }
    }

    func answerClaudeApproval(requestId: String, decision: String) -> CodexSnapshot {
        withLockedEngine {
            requestId.withCString { requestId in
                decision.withCString { decision in
                    decodeCodex(
                        shellow_engine_answer_claude_approval_json(engine, requestId, decision),
                        label: "answer_claude_approval"
                    )
                }
            }
        }
    }

    func disconnectClaude() -> CodexSnapshot {
        withLockedEngine {
            decodeCodex(shellow_engine_disconnect_claude_json(engine), label: "disconnect_claude")
        }
    }

    private func runCodexThreadAction(
        threadId: String,
        _ action: @escaping (UnsafeMutableRawPointer?, UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar>?
    ) async -> CodexSnapshot {
        await withCheckedContinuation { continuation in
            DispatchQueue.global(qos: .userInitiated).async {
                let result = self.withLockedEngine {
                    threadId.withCString { threadId in
                        self.decodeCodex(action(self.engine, threadId))
                    }
                }
                continuation.resume(returning: result)
            }
        }
    }

    private func withLockedEngine<T>(_ body: () -> T) -> T {
        lock.lock()
        defer { lock.unlock() }
        return body()
    }

    private func decode(_ pointer: UnsafeMutablePointer<CChar>?) -> TerminalSession {
        guard let pointer else {
            return .bridgeFailure("Rust returned a null response")
        }

        defer { shellow_string_free(pointer) }

        let json = String(cString: pointer)
        guard !hasRootError(json) else {
            return .bridgeFailure(json)
        }

        do {
            return try decoder.decode(TerminalSession.self, from: Data(json.utf8))
        } catch {
            return .bridgeFailure("Failed to decode Rust snapshot: \(error)")
        }
    }

    private func decodeCodex(_ pointer: UnsafeMutablePointer<CChar>?, label: String = "codex") -> CodexSnapshot {
        let started = monotonicNanos()
        guard let pointer else {
            return .bridgeFailure("Rust returned a null Codex response")
        }

        defer { shellow_string_free(pointer) }

        let stringStarted = monotonicNanos()
        let json = String(cString: pointer)
        let stringMs = elapsedMs(since: stringStarted)
        guard !hasRootError(json) else {
            print("[Shellow Codex] bridge decode root_error label=\(label) bytes=\(json.utf8.count) total_ms=\(elapsedMs(since: started))")
            return .bridgeFailure(json)
        }

        do {
            let dataStarted = monotonicNanos()
            let data = Data(json.utf8)
            let dataMs = elapsedMs(since: dataStarted)
            let decodeStarted = monotonicNanos()
            let snapshot = try decoder.decode(CodexSnapshot.self, from: data)
            let messageTextBytes = snapshot.messages.reduce(0) { $0 + $1.text.utf8.count }
            let maxMessageTextBytes = snapshot.messages.map { $0.text.utf8.count }.max() ?? 0
            print("[Shellow Codex] bridge decode label=\(label) bytes=\(data.count) string_ms=\(stringMs) data_ms=\(dataMs) decode_ms=\(elapsedMs(since: decodeStarted)) total_ms=\(elapsedMs(since: started)) threadId=\(snapshot.threadId ?? "nil") detailThreadId=\(snapshot.threadDetail.thread?.id ?? "nil") messages=\(snapshot.messages.count) messageTextBytes=\(messageTextBytes) maxMessageTextBytes=\(maxMessageTextBytes) opError=\(snapshot.operation.lastError ?? "")")
            return snapshot
        } catch {
            print("[Shellow Codex] bridge decode failed label=\(label) bytes=\(json.utf8.count) total_ms=\(elapsedMs(since: started)) error=\(error)")
            return .bridgeFailure("Failed to decode Rust Codex snapshot: \(error)")
        }
    }

    private func takeString(_ pointer: UnsafeMutablePointer<CChar>?) -> String {
        guard let pointer else {
            return "{\"error\":\"Rust returned a null response\"}"
        }

        defer { shellow_string_free(pointer) }
        return String(cString: pointer)
    }

    private func hasRootError(_ json: String) -> Bool {
        json.trimmingCharacters(in: .whitespacesAndNewlines).hasPrefix("{\"error\":")
    }
}

private func monotonicNanos() -> UInt64 {
    DispatchTime.now().uptimeNanoseconds
}

private func elapsedMs(since start: UInt64) -> String {
    let now = DispatchTime.now().uptimeNanoseconds
    let elapsed = now >= start ? now - start : 0
    return String(format: "%.1f", Double(elapsed) / 1_000_000.0)
}

private struct RendererFrameReport: Decodable {
    let nativeSurfaceTerminalFramePresentedThisFrame: Bool
}

private extension AuthenticationKind {
    var ffiValue: UInt8 {
        switch self {
        case .automatic: 0
        case .password: 0
        case .privateKey: 1
        }
    }
}
