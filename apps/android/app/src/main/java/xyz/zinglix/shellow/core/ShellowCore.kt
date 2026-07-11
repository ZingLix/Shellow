package xyz.zinglix.shellow.core

import android.view.Surface
import java.io.Closeable
import java.util.UUID
import java.util.concurrent.locks.ReentrantLock
import kotlin.concurrent.withLock
import org.json.JSONArray
import org.json.JSONObject

private fun JSONObject.optNullableString(name: String): String? {
  if (!has(name) || isNull(name)) return null
  return optString(name).takeIf { it.isNotBlank() && it != "null" }
}

data class HostProfile(
  val name: String,
  val host: String,
  val port: Int,
  val username: String,
  val authentication: AuthenticationKind,
  val launchKind: ProfileLaunchKind = ProfileLaunchKind.Terminal,
  val trustedHostKeySha256: String? = null,
  val persistentTerminal: PersistentTerminalConfiguration? = null,
  val capabilityReport: RemoteHostCapabilityReport? = null,
  val id: String = UUID.randomUUID().toString(),
) {
  val endpoint: String = "$username@$host:$port"
  val hostKeyTrustTitle: String =
    if (trustedHostKeySha256.isNullOrBlank()) {
      "Host key unverified"
    } else {
      "Host key pinned"
    }

  val terminalStartupCommand: String
    get() {
      val configuration = persistentTerminal ?: return ""
      val backend = configuration.backend
      return "if command -v ${backend.executable} >/dev/null 2>&1; then ${backend.attachCommand(configuration.name)}; else echo 'Shellow: ${backend.displayTitle} is not installed; continuing with the regular shell.'; fi"
    }

  fun toJson() =
    JSONObject()
      .put("name", name)
      .put("host", host)
      .put("port", port)
      .put("username", username)
      .put("authentication", authentication.wire)
      .put("launchKind", launchKind.wire)
      .put("trustedHostKeySha256", trustedHostKeySha256.orEmpty())
      .put("persistentTerminal", persistentTerminal?.toJson())
      .put("capabilityReport", capabilityReport?.toJson())
      .put("id", id)

  companion object {
    fun fromJson(json: JSONObject): HostProfile {
      val name = json.optString("name")
      val host = json.optString("host")
      val port = json.optInt("port", 22)
      val username = json.optString("username")
      val authentication =
        if (json.optInt("authentication", AuthenticationKind.Password.wire) == AuthenticationKind.PrivateKey.wire) {
          AuthenticationKind.PrivateKey
        } else {
          AuthenticationKind.Password
        }

      return HostProfile(
        name = name,
        host = host,
        port = port,
        username = username,
        authentication = authentication,
        launchKind = ProfileLaunchKind.fromWire(json.optString("launchKind")),
        trustedHostKeySha256 = json.optNullableString("trustedHostKeySha256"),
        persistentTerminal =
          PersistentTerminalConfiguration.fromJson(
            json.optJSONObject("persistentTerminal") ?: json.optJSONObject("tmuxSession"),
          ),
        capabilityReport = RemoteHostCapabilityReport.fromJson(json.optJSONObject("capabilityReport")),
        id =
          json.optNullableString("id")
            ?: legacyProfileId(name, host, port, username, authentication),
      )
    }
  }
}

enum class ProfileLaunchKind(
  val wire: String,
  val title: String,
) {
  Terminal("terminal", "Terminal"),
  Codex("codex", "Codex");

  companion object {
    fun fromWire(value: String?): ProfileLaunchKind =
      entries.firstOrNull { it.wire == value } ?: Terminal
  }
}

private fun legacyProfileId(
  name: String,
  host: String,
  port: Int,
  username: String,
  authentication: AuthenticationKind,
): String =
  "legacy-" + listOf(name, host, port.toString(), username, authentication.wire.toString()).joinToString("|").hashCode().toUInt().toString(16)

enum class AuthenticationKind(val wire: Int, val title: String) {
  Password(0, "Password"),
  PrivateKey(1, "Private Key"),
}

data class TerminalSession(
  val title: String,
  val host: String,
  val state: ConnectionState,
  val observedHostKeySha256: String?,
  val pendingClipboardText: String?,
  val clipboardSequence: Long,
  val bellCount: Int,
  val rows: List<TerminalRow>,
  val grid: TerminalGridSnapshot?,
  val cursorColumn: Int,
  val terminalCols: Int,
  val terminalRows: Int,
  val integration: IntegrationReport,
) {
  companion object {
    fun fromJson(json: String): TerminalSession {
      val root = JSONObject(json)
      if (root.has("error")) {
        return bridgeFailure(root.optString("error", json))
      }

      return TerminalSession(
        title = root.getString("title"),
        host = root.getString("host"),
        state = ConnectionState.fromWire(root.getString("state")),
        observedHostKeySha256 = root.optNullableString("observed_host_key_sha256"),
        pendingClipboardText = root.optNullableString("pending_clipboard_text"),
        clipboardSequence = root.optLong("clipboard_sequence"),
        bellCount = root.optInt("bell_count"),
        rows = root.getJSONArray("rows").mapObjects(TerminalRow::fromJson),
        grid = root.optJSONObject("grid")?.let(TerminalGridSnapshot::fromJson),
        cursorColumn = root.optInt("cursor_column"),
        terminalCols = root.optInt("terminal_cols", 80),
        terminalRows = root.optInt("terminal_rows", 24),
        integration = IntegrationReport.fromJson(root.getJSONObject("integration")),
      )
    }

    fun bridgeFailure(message: String) =
      TerminalSession(
        title = "Shellow",
        host = "bridge.error",
        state = ConnectionState.Disconnected,
        observedHostKeySha256 = null,
        pendingClipboardText = null,
        clipboardSequence = 0,
        bellCount = 0,
        rows =
          listOf(
            TerminalRow("", "Shellow native bridge failed", TerminalRowStyle.Warning),
            TerminalRow("", message, TerminalRowStyle.Muted),
            TerminalRow("$", "", TerminalRowStyle.Prompt),
        ),
        grid = null,
        cursorColumn = 0,
        terminalCols = 80,
        terminalRows = 24,
        integration = IntegrationReport.fallback,
      )

    fun connecting(profile: HostProfile) =
      TerminalSession(
        title = profile.name,
        host = profile.endpoint,
        state = ConnectionState.Connecting,
        observedHostKeySha256 = null,
        pendingClipboardText = null,
        clipboardSequence = 0,
        bellCount = 0,
        rows =
          listOf(
            TerminalRow("$", "ssh ${profile.endpoint}", TerminalRowStyle.Command),
            TerminalRow("", "Connecting...", TerminalRowStyle.Muted),
            TerminalRow("$", "", TerminalRowStyle.Prompt),
          ),
        grid = null,
        cursorColumn = 0,
        terminalCols = 80,
        terminalRows = 24,
        integration = IntegrationReport.fallback,
      )
  }
}

data class CodexSnapshot(
  val title: String,
  val endpoint: String,
  val cwd: String?,
  val status: CodexStatus,
  val observedHostKeySha256: String?,
  val threadId: String?,
  val turnActive: Boolean,
  val messages: List<CodexMessage>,
  val pendingApprovals: List<CodexApproval>,
  val directory: CodexDirectoryState,
  val threads: CodexThreadListState,
  val projects: CodexProjectState,
  val threadDetail: CodexThreadDetailState,
  val activeTurn: CodexActiveTurn?,
  val operation: CodexOperationState,
  val settings: CodexSettingsState,
  val lastError: String?,
) {
  companion object {
    fun fromJson(json: String): CodexSnapshot {
      val root = JSONObject(json)
      if (root.has("error")) {
        return bridgeFailure(root.optString("error", json))
      }

      return CodexSnapshot(
        title = root.optString("title", "Codex"),
        endpoint = root.optString("endpoint", "not connected"),
        cwd = root.optNullableString("cwd"),
        status = CodexStatus.fromWire(root.optString("status")),
        observedHostKeySha256 = root.optNullableString("observed_host_key_sha256"),
        threadId = root.optNullableString("thread_id"),
        turnActive = root.optBoolean("turn_active"),
        messages = root.optJSONArray("messages")?.mapObjects(CodexMessage::fromJson).orEmpty(),
        pendingApprovals = root.optJSONArray("pending_approvals")?.mapObjects(CodexApproval::fromJson).orEmpty(),
        directory = root.optJSONObject("directory")?.let(CodexDirectoryState::fromJson) ?: CodexDirectoryState.Empty,
        threads = root.optJSONObject("threads")?.let(CodexThreadListState::fromJson) ?: CodexThreadListState.Empty,
        projects = root.optJSONObject("projects")?.let(CodexProjectState::fromJson) ?: CodexProjectState.Empty,
        threadDetail = root.optJSONObject("thread_detail")?.let(CodexThreadDetailState::fromJson) ?: CodexThreadDetailState.Empty,
        activeTurn = root.optJSONObject("active_turn")?.let(CodexActiveTurn::fromJson),
        operation = root.optJSONObject("operation")?.let(CodexOperationState::fromJson) ?: CodexOperationState.Idle,
        settings = root.optJSONObject("settings")?.let(CodexSettingsState::fromJson) ?: CodexSettingsState.Empty,
        lastError = root.optNullableString("last_error"),
      )
    }

    fun disconnected() =
      CodexSnapshot(
        title = "Codex",
        endpoint = "not connected",
        cwd = null,
        status = CodexStatus.Disconnected,
        observedHostKeySha256 = null,
        threadId = null,
        turnActive = false,
        messages = listOf(CodexMessage("status-0", CodexMessageRole.Status, "Connect to a host to start Codex.")),
        pendingApprovals = emptyList(),
        directory = CodexDirectoryState.Empty,
        threads = CodexThreadListState.Empty,
        projects = CodexProjectState.Empty,
        threadDetail = CodexThreadDetailState.Empty,
        activeTurn = null,
        operation = CodexOperationState.Idle,
        settings = CodexSettingsState.Empty,
        lastError = null,
      )

    fun connecting(profile: HostProfile, cwd: String) =
      CodexSnapshot(
        title = "Codex",
        endpoint = profile.endpoint,
        cwd = cwd.trim().takeIf { it.isNotEmpty() },
        status = CodexStatus.Connecting,
        observedHostKeySha256 = null,
        threadId = null,
        turnActive = false,
        messages = listOf(CodexMessage("status-0", CodexMessageRole.Status, "Starting Codex on ${profile.endpoint}.")),
        pendingApprovals = emptyList(),
        directory = CodexDirectoryState.Empty.copy(path = cwd.trim().takeIf { it.isNotEmpty() }),
        threads = CodexThreadListState.Empty,
        projects = CodexProjectState.Empty.copy(current = cwd.trim().takeIf { it.isNotEmpty() }),
        threadDetail = CodexThreadDetailState.Empty,
        activeTurn = null,
        operation = CodexOperationState.Idle,
        settings = CodexSettingsState.Empty,
        lastError = null,
      )

    fun bridgeFailure(message: String) =
      CodexSnapshot(
        title = "Codex",
        endpoint = "bridge.error",
        cwd = null,
        status = CodexStatus.Failed,
        observedHostKeySha256 = null,
        threadId = null,
        turnActive = false,
        messages =
          listOf(
            CodexMessage("status-0", CodexMessageRole.Status, "Codex native bridge failed"),
            CodexMessage("status-1", CodexMessageRole.Status, message),
          ),
        pendingApprovals = emptyList(),
        directory = CodexDirectoryState.Empty,
        threads = CodexThreadListState.Empty,
        projects = CodexProjectState.Empty,
        threadDetail = CodexThreadDetailState.Empty,
        activeTurn = null,
        operation = CodexOperationState.failure(message),
        settings = CodexSettingsState.Empty,
        lastError = message,
      )
  }
}

data class CodexProjectState(
  val current: String?,
  val remoteHome: String?,
  val recent: List<String>,
  val favorites: List<String>,
) {
  companion object {
    val Empty = CodexProjectState(null, null, emptyList(), emptyList())

    fun fromJson(json: JSONObject) =
      CodexProjectState(
        current = json.optNullableString("current"),
        remoteHome = json.optNullableString("remote_home"),
        recent = json.optJSONArray("recent")?.mapStrings().orEmpty().filter { it.isNotBlank() },
        favorites = json.optJSONArray("favorites")?.mapStrings().orEmpty().filter { it.isNotBlank() },
      )
  }
}

data class CodexDirectoryState(
  val path: String?,
  val parent: String?,
  val entries: List<CodexDirectoryEntry>,
  val isLoading: Boolean,
  val error: String?,
) {
  companion object {
    val Empty =
      CodexDirectoryState(
        path = null,
        parent = null,
        entries = emptyList(),
        isLoading = false,
        error = null,
      )

    fun fromJson(json: JSONObject) =
      CodexDirectoryState(
        path = json.optNullableString("path"),
        parent = json.optNullableString("parent"),
        entries = json.optJSONArray("entries")?.mapObjects(CodexDirectoryEntry::fromJson).orEmpty(),
        isLoading = json.optBoolean("is_loading"),
        error = json.optNullableString("error"),
      )
  }
}

data class CodexDirectoryEntry(
  val name: String,
  val path: String,
  val isDirectory: Boolean,
  val isFile: Boolean,
) {
  companion object {
    fun fromJson(json: JSONObject) =
      CodexDirectoryEntry(
        name = json.optString("name"),
        path = json.optString("path"),
        isDirectory = json.optBoolean("is_directory"),
        isFile = json.optBoolean("is_file"),
      )
  }
}

data class CodexThreadListState(
  val cwd: String?,
  val searchTerm: String?,
  val archived: Boolean,
  val threads: List<CodexThreadSummary>,
  val nextCursor: String?,
  val backwardsCursor: String?,
  val isLoading: Boolean,
  val isLoadingMore: Boolean,
  val error: String?,
) {
  companion object {
    val Empty =
      CodexThreadListState(
        cwd = null,
        searchTerm = null,
        archived = false,
        threads = emptyList(),
        nextCursor = null,
        backwardsCursor = null,
        isLoading = false,
        isLoadingMore = false,
        error = null,
      )

    fun fromJson(json: JSONObject) =
      CodexThreadListState(
        cwd = json.optNullableString("cwd"),
        searchTerm = json.optNullableString("search_term"),
        archived = json.optBoolean("archived"),
        threads = json.optJSONArray("threads")?.mapObjects(CodexThreadSummary::fromJson).orEmpty(),
        nextCursor = json.optNullableString("next_cursor"),
        backwardsCursor = json.optNullableString("backwards_cursor"),
        isLoading = json.optBoolean("is_loading"),
        isLoadingMore = json.optBoolean("is_loading_more"),
        error = json.optNullableString("error"),
      )
  }
}

data class CodexThreadSummary(
  val id: String,
  val name: String?,
  val preview: String,
  val cwd: String,
  val status: String,
  val activeFlags: List<String>,
  val pendingApprovalCount: Int,
  val lastTurnStatus: String?,
  val lastTurnError: String?,
  val updatedAt: Long,
  val createdAt: Long,
  val source: String,
  val modelProvider: String,
  val forkedFromId: String?,
  val parentThreadId: String?,
) {
  companion object {
    fun fromJson(json: JSONObject) =
      CodexThreadSummary(
        id = json.optString("id"),
        name = json.optNullableString("name"),
        preview = json.optString("preview"),
        cwd = json.optString("cwd"),
        status = json.optString("status"),
        activeFlags = json.optJSONArray("active_flags")?.mapStrings().orEmpty(),
        pendingApprovalCount = json.optInt("pending_approval_count"),
        lastTurnStatus = json.optNullableString("last_turn_status"),
        lastTurnError = json.optNullableString("last_turn_error"),
        updatedAt = json.optLong("updated_at"),
        createdAt = json.optLong("created_at"),
        source = json.optString("source"),
        modelProvider = json.optString("model_provider"),
        forkedFromId = json.optNullableString("forked_from_id"),
        parentThreadId = json.optNullableString("parent_thread_id"),
      )
  }
}

data class CodexThreadDetailState(
  val thread: CodexThreadSummary?,
  val turnsNextCursor: String?,
  val turnsBackwardsCursor: String?,
  val isLoading: Boolean,
  val isLoadingMore: Boolean,
  val error: String?,
) {
  companion object {
    val Empty = CodexThreadDetailState(null, null, null, false, false, null)

    fun fromJson(json: JSONObject) =
      CodexThreadDetailState(
        thread = json.optJSONObject("thread")?.let(CodexThreadSummary::fromJson),
        turnsNextCursor = json.optNullableString("turns_next_cursor"),
        turnsBackwardsCursor = json.optNullableString("turns_backwards_cursor"),
        isLoading = json.optBoolean("is_loading"),
        isLoadingMore = json.optBoolean("is_loading_more"),
        error = json.optNullableString("error"),
      )
  }
}

data class CodexActiveTurn(
  val id: String,
  val status: String,
) {
  companion object {
    fun fromJson(json: JSONObject) =
      CodexActiveTurn(
        id = json.optString("id"),
        status = json.optString("status"),
      )
  }
}

data class CodexOperationState(
  val isRunning: Boolean,
  val label: String?,
  val lastSuccess: String?,
  val lastError: String?,
) {
  companion object {
    val Idle = CodexOperationState(false, null, null, null)

    fun failure(message: String) = CodexOperationState(false, null, null, message)

    fun fromJson(json: JSONObject) =
      CodexOperationState(
        isRunning = json.optBoolean("is_running"),
        label = json.optNullableString("label"),
        lastSuccess = json.optNullableString("last_success"),
        lastError = json.optNullableString("last_error"),
      )
  }
}

data class CodexSettingsState(
  val model: String?,
  val reasoningEffort: String?,
  val serviceTier: String?,
  val approvalPolicy: String?,
  val sandbox: String?,
  val availableModels: List<CodexModelOption>,
  val isLoadingModels: Boolean,
  val modelsError: String?,
) {
  companion object {
    val Empty = CodexSettingsState(null, null, null, null, null, emptyList(), false, null)

    fun fromJson(json: JSONObject) =
      CodexSettingsState(
        model = json.optNullableString("model"),
        reasoningEffort = json.optNullableString("reasoning_effort"),
        serviceTier = json.optNullableString("service_tier"),
        approvalPolicy = json.optNullableString("approval_policy"),
        sandbox = json.optNullableString("sandbox"),
        availableModels = json.optJSONArray("available_models")?.mapObjects(CodexModelOption::fromJson).orEmpty(),
        isLoadingModels = json.optBoolean("is_loading_models"),
        modelsError = json.optNullableString("models_error"),
      )
  }
}

data class CodexModelOption(
  val id: String,
  val name: String,
  val reasoningEfforts: List<CodexSettingOption> = emptyList(),
  val defaultReasoningEffort: String? = null,
  val serviceTiers: List<CodexSettingOption> = emptyList(),
  val defaultServiceTier: String? = null,
) {
  companion object {
    fun fromJson(json: JSONObject) =
      CodexModelOption(
        id = json.optString("id"),
        name = json.optString("name"),
        reasoningEfforts = json.optJSONArray("reasoning_efforts")?.mapObjects(CodexSettingOption::fromJson).orEmpty(),
        defaultReasoningEffort = json.optNullableString("default_reasoning_effort"),
        serviceTiers = json.optJSONArray("service_tiers")?.mapObjects(CodexSettingOption::fromJson).orEmpty(),
        defaultServiceTier = json.optNullableString("default_service_tier"),
      )
  }
}

data class CodexSettingOption(
  val id: String,
  val name: String,
  val description: String?,
) {
  companion object {
    fun fromJson(json: JSONObject) =
      CodexSettingOption(
        id = json.optString("id"),
        name = json.optString("name"),
        description = json.optNullableString("description"),
      )
  }
}

enum class CodexStatus(val wire: String, val title: String) {
  Disconnected("disconnected", "Offline"),
  Connecting("connecting", "Connecting"),
  Connected("connected", "Connected"),
  Failed("failed", "Failed");

  companion object {
    fun fromWire(value: String) = entries.firstOrNull { it.wire == value } ?: Disconnected
  }
}

data class CodexMessage(
  val id: String,
  val role: CodexMessageRole,
  val text: String,
  val kind: CodexMessageKind = CodexMessageKind.Status,
  val visibility: CodexMessageVisibility = CodexMessageVisibility.Primary,
  val title: String? = null,
  val detail: String? = null,
  val transcript: String? = null,
  val format: CodexMessageFormat = CodexMessageFormat.Plain,
  val blocks: List<CodexMarkdownBlock> = emptyList(),
  val isStreaming: Boolean = false,
  val truncated: Boolean = false,
) {
  companion object {
    fun fromJson(json: JSONObject) =
      CodexMessage(
        id = json.optString("id"),
        role = CodexMessageRole.fromWire(json.optString("role")),
        text = json.optString("text"),
        kind = CodexMessageKind.fromWire(json.optString("kind")),
        visibility = CodexMessageVisibility.fromWire(json.optString("visibility")),
        title = json.optNullableString("title"),
        detail = json.optNullableString("detail"),
        transcript = json.optNullableString("transcript"),
        format = CodexMessageFormat.fromWire(json.optString("format")),
        blocks = json.optJSONArray("blocks")?.mapObjects(CodexMarkdownBlock::fromJson).orEmpty(),
        isStreaming = json.optBoolean("is_streaming"),
        truncated = json.optBoolean("truncated"),
      )
  }
}

enum class CodexMessageKind(val wire: String) {
  UserMessage("user_message"),
  FinalAnswer("final_answer"),
  Commentary("commentary"),
  ReasoningSummary("reasoning_summary"),
  Status("status"),
  ToolCall("tool_call"),
  ToolResult("tool_result"),
  Command("command"),
  CommandOutput("command_output"),
  FileChange("file_change"),
  Plan("plan");

  companion object {
    fun fromWire(value: String) = entries.firstOrNull { it.wire == value } ?: Status
  }
}

enum class CodexMessageVisibility(val wire: String) {
  Primary("primary"),
  Compact("compact"),
  TranscriptOnly("transcript_only"),
  Hidden("hidden");

  companion object {
    fun fromWire(value: String) = entries.firstOrNull { it.wire == value } ?: Primary
  }
}

enum class CodexMessageRole(val wire: String) {
  User("user"),
  Assistant("assistant"),
  Status("status"),
  Tool("tool"),
  CommandOutput("command_output");

  companion object {
    fun fromWire(value: String) = entries.firstOrNull { it.wire == value } ?: Status
  }
}

enum class CodexMessageFormat(val wire: String) {
  Plain("plain"),
  Markdown("markdown"),
  Code("code"),
  Status("status");

  companion object {
    fun fromWire(value: String) = entries.firstOrNull { it.wire == value } ?: Plain
  }
}

data class CodexMarkdownBlock(
  val id: String,
  val kind: CodexMarkdownBlockKind,
  val text: String,
  val imageUrl: String?,
  val imageAlt: String?,
  val level: Int?,
  val language: String?,
  val ordered: Boolean,
  val items: List<CodexMarkdownListItem>,
  val tableHeaders: List<CodexMarkdownTableCell>,
  val tableRows: List<List<CodexMarkdownTableCell>>,
  val runs: List<CodexMarkdownInlineRun>,
  val incomplete: Boolean,
) {
  companion object {
    fun fromJson(json: JSONObject) =
      CodexMarkdownBlock(
        id = json.optString("id"),
        kind = CodexMarkdownBlockKind.fromWire(json.optString("kind")),
        text = json.optString("text"),
        imageUrl = json.optNullableString("image_url"),
        imageAlt = json.optNullableString("image_alt"),
        level = json.optInt("level").takeIf { json.has("level") && !json.isNull("level") },
        language = json.optNullableString("language"),
        ordered = json.optBoolean("ordered"),
        items = json.optJSONArray("items")?.mapObjects(CodexMarkdownListItem::fromJson).orEmpty(),
        tableHeaders = json.optJSONArray("table_headers")?.mapObjects(CodexMarkdownTableCell::fromJson).orEmpty(),
        tableRows = json.optJSONArray("table_rows")?.mapTableRows().orEmpty(),
        runs = json.optJSONArray("runs")?.mapObjects(CodexMarkdownInlineRun::fromJson).orEmpty(),
        incomplete = json.optBoolean("incomplete"),
      )
  }
}

enum class CodexMarkdownBlockKind(val wire: String) {
  Paragraph("paragraph"),
  Heading("heading"),
  List("list"),
  BlockQuote("block_quote"),
  CodeBlock("code_block"),
  Table("table"),
  HorizontalRule("horizontal_rule"),
  Image("image");

  companion object {
    fun fromWire(value: String) = entries.firstOrNull { it.wire == value } ?: Paragraph
  }
}

data class CodexMarkdownListItem(
  val text: String,
  val runs: List<CodexMarkdownInlineRun>,
) {
  companion object {
    fun fromJson(json: JSONObject) =
      CodexMarkdownListItem(
        text = json.optString("text"),
        runs = json.optJSONArray("runs")?.mapObjects(CodexMarkdownInlineRun::fromJson).orEmpty(),
      )
  }
}

data class CodexMarkdownTableCell(
  val text: String,
  val runs: List<CodexMarkdownInlineRun>,
) {
  companion object {
    fun fromJson(json: JSONObject) =
      CodexMarkdownTableCell(
        text = json.optString("text"),
        runs = json.optJSONArray("runs")?.mapObjects(CodexMarkdownInlineRun::fromJson).orEmpty(),
      )
  }
}

private fun JSONArray.mapTableRows(): List<List<CodexMarkdownTableCell>> =
  buildList {
    for (index in 0 until length()) {
      optJSONArray(index)?.let { row ->
        add(row.mapObjects(CodexMarkdownTableCell::fromJson))
      }
    }
  }

data class CodexMarkdownInlineRun(
  val text: String,
  val style: CodexMarkdownInlineStyle,
  val url: String?,
) {
  companion object {
    fun fromJson(json: JSONObject) =
      CodexMarkdownInlineRun(
        text = json.optString("text"),
        style = CodexMarkdownInlineStyle.fromWire(json.optString("style")),
        url = json.optNullableString("url"),
      )
  }
}

enum class CodexMarkdownInlineStyle(val wire: String) {
  Text("text"),
  Bold("bold"),
  Italic("italic"),
  BoldItalic("bold_italic"),
  Code("code"),
  Link("link");

  companion object {
    fun fromWire(value: String) = entries.firstOrNull { it.wire == value } ?: Text
  }
}

data class CodexApproval(
  val requestId: String,
  val kind: CodexApprovalKind,
  val title: String,
  val detail: String,
  val command: String?,
  val cwd: String?,
  val reason: String?,
) {
  companion object {
    fun fromJson(json: JSONObject) =
      CodexApproval(
        requestId = json.optString("request_id"),
        kind = CodexApprovalKind.fromWire(json.optString("kind")),
        title = json.optString("title"),
        detail = json.optString("detail"),
        command = json.optNullableString("command"),
        cwd = json.optNullableString("cwd"),
        reason = json.optNullableString("reason"),
      )
  }
}

enum class CodexApprovalKind(val wire: String) {
  Command("command"),
  FileChange("file_change"),
  UserInput("user_input"),
  Permissions("permissions"),
  Tool("tool");

  companion object {
    fun fromWire(value: String) = entries.firstOrNull { it.wire == value } ?: Tool
  }
}

enum class ConnectionState(val wire: String, val title: String) {
  Disconnected("disconnected", "Offline"),
  Connecting("connecting", "Connecting"),
  Connected("connected", "Connected");

  companion object {
    fun fromWire(value: String) = entries.firstOrNull { it.wire == value } ?: Disconnected
  }
}

data class TerminalRow(
  val prompt: String,
  val text: String,
  val style: TerminalRowStyle,
) {
  companion object {
    fun fromJson(json: JSONObject) =
      TerminalRow(
        prompt = json.optString("prompt"),
        text = json.optString("text"),
        style = TerminalRowStyle.fromWire(json.optString("style")),
      )
  }
}

enum class TerminalRowStyle(val wire: String) {
  Command("command"),
  Muted("muted"),
  Success("success"),
  Prompt("prompt"),
  Warning("warning");

  companion object {
    fun fromWire(value: String) = entries.firstOrNull { it.wire == value } ?: Muted
  }
}

data class TerminalGridSnapshot(
  val cols: Int,
  val rows: Int,
  val cursorColumn: Int,
  val cursorRow: Int,
  val cursorVisible: Boolean,
  val cursorShape: TerminalCursorShape,
  val activeScreen: TerminalScreenKind,
  val scrollbackLen: Int,
  val bracketedPaste: Boolean,
  val applicationCursorKeys: Boolean,
  val mouseReporting: Boolean,
  val mouseDragReporting: Boolean,
  val sgrMouse: Boolean,
  val lines: List<String>,
  val styledLines: List<TerminalGridLine>,
  val dirtyRows: List<Int>,
) {
  val hasVisibleContent: Boolean
    get() = lines.any { it.trimEnd().isNotEmpty() }

  companion object {
    fun fromJson(json: JSONObject) =
      TerminalGridSnapshot(
        cols = json.optInt("cols", 80),
        rows = json.optInt("rows", 24),
        cursorColumn = json.optInt("cursor_column"),
        cursorRow = json.optInt("cursor_row"),
        cursorVisible = json.optBoolean("cursor_visible", true),
        cursorShape = TerminalCursorShape.fromWire(json.optString("cursor_shape")),
        activeScreen = TerminalScreenKind.fromWire(json.optString("active_screen")),
        scrollbackLen = json.optInt("scrollback_len"),
        bracketedPaste = json.optBoolean("bracketed_paste"),
        applicationCursorKeys = json.optBoolean("application_cursor_keys"),
        mouseReporting = json.optBoolean("mouse_reporting"),
        mouseDragReporting = json.optBoolean("mouse_drag_reporting"),
        sgrMouse = json.optBoolean("sgr_mouse"),
        lines = json.optJSONArray("lines")?.mapStrings().orEmpty(),
        styledLines = json.optJSONArray("styled_lines")?.mapObjects(TerminalGridLine::fromJson).orEmpty(),
        dirtyRows = json.optJSONArray("dirty_rows")?.mapInts().orEmpty(),
      )
  }
}

enum class TerminalCursorShape(val wire: String) {
  Block("block"),
  Underline("underline"),
  Bar("bar");

  companion object {
    fun fromWire(value: String) = entries.firstOrNull { it.wire == value } ?: Block
  }
}

data class TerminalGridLine(
  val runs: List<TerminalGridRun>,
) {
  companion object {
    fun fromJson(json: JSONObject) =
      TerminalGridLine(
        runs = json.optJSONArray("runs")?.mapObjects(TerminalGridRun::fromJson).orEmpty(),
      )
  }
}

data class TerminalGridRun(
  val text: String,
  val style: TerminalGridStyle,
) {
  companion object {
    fun fromJson(json: JSONObject) =
      TerminalGridRun(
        text = json.optString("text"),
        style = json.optJSONObject("style")?.let(TerminalGridStyle::fromJson) ?: TerminalGridStyle.Plain,
      )
  }
}

data class TerminalGridStyle(
  val bold: Boolean,
  val faint: Boolean,
  val italic: Boolean,
  val underline: Boolean,
  val blink: Boolean,
  val inverse: Boolean,
  val strikethrough: Boolean,
  val fg: TerminalGridColor?,
  val bg: TerminalGridColor?,
) {
  companion object {
    val Plain =
      TerminalGridStyle(
        bold = false,
        faint = false,
        italic = false,
        underline = false,
        blink = false,
        inverse = false,
        strikethrough = false,
        fg = null,
        bg = null,
      )

    fun fromJson(json: JSONObject) =
      TerminalGridStyle(
        bold = json.optBoolean("bold"),
        faint = json.optBoolean("faint"),
        italic = json.optBoolean("italic"),
        underline = json.optBoolean("underline"),
        blink = json.optBoolean("blink"),
        inverse = json.optBoolean("inverse"),
        strikethrough = json.optBoolean("strikethrough"),
        fg = json.optJSONObject("fg")?.let(TerminalGridColor::fromJson),
        bg = json.optJSONObject("bg")?.let(TerminalGridColor::fromJson),
      )
  }
}

data class TerminalGridColor(
  val r: Int,
  val g: Int,
  val b: Int,
) {
  companion object {
    fun fromJson(json: JSONObject) =
      TerminalGridColor(
        r = json.optInt("r").coerceIn(0, 255),
        g = json.optInt("g").coerceIn(0, 255),
        b = json.optInt("b").coerceIn(0, 255),
      )
  }
}

enum class TerminalScreenKind(val wire: String) {
  Primary("primary"),
  Alternate("alternate");

  companion object {
    fun fromWire(value: String) = entries.firstOrNull { it.wire == value } ?: Primary
  }
}

data class IntegrationReport(
  val terminalBackend: String,
  val terminalTargetBackend: String,
  val terminalBackendMigration: String,
  val sshBackend: String,
  val rendererBackend: String,
  val rendererTargetBackend: String,
  val ghosttyReady: Boolean,
  val libGhosttyVtLinkConfigured: Boolean,
  val libGhosttyVtReady: Boolean,
  val libGhosttyVtAbiContract: String,
  val libGhosttyVtAbiStatus: String,
  val russhReady: Boolean,
  val wgpuReady: Boolean,
  val rendererSurfaceReady: Boolean,
) {
  companion object {
    val fallback =
      IntegrationReport(
        terminalBackend = "unavailable",
        terminalTargetBackend = "libghostty-vt",
        terminalBackendMigration = "unavailable",
        sshBackend = "unavailable",
        rendererBackend = "unavailable",
        rendererTargetBackend = "wgpu-native-surface",
        ghosttyReady = false,
        libGhosttyVtLinkConfigured = false,
        libGhosttyVtReady = false,
        libGhosttyVtAbiContract = "libghostty-vt-rs-0.2.0",
        libGhosttyVtAbiStatus = "not-linked crate=libghostty-vt version=0.2.0",
        russhReady = false,
        wgpuReady = false,
        rendererSurfaceReady = false,
      )

    fun fromJson(json: JSONObject) =
      IntegrationReport(
        terminalBackend = json.optString("terminal_backend"),
        terminalTargetBackend = json.optString("terminal_target_backend", "libghostty-vt"),
        terminalBackendMigration = json.optString("terminal_backend_migration"),
        sshBackend = json.optString("ssh_backend"),
        rendererBackend = json.optString("renderer_backend"),
        rendererTargetBackend = json.optString("renderer_target_backend", "wgpu-native-surface"),
        ghosttyReady = json.optBoolean("ghostty_ready"),
        libGhosttyVtLinkConfigured = json.optBoolean("libghostty_vt_link_configured"),
        libGhosttyVtReady = json.optBoolean("libghostty_vt_ready"),
        libGhosttyVtAbiContract =
          json.optString("libghostty_vt_abi_contract", "libghostty-vt-rs-0.2.0"),
        libGhosttyVtAbiStatus =
          json.optString(
            "libghostty_vt_abi_status",
            "not-linked crate=libghostty-vt version=0.2.0",
          ),
        russhReady = json.optBoolean("russh_ready"),
        wgpuReady = json.optBoolean("wgpu_ready"),
        rendererSurfaceReady = json.optBoolean("renderer_surface_ready"),
      )
  }
}

class ShellowCoreSession : Closeable {
  private val lock = ReentrantLock()
  private var initFailure: String? = null
  private var handle =
    try {
      ShellowNative.nativeCreate().also { created ->
        if (created == 0L) {
          initFailure = "native engine creation returned null"
        }
      }
    } catch (error: Throwable) {
      initFailure = "native bridge init failed: ${error.message ?: error.toString()}"
      0L
    }

  fun snapshot() = decode { current -> ShellowNative.nativeSnapshotJson(current) }

  fun renderFrameJson(widthPx: Int, heightPx: Int): String =
    nativeJson { current -> ShellowNative.nativeRenderFrameJson(current, widthPx, heightPx) }

  fun renderFrameViewportJson(
    widthPx: Int,
    heightPx: Int,
    firstRow: Int,
    rowCount: Int,
  ): String =
    nativeJson { current ->
      ShellowNative.nativeRenderFrameViewportJson(current, widthPx, heightPx, firstRow, rowCount)
    }

  fun rendererInfoJson(): String =
    nativeJson { current -> ShellowNative.nativeRendererInfoJson(current) }

  fun setRendererOverlayJson(overlayJson: String): String =
    nativeJson { current -> ShellowNative.nativeSetRendererOverlayJson(current, overlayJson) }

  fun setTerminalTheme(themeId: String): String =
    nativeJson { current -> ShellowNative.nativeSetTerminalThemeJson(current, themeId) }

  fun attachAndroidNativeWindow(
    rawHandle: Long,
    widthPx: Int,
    heightPx: Int,
  ): String =
    nativeJson { current ->
      ShellowNative.nativeAttachAndroidNativeWindowJson(current, rawHandle, widthPx, heightPx)
    }

  fun attachAndroidSurface(
    surface: Surface,
    widthPx: Int,
    heightPx: Int,
  ): String =
    nativeJson { current ->
      ShellowNative.nativeAttachAndroidSurfaceJson(current, surface, widthPx, heightPx)
    }

  fun renderRendererSurfaceFrame(
    widthPx: Int,
    heightPx: Int,
    firstRow: Int,
    rowCount: Int,
  ): Boolean =
    lock.withLock {
      val current = handle
      current != 0L && ShellowNative.nativeRenderSurfaceFramePresented(current, widthPx, heightPx, firstRow, rowCount)
    }

  fun liveShellEventRevision(): Long =
    lock.withLock {
      val current = handle
      if (current == 0L) 0L else ShellowNative.nativeLiveShellEventRevision(current)
    }

  fun codexEventRevision(): Long =
    lock.withLock {
      val current = handle
      if (current == 0L) 0L else ShellowNative.nativeCodexEventRevision(current)
    }

  fun detachRendererSurface(): String =
    nativeJson { current -> ShellowNative.nativeDetachRendererSurfaceJson(current) }

  fun sendCommand(command: String) =
    decode { current -> ShellowNative.nativeSendCommandJson(current, command) }

  fun sendTerminalInput(input: String) =
    decode { current -> ShellowNative.nativeSendTerminalInputJson(current, input) }

  fun resizeTerminal(cols: Int, rows: Int) =
    decode { current -> ShellowNative.nativeResizeTerminalJson(current, cols, rows) }

  fun clearTerminal() = decode { current -> ShellowNative.nativeClearTerminalJson(current) }

  fun resetTerminal() = decode { current -> ShellowNative.nativeResetTerminalJson(current) }

  fun connectPreview(profile: HostProfile) =
    decode { current ->
      ShellowNative.nativeConnectPreviewJson(
        current,
        profile.name,
        profile.host,
        profile.port,
        profile.username,
        profile.trustedHostKeySha256.orEmpty(),
        profile.authentication.wire,
      )
    }

  fun startPasswordShell(profile: HostProfile, password: String) =
    decode { current ->
      ShellowNative.nativeStartPasswordShellJson(
        current,
        profile.name,
        profile.host,
        profile.port,
        profile.username,
        profile.trustedHostKeySha256.orEmpty(),
        password,
      )
    }

  fun startPrivateKeyShell(
    profile: HostProfile,
    privateKeyPem: String,
    passphrase: String,
  ) =
    decode { current ->
      ShellowNative.nativeStartPrivateKeyShellJson(
        current,
        profile.name,
        profile.host,
        profile.port,
        profile.username,
        profile.trustedHostKeySha256.orEmpty(),
        privateKeyPem,
        passphrase,
      )
    }

  fun connectPasswordExec(
    profile: HostProfile,
    password: String,
    command: String,
  ) =
    decode { current ->
      ShellowNative.nativeConnectPasswordExecJson(
        current,
        profile.name,
        profile.host,
        profile.port,
        profile.username,
        profile.trustedHostKeySha256.orEmpty(),
        password,
        command,
      )
    }

  fun connectPrivateKeyExec(
    profile: HostProfile,
    privateKeyPem: String,
    passphrase: String,
    command: String,
  ) =
    decode { current ->
      ShellowNative.nativeConnectPrivateKeyExecJson(
        current,
        profile.name,
        profile.host,
        profile.port,
        profile.username,
        profile.trustedHostKeySha256.orEmpty(),
        privateKeyPem,
        passphrase,
        command,
      )
    }

  fun pollLiveShell() = decode { current -> ShellowNative.nativePollLiveShellJson(current) }

  fun disconnectLiveShell() =
    decode { current -> ShellowNative.nativeDisconnectLiveShellJson(current) }

  fun codexSnapshot() =
    decodeCodex { current -> ShellowNative.nativeCodexSnapshotJson(current) }

  fun startCodexPassword(
    profile: HostProfile,
    password: String,
    cwd: String,
  ) =
    decodeCodex { current ->
      ShellowNative.nativeStartCodexPasswordJson(
        current,
        profile.name,
        profile.host,
        profile.port,
        profile.username,
        profile.trustedHostKeySha256.orEmpty(),
        password,
        cwd,
      )
    }

  fun startCodexPrivateKey(
    profile: HostProfile,
    privateKeyPem: String,
    passphrase: String,
    cwd: String,
  ) =
    decodeCodex { current ->
      ShellowNative.nativeStartCodexPrivateKeyJson(
        current,
        profile.name,
        profile.host,
        profile.port,
        profile.username,
        profile.trustedHostKeySha256.orEmpty(),
        privateKeyPem,
        passphrase,
        cwd,
      )
    }

  fun pollCodex() =
    decodeCodex { current -> ShellowNative.nativePollCodexJson(current) }

  fun sendCodexMessage(message: String) =
    decodeCodex { current -> ShellowNative.nativeSendCodexMessageJson(current, message) }

  fun updateCodexSettings(
    model: String,
    reasoningEffort: String,
    serviceTier: String,
    approvalPolicy: String,
    sandbox: String,
  ) =
    decodeCodex {
      current -> ShellowNative.nativeUpdateCodexSettingsJson(
        current,
        model,
        reasoningEffort,
        serviceTier,
        approvalPolicy,
        sandbox,
      )
    }

  fun browseCodexDirectory(path: String) =
    decodeCodex { current -> ShellowNative.nativeBrowseCodexDirectoryJson(current, path) }

  fun listCodexThreads(
    cwd: String,
    searchTerm: String,
  ) =
    decodeCodex { current -> ShellowNative.nativeListCodexThreadsJson(current, cwd, searchTerm) }

  fun listCodexThreadsPage(
    cwd: String,
    searchTerm: String,
    cursor: String,
    archived: Boolean,
    append: Boolean,
  ) =
    decodeCodex { current ->
      ShellowNative.nativeListCodexThreadsPageJson(current, cwd, searchTerm, cursor, archived, append)
    }

  fun startCodexThread(cwd: String) =
    decodeCodex { current -> ShellowNative.nativeStartCodexThreadJson(current, cwd) }

  fun resumeCodexThread(threadId: String) =
    decodeCodex { current -> ShellowNative.nativeResumeCodexThreadJson(current, threadId) }

  fun readCodexThread(threadId: String) =
    decodeCodex { current -> ShellowNative.nativeReadCodexThreadJson(current, threadId) }

  fun loadMoreCodexThreadTurns(
    threadId: String,
    cursor: String,
  ) =
    decodeCodex { current -> ShellowNative.nativeLoadMoreCodexThreadTurnsJson(current, threadId, cursor) }

  fun renameCodexThread(
    threadId: String,
    name: String,
  ) =
    decodeCodex { current -> ShellowNative.nativeRenameCodexThreadJson(current, threadId, name) }

  fun archiveCodexThread(threadId: String) =
    decodeCodex { current -> ShellowNative.nativeArchiveCodexThreadJson(current, threadId) }

  fun unarchiveCodexThread(threadId: String) =
    decodeCodex { current -> ShellowNative.nativeUnarchiveCodexThreadJson(current, threadId) }

  fun deleteCodexThread(threadId: String) =
    decodeCodex { current -> ShellowNative.nativeDeleteCodexThreadJson(current, threadId) }

  fun forkCodexThread(
    threadId: String,
    cwd: String,
  ) =
    decodeCodex { current -> ShellowNative.nativeForkCodexThreadJson(current, threadId, cwd) }

  fun interruptCodexTurn() =
    decodeCodex { current -> ShellowNative.nativeInterruptCodexTurnJson(current) }

  fun answerCodexApproval(
    requestId: String,
    decision: String,
  ) =
    decodeCodex { current -> ShellowNative.nativeAnswerCodexApprovalJson(current, requestId, decision) }

  fun disconnectCodex() =
    decodeCodex { current -> ShellowNative.nativeDisconnectCodexJson(current) }

  override fun close() {
    lock.withLock {
      val current = handle
      if (current != 0L) {
        ShellowNative.nativeDestroy(current)
        handle = 0L
      }
    }
  }

  private inline fun nativeJson(crossinline body: (Long) -> String): String =
    lock.withLock {
      val current = handle
      if (current == 0L) {
        "{\"error\":${JSONObject.quote(initFailure ?: "native engine is not available")}}"
      } else {
        body(current)
      }
    }

  private inline fun decode(crossinline body: (Long) -> String): TerminalSession =
    decodeNative { nativeJson(body) }

  private inline fun decodeCodex(crossinline body: (Long) -> String): CodexSnapshot =
    decodeNativeCodex { nativeJson(body) }

  private fun decodeNative(body: () -> String): TerminalSession =
    try {
      TerminalSession.fromJson(body())
    } catch (error: Throwable) {
      TerminalSession.bridgeFailure(error.message ?: error.toString())
    }

  private fun decodeNativeCodex(body: () -> String): CodexSnapshot =
    try {
      CodexSnapshot.fromJson(body())
    } catch (error: Throwable) {
      CodexSnapshot.bridgeFailure(error.message ?: error.toString())
    }
}

internal object ShellowNative {
  init {
    System.loadLibrary("shellow_ffi")
    System.loadLibrary("shellow_jni")
  }

  external fun nativeCreate(): Long
  external fun nativeDestroy(handle: Long)
  external fun nativeSnapshotJson(handle: Long): String
  external fun nativeRenderFrameJson(handle: Long, widthPx: Int, heightPx: Int): String
  external fun nativeRenderFrameViewportJson(handle: Long, widthPx: Int, heightPx: Int, firstRow: Int, rowCount: Int): String
  external fun nativeRenderSurfaceFramePresented(handle: Long, widthPx: Int, heightPx: Int, firstRow: Int, rowCount: Int): Boolean
  external fun nativeRendererInfoJson(handle: Long): String
  external fun nativeLiveShellEventRevision(handle: Long): Long
  external fun nativeCodexEventRevision(handle: Long): Long
  external fun nativeSetRendererOverlayJson(handle: Long, overlayJson: String): String
  external fun nativeSetTerminalThemeJson(handle: Long, themeId: String): String
  external fun nativeAttachAndroidNativeWindowJson(handle: Long, rawHandle: Long, widthPx: Int, heightPx: Int): String
  external fun nativeAttachAndroidSurfaceJson(handle: Long, surface: Surface, widthPx: Int, heightPx: Int): String
  external fun nativeDetachRendererSurfaceJson(handle: Long): String
  external fun nativeSendCommandJson(handle: Long, input: String): String
  external fun nativeSendTerminalInputJson(handle: Long, input: String): String
  external fun nativeResizeTerminalJson(handle: Long, cols: Int, rows: Int): String
  external fun nativeClearTerminalJson(handle: Long): String
  external fun nativeResetTerminalJson(handle: Long): String
  external fun nativeConnectPreviewJson(
    handle: Long,
    name: String,
    host: String,
    port: Int,
    username: String,
    trustedHostKeySha256: String,
    authKind: Int,
  ): String

  external fun nativeStartPasswordShellJson(
    handle: Long,
    name: String,
    host: String,
    port: Int,
    username: String,
    trustedHostKeySha256: String,
    password: String,
  ): String

  external fun nativeConnectPasswordExecJson(
    handle: Long,
    name: String,
    host: String,
    port: Int,
    username: String,
    trustedHostKeySha256: String,
    password: String,
    command: String,
  ): String

  external fun nativeConnectPrivateKeyExecJson(
    handle: Long,
    name: String,
    host: String,
    port: Int,
    username: String,
    trustedHostKeySha256: String,
    privateKeyPem: String,
    passphrase: String,
    command: String,
  ): String

  external fun nativeStartPrivateKeyShellJson(
    handle: Long,
    name: String,
    host: String,
    port: Int,
    username: String,
    trustedHostKeySha256: String,
    privateKeyPem: String,
    passphrase: String,
  ): String

  external fun nativePollLiveShellJson(handle: Long): String
  external fun nativeDisconnectLiveShellJson(handle: Long): String
  external fun nativeCodexSnapshotJson(handle: Long): String

  external fun nativeStartCodexPasswordJson(
    handle: Long,
    name: String,
    host: String,
    port: Int,
    username: String,
    trustedHostKeySha256: String,
    password: String,
    cwd: String,
  ): String

  external fun nativeStartCodexPrivateKeyJson(
    handle: Long,
    name: String,
    host: String,
    port: Int,
    username: String,
    trustedHostKeySha256: String,
    privateKeyPem: String,
    passphrase: String,
    cwd: String,
  ): String

  external fun nativePollCodexJson(handle: Long): String
  external fun nativeSendCodexMessageJson(handle: Long, message: String): String
  external fun nativeUpdateCodexSettingsJson(handle: Long, model: String, reasoningEffort: String, serviceTier: String, approvalPolicy: String, sandbox: String): String
  external fun nativeBrowseCodexDirectoryJson(handle: Long, path: String): String
  external fun nativeListCodexThreadsJson(handle: Long, cwd: String, searchTerm: String): String
  external fun nativeListCodexThreadsPageJson(
    handle: Long,
    cwd: String,
    searchTerm: String,
    cursor: String,
    archived: Boolean,
    append: Boolean,
  ): String
  external fun nativeStartCodexThreadJson(handle: Long, cwd: String): String
  external fun nativeResumeCodexThreadJson(handle: Long, threadId: String): String
  external fun nativeReadCodexThreadJson(handle: Long, threadId: String): String
  external fun nativeLoadMoreCodexThreadTurnsJson(handle: Long, threadId: String, cursor: String): String
  external fun nativeRenameCodexThreadJson(handle: Long, threadId: String, name: String): String
  external fun nativeArchiveCodexThreadJson(handle: Long, threadId: String): String
  external fun nativeUnarchiveCodexThreadJson(handle: Long, threadId: String): String
  external fun nativeDeleteCodexThreadJson(handle: Long, threadId: String): String
  external fun nativeForkCodexThreadJson(handle: Long, threadId: String, cwd: String): String
  external fun nativeInterruptCodexTurnJson(handle: Long): String
  external fun nativeAnswerCodexApprovalJson(handle: Long, requestId: String, decision: String): String
  external fun nativeDisconnectCodexJson(handle: Long): String
}

private fun <T> JSONArray.mapObjects(transform: (JSONObject) -> T): List<T> =
  List(length()) { index -> transform(getJSONObject(index)) }

private fun JSONArray.mapStrings(): List<String> =
  List(length()) { index -> optString(index) }

private fun JSONArray.mapInts(): List<Int> =
  List(length()) { index -> optInt(index) }
