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
        let json = renderFrameJSON(width: width, height: height, firstRow: firstRow, rowCount: rowCount)
        guard let data = json.data(using: .utf8),
              let report = try? decoder.decode(RendererFrameReport.self, from: data)
        else {
            return false
        }
        return report.nativeSurfaceTerminalFramePresentedThisFrame
    }

    func rendererInfoJSON() -> String {
        withLockedEngine {
            takeString(shellow_engine_renderer_info_json(engine))
        }
    }

    func setRendererOverlayJSON(_ overlayJSON: String) -> String {
        withLockedEngine {
            overlayJSON.withCString { pointer in
                takeString(shellow_engine_set_renderer_overlay_json(engine, pointer))
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
        guard !json.contains("\"error\"") else {
            return .bridgeFailure(json)
        }

        do {
            return try decoder.decode(TerminalSession.self, from: Data(json.utf8))
        } catch {
            return .bridgeFailure("Failed to decode Rust snapshot: \(error)")
        }
    }

    private func takeString(_ pointer: UnsafeMutablePointer<CChar>?) -> String {
        guard let pointer else {
            return "{\"error\":\"Rust returned a null response\"}"
        }

        defer { shellow_string_free(pointer) }
        return String(cString: pointer)
    }
}

private struct RendererFrameReport: Decodable {
    let nativeSurfaceTerminalFramePresentedThisFrame: Bool
}

private extension AuthenticationKind {
    var ffiValue: UInt8 {
        switch self {
        case .password: 0
        case .privateKey: 1
        }
    }
}
