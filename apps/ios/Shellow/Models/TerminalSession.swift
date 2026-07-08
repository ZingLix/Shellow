import Foundation

struct TerminalSession: Equatable, Decodable {
    var title: String
    var host: String
    var state: ConnectionState
    var observedHostKeySha256: String?
    var pendingClipboardText: String?
    var clipboardSequence: Int
    var bellCount: Int
    var rows: [TerminalRow]
    var grid: TerminalGridSnapshot?
    var cursorColumn: Int
    var terminalCols: Int
    var terminalRows: Int
    var integration: IntegrationReport

    static func bridgeFailure(_ message: String) -> TerminalSession {
        TerminalSession(
            title: "Shellow",
            host: "bridge.error",
            state: .disconnected,
            observedHostKeySha256: nil,
            pendingClipboardText: nil,
            clipboardSequence: 0,
            bellCount: 0,
            rows: [
                TerminalRow(prompt: "", text: "Shellow Rust bridge failed", style: .warning),
                TerminalRow(prompt: "", text: message, style: .muted),
                TerminalRow(prompt: "$", text: "", style: .prompt)
            ],
            grid: nil,
            cursorColumn: 0,
            terminalCols: 80,
            terminalRows: 24,
            integration: .fallback
        )
    }

    static func connecting(to profile: HostProfile) -> TerminalSession {
        TerminalSession(
            title: profile.name,
            host: profile.endpoint,
            state: .connecting,
            observedHostKeySha256: nil,
            pendingClipboardText: nil,
            clipboardSequence: 0,
            bellCount: 0,
            rows: [
                TerminalRow(prompt: "$", text: "ssh \(profile.endpoint)", style: .command),
                TerminalRow(prompt: "", text: "Connecting...", style: .muted),
                TerminalRow(prompt: "$", text: "", style: .prompt)
            ],
            grid: nil,
            cursorColumn: 0,
            terminalCols: 80,
            terminalRows: 24,
            integration: .fallback
        )
    }
}

enum ConnectionState: String, Equatable, Decodable {
    case disconnected
    case connecting
    case connected

    var title: String {
        switch self {
        case .disconnected: "Offline"
        case .connecting: "Connecting"
        case .connected: "Connected"
        }
    }
}

struct TerminalRow: Identifiable, Equatable, Decodable {
    var id = UUID()
    var prompt: String
    var text: String
    var style: TerminalRowStyle

    static func == (lhs: TerminalRow, rhs: TerminalRow) -> Bool {
        lhs.prompt == rhs.prompt && lhs.text == rhs.text && lhs.style == rhs.style
    }

    private enum CodingKeys: String, CodingKey {
        case prompt
        case text
        case style
    }
}

enum TerminalRowStyle: String, Equatable, Decodable {
    case command
    case muted
    case success
    case prompt
    case warning
}

struct TerminalGridSnapshot: Equatable, Decodable {
    var cols: Int
    var rows: Int
    var cursorColumn: Int
    var cursorRow: Int
    var cursorVisible: Bool
    var cursorShape: TerminalCursorShape
    var activeScreen: TerminalScreenKind
    var scrollbackLen: Int
    var bracketedPaste: Bool
    var applicationCursorKeys: Bool
    var mouseReporting: Bool
    var mouseDragReporting: Bool
    var sgrMouse: Bool
    var lines: [String]
    var styledLines: [TerminalGridLine]
    var dirtyRows: [Int]

    var hasVisibleContent: Bool {
        lines.contains { !$0.trimmingCharacters(in: .whitespaces).isEmpty }
    }

    static func == (lhs: TerminalGridSnapshot, rhs: TerminalGridSnapshot) -> Bool {
        lhs.cols == rhs.cols &&
            lhs.rows == rhs.rows &&
            lhs.cursorColumn == rhs.cursorColumn &&
            lhs.cursorRow == rhs.cursorRow &&
            lhs.cursorVisible == rhs.cursorVisible &&
            lhs.cursorShape == rhs.cursorShape &&
            lhs.activeScreen == rhs.activeScreen &&
            lhs.scrollbackLen == rhs.scrollbackLen &&
            lhs.bracketedPaste == rhs.bracketedPaste &&
            lhs.applicationCursorKeys == rhs.applicationCursorKeys &&
            lhs.mouseReporting == rhs.mouseReporting &&
            lhs.mouseDragReporting == rhs.mouseDragReporting &&
            lhs.sgrMouse == rhs.sgrMouse &&
            lhs.lines == rhs.lines &&
            lhs.styledLines == rhs.styledLines &&
            lhs.dirtyRows == rhs.dirtyRows
    }

    private enum CodingKeys: String, CodingKey {
        case cols
        case rows
        case cursorColumn
        case cursorRow
        case cursorVisible
        case cursorShape
        case activeScreen
        case scrollbackLen
        case bracketedPaste
        case applicationCursorKeys
        case mouseReporting
        case mouseDragReporting
        case sgrMouse
        case lines
        case styledLines
        case dirtyRows
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        cols = try container.decode(Int.self, forKey: .cols)
        rows = try container.decode(Int.self, forKey: .rows)
        cursorColumn = try container.decode(Int.self, forKey: .cursorColumn)
        cursorRow = try container.decode(Int.self, forKey: .cursorRow)
        cursorVisible = try container.decode(Bool.self, forKey: .cursorVisible)
        cursorShape = try container.decode(TerminalCursorShape.self, forKey: .cursorShape)
        activeScreen = try container.decode(TerminalScreenKind.self, forKey: .activeScreen)
        scrollbackLen = try container.decode(Int.self, forKey: .scrollbackLen)
        bracketedPaste = try container.decode(Bool.self, forKey: .bracketedPaste)
        applicationCursorKeys = try container.decode(Bool.self, forKey: .applicationCursorKeys)
        mouseReporting = try container.decode(Bool.self, forKey: .mouseReporting)
        mouseDragReporting = try container.decodeIfPresent(Bool.self, forKey: .mouseDragReporting) ?? false
        sgrMouse = try container.decode(Bool.self, forKey: .sgrMouse)
        lines = try container.decode([String].self, forKey: .lines)
        styledLines = try container.decode([TerminalGridLine].self, forKey: .styledLines)
        dirtyRows = try container.decodeIfPresent([Int].self, forKey: .dirtyRows) ?? []
    }
}

enum TerminalCursorShape: String, Equatable, Decodable {
    case block
    case underline
    case bar
}

struct TerminalGridLine: Equatable, Decodable {
    var runs: [TerminalGridRun]
}

struct TerminalGridRun: Equatable, Decodable {
    var text: String
    var style: TerminalGridStyle
}

struct TerminalGridStyle: Equatable, Decodable {
    var bold: Bool
    var faint: Bool
    var italic: Bool
    var underline: Bool
    var blink: Bool
    var inverse: Bool
    var strikethrough: Bool
    var fg: TerminalGridColor?
    var bg: TerminalGridColor?
}

struct TerminalGridColor: Equatable, Decodable {
    var r: UInt8
    var g: UInt8
    var b: UInt8
}

enum TerminalScreenKind: String, Equatable, Decodable {
    case primary
    case alternate
}

struct IntegrationReport: Equatable, Decodable {
    var terminalBackend: String
    var terminalTargetBackend: String
    var terminalBackendMigration: String
    var sshBackend: String
    var rendererBackend: String
    var rendererTargetBackend: String
    var ghosttyReady: Bool
    var libghosttyVtLinkConfigured: Bool
    var libghosttyVtReady: Bool
    var libghosttyVtAbiContract: String
    var libghosttyVtAbiStatus: String
    var russhReady: Bool
    var wgpuReady: Bool
    var rendererSurfaceReady: Bool

    static let fallback = IntegrationReport(
        terminalBackend: "unavailable",
        terminalTargetBackend: "libghostty-vt",
        terminalBackendMigration: "unavailable",
        sshBackend: "unavailable",
        rendererBackend: "unavailable",
        rendererTargetBackend: "wgpu-native-surface",
        ghosttyReady: false,
        libghosttyVtLinkConfigured: false,
        libghosttyVtReady: false,
        libghosttyVtAbiContract: "libghostty-vt-rs-0.2.0",
        libghosttyVtAbiStatus: "not-linked crate=libghostty-vt version=0.2.0",
        russhReady: false,
        wgpuReady: false,
        rendererSurfaceReady: false
    )

    private enum CodingKeys: String, CodingKey {
        case terminalBackend
        case terminalTargetBackend
        case terminalBackendMigration
        case sshBackend
        case rendererBackend
        case rendererTargetBackend
        case ghosttyReady
        case libghosttyVtLinkConfigured
        case libghosttyVtReady
        case libghosttyVtAbiContract
        case libghosttyVtAbiStatus
        case russhReady
        case wgpuReady
        case rendererSurfaceReady
    }

    init(
        terminalBackend: String,
        terminalTargetBackend: String,
        terminalBackendMigration: String,
        sshBackend: String,
        rendererBackend: String,
        rendererTargetBackend: String,
        ghosttyReady: Bool,
        libghosttyVtLinkConfigured: Bool,
        libghosttyVtReady: Bool,
        libghosttyVtAbiContract: String,
        libghosttyVtAbiStatus: String,
        russhReady: Bool,
        wgpuReady: Bool,
        rendererSurfaceReady: Bool
    ) {
        self.terminalBackend = terminalBackend
        self.terminalTargetBackend = terminalTargetBackend
        self.terminalBackendMigration = terminalBackendMigration
        self.sshBackend = sshBackend
        self.rendererBackend = rendererBackend
        self.rendererTargetBackend = rendererTargetBackend
        self.ghosttyReady = ghosttyReady
        self.libghosttyVtLinkConfigured = libghosttyVtLinkConfigured
        self.libghosttyVtReady = libghosttyVtReady
        self.libghosttyVtAbiContract = libghosttyVtAbiContract
        self.libghosttyVtAbiStatus = libghosttyVtAbiStatus
        self.russhReady = russhReady
        self.wgpuReady = wgpuReady
        self.rendererSurfaceReady = rendererSurfaceReady
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        terminalBackend = try container.decodeIfPresent(String.self, forKey: .terminalBackend) ?? "unavailable"
        terminalTargetBackend = try container.decodeIfPresent(String.self, forKey: .terminalTargetBackend) ?? "libghostty-vt"
        terminalBackendMigration = try container.decodeIfPresent(String.self, forKey: .terminalBackendMigration) ?? "unavailable"
        sshBackend = try container.decodeIfPresent(String.self, forKey: .sshBackend) ?? "unavailable"
        rendererBackend = try container.decodeIfPresent(String.self, forKey: .rendererBackend) ?? "unavailable"
        rendererTargetBackend = try container.decodeIfPresent(String.self, forKey: .rendererTargetBackend) ?? "wgpu-native-surface"
        ghosttyReady = try container.decodeIfPresent(Bool.self, forKey: .ghosttyReady) ?? false
        libghosttyVtLinkConfigured = try container.decodeIfPresent(Bool.self, forKey: .libghosttyVtLinkConfigured) ?? false
        libghosttyVtReady = try container.decodeIfPresent(Bool.self, forKey: .libghosttyVtReady) ?? false
        libghosttyVtAbiContract = try container.decodeIfPresent(String.self, forKey: .libghosttyVtAbiContract) ?? "libghostty-vt-rs-0.2.0"
        libghosttyVtAbiStatus = try container.decodeIfPresent(String.self, forKey: .libghosttyVtAbiStatus) ?? "not-linked crate=libghostty-vt version=0.2.0"
        russhReady = try container.decodeIfPresent(Bool.self, forKey: .russhReady) ?? false
        wgpuReady = try container.decodeIfPresent(Bool.self, forKey: .wgpuReady) ?? false
        rendererSurfaceReady = try container.decodeIfPresent(Bool.self, forKey: .rendererSurfaceReady) ?? false
    }
}

extension TerminalSession {
    static let preview = TerminalSession(
        title: "Shellow",
        host: "preview.local",
        state: .connected,
        observedHostKeySha256: nil,
        pendingClipboardText: nil,
        clipboardSequence: 0,
        bellCount: 0,
        rows: [
            TerminalRow(prompt: "$", text: "ssh deploy@staging", style: .command),
            TerminalRow(prompt: "", text: "Shellow preview terminal", style: .success),
            TerminalRow(prompt: "", text: "rust core: planned  |  renderer: planned  |  transport: planned", style: .muted),
            TerminalRow(prompt: "$", text: "uname -a", style: .command),
            TerminalRow(prompt: "", text: "Darwin shellow 26.4.1 arm64", style: .muted),
            TerminalRow(prompt: "$", text: "", style: .prompt)
        ],
        grid: nil,
        cursorColumn: 0,
        terminalCols: 80,
        terminalRows: 24,
        integration: .fallback
    )
}
