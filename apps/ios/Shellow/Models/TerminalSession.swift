import Foundation

struct TerminalSession: Equatable, Decodable {
    var title: String
    var host: String
    var state: ConnectionState
    var observedHostKeySha256: String?
    var pendingClipboardText: String?
    var clipboardSequence: Int
    var bellCount: Int
    var detectedRemotePorts: [Int]
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
            detectedRemotePorts: [],
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
            detectedRemotePorts: [],
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

struct CodexSnapshot: Equatable, Decodable {
    var title: String
    var endpoint: String
    var cwd: String?
    var status: CodexStatus
    var observedHostKeySha256: String?
    var threadId: String?
    var turnActive: Bool
    var messages: [CodexMessage]
    var messagesStartIndex: Int
    var messagesReplaceAll: Bool
    var pendingApprovals: [CodexApproval]
    var directory: CodexDirectoryState
    var threads: CodexThreadListState
    var projects: CodexProjectState
    var threadDetail: CodexThreadDetailState
    var activeTurn: CodexActiveTurn?
    var operation: CodexOperationState
    var settings: CodexSettingsState
    var usage: CodexUsageState
    var lastError: String?

    private enum CodingKeys: String, CodingKey {
        case title
        case endpoint
        case cwd
        case status
        case observedHostKeySha256
        case threadId
        case turnActive
        case messages
        case messagesStartIndex
        case messagesReplaceAll
        case pendingApprovals
        case directory
        case threads
        case projects
        case threadDetail
        case activeTurn
        case operation
        case settings
        case usage
        case lastError
    }

    init(
        title: String,
        endpoint: String,
        cwd: String?,
        status: CodexStatus,
        observedHostKeySha256: String?,
        threadId: String?,
        turnActive: Bool,
        messages: [CodexMessage],
        pendingApprovals: [CodexApproval],
        directory: CodexDirectoryState,
        threads: CodexThreadListState,
        projects: CodexProjectState = .empty,
        threadDetail: CodexThreadDetailState = .empty,
        activeTurn: CodexActiveTurn? = nil,
        operation: CodexOperationState = .idle,
        settings: CodexSettingsState = .empty,
        usage: CodexUsageState = .empty,
        lastError: String?,
        messagesStartIndex: Int = 0,
        messagesReplaceAll: Bool = true
    ) {
        self.title = title
        self.endpoint = endpoint
        self.cwd = cwd
        self.status = status
        self.observedHostKeySha256 = observedHostKeySha256
        self.threadId = threadId
        self.turnActive = turnActive
        self.messages = messages
        self.messagesStartIndex = messagesStartIndex
        self.messagesReplaceAll = messagesReplaceAll
        self.pendingApprovals = pendingApprovals
        self.directory = directory
        self.threads = threads
        self.projects = projects
        self.threadDetail = threadDetail
        self.activeTurn = activeTurn
        self.operation = operation
        self.settings = settings
        self.usage = usage
        self.lastError = lastError
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        title = try container.decodeIfPresent(String.self, forKey: .title) ?? "Codex"
        endpoint = try container.decodeIfPresent(String.self, forKey: .endpoint) ?? "not connected"
        cwd = try container.decodeIfPresent(String.self, forKey: .cwd)
        status = try container.decodeIfPresent(CodexStatus.self, forKey: .status) ?? .disconnected
        observedHostKeySha256 = try container.decodeIfPresent(String.self, forKey: .observedHostKeySha256)
        threadId = try container.decodeIfPresent(String.self, forKey: .threadId)
        turnActive = try container.decodeIfPresent(Bool.self, forKey: .turnActive) ?? false
        messages = try container.decodeIfPresent([CodexMessage].self, forKey: .messages) ?? []
        messagesStartIndex = try container.decodeIfPresent(Int.self, forKey: .messagesStartIndex) ?? 0
        messagesReplaceAll = try container.decodeIfPresent(Bool.self, forKey: .messagesReplaceAll) ?? true
        pendingApprovals = try container.decodeIfPresent([CodexApproval].self, forKey: .pendingApprovals) ?? []
        directory = try container.decodeIfPresent(CodexDirectoryState.self, forKey: .directory) ?? .empty
        threads = try container.decodeIfPresent(CodexThreadListState.self, forKey: .threads) ?? .empty
        projects = try container.decodeIfPresent(CodexProjectState.self, forKey: .projects) ?? .empty
        threadDetail = try container.decodeIfPresent(CodexThreadDetailState.self, forKey: .threadDetail) ?? .empty
        activeTurn = try container.decodeIfPresent(CodexActiveTurn.self, forKey: .activeTurn)
        operation = try container.decodeIfPresent(CodexOperationState.self, forKey: .operation) ?? .idle
        settings = try container.decodeIfPresent(CodexSettingsState.self, forKey: .settings) ?? .empty
        usage = try container.decodeIfPresent(CodexUsageState.self, forKey: .usage) ?? .empty
        lastError = try container.decodeIfPresent(String.self, forKey: .lastError)
    }

    static func disconnected() -> CodexSnapshot {
        CodexSnapshot(
            title: "Codex",
            endpoint: "not connected",
            cwd: nil,
            status: .disconnected,
            observedHostKeySha256: nil,
            threadId: nil,
            turnActive: false,
            messages: [
                CodexMessage(id: "status-0", role: .status, text: "Connect to a host to start Codex.")
            ],
            pendingApprovals: [],
            directory: .empty,
            threads: .empty,
            projects: .empty,
            threadDetail: .empty,
            activeTurn: nil,
            operation: .idle,
            settings: .empty,
            lastError: nil
        )
    }

    static func connecting(to profile: HostProfile, cwd: String) -> CodexSnapshot {
        let trimmedCwd = cwd.trimmingCharacters(in: .whitespacesAndNewlines)
        let resolvedCwd = trimmedCwd.isEmpty ? nil : trimmedCwd

        return CodexSnapshot(
            title: "Codex",
            endpoint: profile.endpoint,
            cwd: resolvedCwd,
            status: .connecting,
            observedHostKeySha256: nil,
            threadId: nil,
            turnActive: false,
            messages: [
                CodexMessage(id: "status-0", role: .status, text: "Starting Codex on \(profile.endpoint).")
            ],
            pendingApprovals: [],
            directory: .empty,
            threads: .empty,
            projects: CodexProjectState(current: resolvedCwd, remoteHome: nil, recent: [], favorites: []),
            threadDetail: .empty,
            activeTurn: nil,
            operation: .idle,
            settings: .empty,
            lastError: nil
        )
    }

    static func bridgeFailure(_ message: String) -> CodexSnapshot {
        CodexSnapshot(
            title: "Codex",
            endpoint: "bridge.error",
            cwd: nil,
            status: .failed,
            observedHostKeySha256: nil,
            threadId: nil,
            turnActive: false,
            messages: [
                CodexMessage(id: "status-0", role: .status, text: "Codex native bridge failed"),
                CodexMessage(id: "status-1", role: .status, text: message)
            ],
            pendingApprovals: [],
            directory: .empty,
            threads: .empty,
            projects: .empty,
            threadDetail: .empty,
            activeTurn: nil,
            operation: .failure(message),
            settings: .empty,
            lastError: message
        )
    }
}

struct CodexProjectState: Equatable, Decodable {
    var current: String?
    var remoteHome: String?
    var recent: [String]
    var favorites: [String]

    static let empty = CodexProjectState(current: nil, remoteHome: nil, recent: [], favorites: [])
}

struct CodexDirectoryState: Equatable, Decodable {
    var path: String?
    var parent: String?
    var entries: [CodexDirectoryEntry]
    var isLoading: Bool
    var error: String?

    static let empty = CodexDirectoryState(
        path: nil,
        parent: nil,
        entries: [],
        isLoading: false,
        error: nil
    )
}

struct CodexDirectoryEntry: Identifiable, Equatable, Decodable {
    var name: String
    var path: String
    var isDirectory: Bool
    var isFile: Bool

    var id: String { path }
}

struct CodexThreadListState: Equatable, Decodable {
    var cwd: String?
    var searchTerm: String?
    var archived: Bool
    var threads: [CodexThreadSummary]
    var nextCursor: String?
    var backwardsCursor: String?
    var isLoading: Bool
    var isLoadingMore: Bool
    var error: String?

    static let empty = CodexThreadListState(
        cwd: nil,
        searchTerm: nil,
        archived: false,
        threads: [],
        nextCursor: nil,
        backwardsCursor: nil,
        isLoading: false,
        isLoadingMore: false,
        error: nil
    )
}

struct CodexThreadSummary: Identifiable, Equatable, Decodable {
    var id: String
    var name: String?
    var preview: String
    var cwd: String
    var status: String
    var activeFlags: [String]
    var pendingApprovalCount: Int
    var lastTurnStatus: String?
    var lastTurnError: String?
    var updatedAt: UInt64
    var createdAt: UInt64
    var source: String
    var modelProvider: String
    var forkedFromId: String?
    var parentThreadId: String?
}

struct CodexThreadDetailState: Equatable, Decodable {
    var thread: CodexThreadSummary?
    var turnsNextCursor: String?
    var turnsBackwardsCursor: String?
    var isLoading: Bool
    var isLoadingMore: Bool
    var error: String?

    static let empty = CodexThreadDetailState(
        thread: nil,
        turnsNextCursor: nil,
        turnsBackwardsCursor: nil,
        isLoading: false,
        isLoadingMore: false,
        error: nil
    )
}

struct CodexActiveTurn: Equatable, Decodable {
    var id: String
    var status: String
}

struct CodexOperationState: Equatable, Decodable {
    var isRunning: Bool
    var label: String?
    var lastSuccess: String?
    var lastError: String?

    static let idle = CodexOperationState(isRunning: false, label: nil, lastSuccess: nil, lastError: nil)

    static func failure(_ message: String) -> CodexOperationState {
        CodexOperationState(isRunning: false, label: nil, lastSuccess: nil, lastError: message)
    }
}

struct CodexModelOption: Identifiable, Equatable, Decodable {
    var id: String
    var name: String
    var reasoningEfforts: [CodexSettingOption]
    var defaultReasoningEffort: String?
    var serviceTiers: [CodexSettingOption]
    var defaultServiceTier: String?

    init(
        id: String,
        name: String,
        reasoningEfforts: [CodexSettingOption] = [],
        defaultReasoningEffort: String? = nil,
        serviceTiers: [CodexSettingOption] = [],
        defaultServiceTier: String? = nil
    ) {
        self.id = id
        self.name = name
        self.reasoningEfforts = reasoningEfforts
        self.defaultReasoningEffort = defaultReasoningEffort
        self.serviceTiers = serviceTiers
        self.defaultServiceTier = defaultServiceTier
    }

    private enum CodingKeys: String, CodingKey {
        case id, name, reasoningEfforts, defaultReasoningEffort, serviceTiers, defaultServiceTier
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        id = try container.decode(String.self, forKey: .id)
        name = try container.decode(String.self, forKey: .name)
        reasoningEfforts = try container.decodeIfPresent([CodexSettingOption].self, forKey: .reasoningEfforts) ?? []
        defaultReasoningEffort = try container.decodeIfPresent(String.self, forKey: .defaultReasoningEffort)
        serviceTiers = try container.decodeIfPresent([CodexSettingOption].self, forKey: .serviceTiers) ?? []
        defaultServiceTier = try container.decodeIfPresent(String.self, forKey: .defaultServiceTier)
    }
}

struct CodexSettingOption: Identifiable, Equatable, Decodable {
    var id: String
    var name: String
    var description: String?
}

struct CodexSettingsState: Equatable, Decodable {
    var model: String?
    var reasoningEffort: String?
    var serviceTier: String?
    var approvalPolicy: String?
    var sandbox: String?
    var availableModels: [CodexModelOption]
    var isLoadingModels: Bool
    var modelsError: String?

    static let empty = CodexSettingsState(
        model: nil,
        reasoningEffort: nil,
        serviceTier: nil,
        approvalPolicy: nil,
        sandbox: nil,
        availableModels: [],
        isLoadingModels: false,
        modelsError: nil
    )

    init(
        model: String?,
        reasoningEffort: String?,
        serviceTier: String?,
        approvalPolicy: String?,
        sandbox: String?,
        availableModels: [CodexModelOption],
        isLoadingModels: Bool,
        modelsError: String?
    ) {
        self.model = model
        self.reasoningEffort = reasoningEffort
        self.serviceTier = serviceTier
        self.approvalPolicy = approvalPolicy
        self.sandbox = sandbox
        self.availableModels = availableModels
        self.isLoadingModels = isLoadingModels
        self.modelsError = modelsError
    }

    private enum CodingKeys: String, CodingKey {
        case model
        case reasoningEffort
        case serviceTier
        case approvalPolicy
        case sandbox
        case availableModels
        case isLoadingModels
        case modelsError
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        model = try container.decodeIfPresent(String.self, forKey: .model)
        reasoningEffort = try container.decodeIfPresent(String.self, forKey: .reasoningEffort)
        serviceTier = try container.decodeIfPresent(String.self, forKey: .serviceTier)
        approvalPolicy = try container.decodeIfPresent(String.self, forKey: .approvalPolicy)
        sandbox = try container.decodeIfPresent(String.self, forKey: .sandbox)
        availableModels = try container.decodeIfPresent([CodexModelOption].self, forKey: .availableModels) ?? []
        isLoadingModels = try container.decodeIfPresent(Bool.self, forKey: .isLoadingModels) ?? false
        modelsError = try container.decodeIfPresent(String.self, forKey: .modelsError)
    }
}

struct CodexUsageState: Equatable, Decodable {
    var thread: CodexThreadTokenUsage?
    var rateLimits: CodexRateLimitSnapshot?
    var isLoadingRateLimits: Bool
    var rateLimitsError: String?

    static let empty = CodexUsageState(
        thread: nil,
        rateLimits: nil,
        isLoadingRateLimits: false,
        rateLimitsError: nil
    )
}

struct CodexThreadTokenUsage: Equatable, Decodable {
    var last: CodexTokenUsageBreakdown
    var total: CodexTokenUsageBreakdown
    var modelContextWindow: UInt64?
}

struct CodexTokenUsageBreakdown: Equatable, Decodable {
    var cachedInputTokens: UInt64
    var inputTokens: UInt64
    var outputTokens: UInt64
    var reasoningOutputTokens: UInt64
    var totalTokens: UInt64
}

struct CodexRateLimitSnapshot: Equatable, Decodable {
    var limitId: String?
    var limitName: String?
    var planType: String?
    var primary: CodexRateLimitWindow?
    var secondary: CodexRateLimitWindow?
    var credits: CodexCreditsSnapshot?
    var individualLimit: CodexSpendControlLimitSnapshot?
    var rateLimitReachedType: String?
}

struct CodexRateLimitWindow: Equatable, Decodable {
    var usedPercent: UInt32
    var resetsAt: UInt64?
    var windowDurationMins: UInt64?
}

struct CodexCreditsSnapshot: Equatable, Decodable {
    var hasCredits: Bool
    var unlimited: Bool
    var balance: String?
}

struct CodexSpendControlLimitSnapshot: Equatable, Decodable {
    var limit: String
    var used: String
    var remainingPercent: UInt32
    var resetsAt: UInt64
}

enum CodexStatus: String, Equatable, Decodable {
    case disconnected
    case connecting
    case connected
    case failed

    var title: String {
        switch self {
        case .disconnected: "Offline"
        case .connecting: "Connecting"
        case .connected: "Connected"
        case .failed: "Failed"
        }
    }
}

struct CodexMessage: Identifiable, Equatable, Decodable {
    var id: String
    var role: CodexMessageRole
    var text: String
    var kind: CodexMessageKind
    var visibility: CodexMessageVisibility
    var title: String?
    var detail: String?
    var transcript: String?
    var format: CodexMessageFormat
    var blocks: [CodexMarkdownBlock]
    var isStreaming: Bool
    var truncated: Bool
    var delivery: CodexMessageDelivery?

    private enum CodingKeys: String, CodingKey {
        case id
        case role
        case text
        case kind
        case visibility
        case title
        case detail
        case transcript
        case format
        case blocks
        case isStreaming
        case truncated
        case delivery
    }

    init(
        id: String,
        role: CodexMessageRole,
        text: String,
        kind: CodexMessageKind = .status,
        visibility: CodexMessageVisibility = .primary,
        title: String? = nil,
        detail: String? = nil,
        transcript: String? = nil,
        format: CodexMessageFormat = .plain,
        blocks: [CodexMarkdownBlock] = [],
        isStreaming: Bool = false,
        truncated: Bool = false,
        delivery: CodexMessageDelivery? = nil
    ) {
        self.id = id
        self.role = role
        self.text = text
        self.kind = kind
        self.visibility = visibility
        self.title = title
        self.detail = detail
        self.transcript = transcript
        self.format = format
        self.blocks = blocks
        self.isStreaming = isStreaming
        self.truncated = truncated
        self.delivery = delivery
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        id = try container.decode(String.self, forKey: .id)
        role = try container.decode(CodexMessageRole.self, forKey: .role)
        text = try container.decodeIfPresent(String.self, forKey: .text) ?? ""
        kind = try container.decodeIfPresent(CodexMessageKind.self, forKey: .kind) ?? .status
        visibility = try container.decodeIfPresent(CodexMessageVisibility.self, forKey: .visibility) ?? .primary
        title = try container.decodeIfPresent(String.self, forKey: .title)
        detail = try container.decodeIfPresent(String.self, forKey: .detail)
        transcript = try container.decodeIfPresent(String.self, forKey: .transcript)
        format = try container.decodeIfPresent(CodexMessageFormat.self, forKey: .format) ?? .plain
        blocks = try container.decodeIfPresent([CodexMarkdownBlock].self, forKey: .blocks) ?? []
        isStreaming = try container.decodeIfPresent(Bool.self, forKey: .isStreaming) ?? false
        truncated = try container.decodeIfPresent(Bool.self, forKey: .truncated) ?? false
        delivery = try container.decodeIfPresent(CodexMessageDelivery.self, forKey: .delivery)
    }
}

enum CodexMessageDelivery: String, Equatable, Decodable {
    case queued
    case sent
    case committed
    case failed
}

enum CodexMessageKind: String, Equatable, Decodable {
    case userMessage = "user_message"
    case finalAnswer = "final_answer"
    case commentary
    case reasoningSummary = "reasoning_summary"
    case status
    case toolCall = "tool_call"
    case toolResult = "tool_result"
    case command
    case commandOutput = "command_output"
    case fileChange = "file_change"
    case plan
}

enum CodexMessageVisibility: String, Equatable, Decodable {
    case primary
    case compact
    case transcriptOnly = "transcript_only"
    case hidden
}

enum CodexMessageRole: String, Equatable, Decodable {
    case user
    case assistant
    case status
    case tool
    case commandOutput = "command_output"
}

enum CodexMessageFormat: String, Equatable, Decodable {
    case plain
    case markdown
    case code
    case status
}

struct CodexMarkdownBlock: Identifiable, Equatable, Decodable {
    var id: String
    var kind: CodexMarkdownBlockKind
    var text: String
    var imageUrl: String?
    var imageAlt: String?
    var level: Int?
    var language: String?
    var ordered: Bool
    var items: [CodexMarkdownListItem]
    var tableHeaders: [CodexMarkdownTableCell]
    var tableRows: [[CodexMarkdownTableCell]]
    var runs: [CodexMarkdownInlineRun]
    var incomplete: Bool

    private enum CodingKeys: String, CodingKey {
        case id
        case kind
        case text
        case imageUrl
        case imageAlt
        case level
        case language
        case ordered
        case items
        case tableHeaders
        case tableRows
        case runs
        case incomplete
    }

    init(
        id: String,
        kind: CodexMarkdownBlockKind,
        text: String,
        imageUrl: String? = nil,
        imageAlt: String? = nil,
        level: Int? = nil,
        language: String? = nil,
        ordered: Bool = false,
        items: [CodexMarkdownListItem] = [],
        tableHeaders: [CodexMarkdownTableCell] = [],
        tableRows: [[CodexMarkdownTableCell]] = [],
        runs: [CodexMarkdownInlineRun] = [],
        incomplete: Bool = false
    ) {
        self.id = id
        self.kind = kind
        self.text = text
        self.imageUrl = imageUrl
        self.imageAlt = imageAlt
        self.level = level
        self.language = language
        self.ordered = ordered
        self.items = items
        self.tableHeaders = tableHeaders
        self.tableRows = tableRows
        self.runs = runs
        self.incomplete = incomplete
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        id = try container.decode(String.self, forKey: .id)
        kind = try container.decode(CodexMarkdownBlockKind.self, forKey: .kind)
        text = try container.decodeIfPresent(String.self, forKey: .text) ?? ""
        imageUrl = try container.decodeIfPresent(String.self, forKey: .imageUrl)
        imageAlt = try container.decodeIfPresent(String.self, forKey: .imageAlt)
        level = try container.decodeIfPresent(Int.self, forKey: .level)
        language = try container.decodeIfPresent(String.self, forKey: .language)
        ordered = try container.decodeIfPresent(Bool.self, forKey: .ordered) ?? false
        items = try container.decodeIfPresent([CodexMarkdownListItem].self, forKey: .items) ?? []
        tableHeaders = try container.decodeIfPresent([CodexMarkdownTableCell].self, forKey: .tableHeaders) ?? []
        tableRows = try container.decodeIfPresent([[CodexMarkdownTableCell]].self, forKey: .tableRows) ?? []
        runs = try container.decodeIfPresent([CodexMarkdownInlineRun].self, forKey: .runs) ?? []
        incomplete = try container.decodeIfPresent(Bool.self, forKey: .incomplete) ?? false
    }
}

enum CodexMarkdownBlockKind: String, Equatable, Decodable {
    case paragraph
    case heading
    case list
    case blockQuote = "block_quote"
    case codeBlock = "code_block"
    case table
    case horizontalRule = "horizontal_rule"
    case image
}

struct CodexMarkdownListItem: Equatable, Decodable {
    var text: String
    var runs: [CodexMarkdownInlineRun]
}

struct CodexMarkdownTableCell: Equatable, Decodable {
    var text: String
    var runs: [CodexMarkdownInlineRun]
}

struct CodexMarkdownInlineRun: Equatable, Decodable {
    var text: String
    var style: CodexMarkdownInlineStyle
    var url: String?
}

enum CodexMarkdownInlineStyle: String, Equatable, Decodable {
    case text
    case bold
    case italic
    case boldItalic = "bold_italic"
    case code
    case link
}

struct CodexApproval: Identifiable, Equatable, Decodable {
    var requestId: String
    var kind: CodexApprovalKind
    var title: String
    var detail: String
    var command: String?
    var cwd: String?
    var reason: String?
    var questions: [CodexUserInputQuestion]
    var availableDecisions: [String]
    var permissions: String?

    var id: String { requestId }
}

struct CodexUserInputQuestion: Equatable, Decodable, Identifiable {
    var id: String
    var header: String
    var question: String
    var isOther: Bool
    var isSecret: Bool
    var multiSelect: Bool
    var options: [CodexUserInputOption]
}

struct CodexUserInputOption: Equatable, Decodable, Identifiable {
    var label: String
    var description: String
    var preview: String?

    var id: String { label }
}

enum CodexApprovalKind: String, Equatable, Decodable {
    case command
    case fileChange = "file_change"
    case userInput = "user_input"
    case permissions
    case elicitation
    case tool
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
        detectedRemotePorts: [],
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
