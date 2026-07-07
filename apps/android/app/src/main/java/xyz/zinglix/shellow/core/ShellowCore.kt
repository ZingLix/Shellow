package xyz.zinglix.shellow.core

import android.view.Surface
import java.io.Closeable
import java.util.UUID
import org.json.JSONArray
import org.json.JSONObject

data class HostProfile(
  val name: String,
  val host: String,
  val port: Int,
  val username: String,
  val authentication: AuthenticationKind,
  val trustedHostKeySha256: String? = null,
  val id: String = UUID.randomUUID().toString(),
) {
  val endpoint: String = "$username@$host:$port"
  val hostKeyTrustTitle: String =
    if (trustedHostKeySha256.isNullOrBlank()) {
      "Host key unverified"
    } else {
      "Host key pinned"
    }

  fun toJson() =
    JSONObject()
      .put("name", name)
      .put("host", host)
      .put("port", port)
      .put("username", username)
      .put("authentication", authentication.wire)
      .put("trustedHostKeySha256", trustedHostKeySha256.orEmpty())
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
        trustedHostKeySha256 = json.optString("trustedHostKeySha256").takeIf { it.isNotBlank() },
        id =
          json
            .optString("id")
            .takeIf { it.isNotBlank() }
            ?: legacyProfileId(name, host, port, username, authentication),
      )
    }
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
        observedHostKeySha256 = root.optString("observed_host_key_sha256").takeIf { it.isNotBlank() },
        pendingClipboardText = root.optString("pending_clipboard_text").takeIf { it.isNotBlank() },
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
            TerminalRow("", "waiting for russh password authentication", TerminalRowStyle.Muted),
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

  fun snapshot() = decode { ShellowNative.nativeSnapshotJson(handle) }

  fun renderFrameJson(widthPx: Int, heightPx: Int): String =
    if (handle == 0L) {
      "{\"error\":${JSONObject.quote(initFailure ?: "native engine is not available")}}"
    } else {
      ShellowNative.nativeRenderFrameJson(handle, widthPx, heightPx)
    }

  fun renderFrameViewportJson(
    widthPx: Int,
    heightPx: Int,
    firstRow: Int,
    rowCount: Int,
  ): String =
    if (handle == 0L) {
      "{\"error\":${JSONObject.quote(initFailure ?: "native engine is not available")}}"
    } else {
      ShellowNative.nativeRenderFrameViewportJson(handle, widthPx, heightPx, firstRow, rowCount)
    }

  fun rendererInfoJson(): String =
    if (handle == 0L) {
      "{\"error\":${JSONObject.quote(initFailure ?: "native engine is not available")}}"
    } else {
      ShellowNative.nativeRendererInfoJson(handle)
    }

  fun setRendererOverlayJson(overlayJson: String): String =
    if (handle == 0L) {
      "{\"error\":${JSONObject.quote(initFailure ?: "native engine is not available")}}"
    } else {
      ShellowNative.nativeSetRendererOverlayJson(handle, overlayJson)
    }

  fun attachAndroidNativeWindow(
    rawHandle: Long,
    widthPx: Int,
    heightPx: Int,
  ): String =
    if (handle == 0L) {
      "{\"error\":${JSONObject.quote(initFailure ?: "native engine is not available")}}"
    } else {
      ShellowNative.nativeAttachAndroidNativeWindowJson(handle, rawHandle, widthPx, heightPx)
    }

  fun attachAndroidSurface(
    surface: Surface,
    widthPx: Int,
    heightPx: Int,
  ): String =
    if (handle == 0L) {
      "{\"error\":${JSONObject.quote(initFailure ?: "native engine is not available")}}"
    } else {
      ShellowNative.nativeAttachAndroidSurfaceJson(handle, surface, widthPx, heightPx)
    }

  fun renderRendererSurfaceFrame(
    widthPx: Int,
    heightPx: Int,
    firstRow: Int,
    rowCount: Int,
  ): Boolean =
    try {
      JSONObject(renderFrameViewportJson(widthPx, heightPx, firstRow, rowCount))
        .optBoolean("native_surface_terminal_frame_presented_this_frame", false)
    } catch (_: Throwable) {
      false
    }

  fun detachRendererSurface(): String =
    if (handle == 0L) {
      "{\"error\":${JSONObject.quote(initFailure ?: "native engine is not available")}}"
    } else {
      ShellowNative.nativeDetachRendererSurfaceJson(handle)
    }

  fun sendCommand(command: String) = decode { ShellowNative.nativeSendCommandJson(handle, command) }

  fun sendTerminalInput(input: String) = decode { ShellowNative.nativeSendTerminalInputJson(handle, input) }

  fun resizeTerminal(cols: Int, rows: Int) = decode { ShellowNative.nativeResizeTerminalJson(handle, cols, rows) }

  fun clearTerminal() = decode { ShellowNative.nativeClearTerminalJson(handle) }

  fun resetTerminal() = decode { ShellowNative.nativeResetTerminalJson(handle) }

  fun connectPreview(profile: HostProfile) =
    decode {
      ShellowNative.nativeConnectPreviewJson(
        handle,
        profile.name,
        profile.host,
        profile.port,
        profile.username,
        profile.trustedHostKeySha256.orEmpty(),
        profile.authentication.wire,
      )
    }

  fun startPasswordShell(profile: HostProfile, password: String) =
    decode {
      ShellowNative.nativeStartPasswordShellJson(
        handle,
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
    decode {
      ShellowNative.nativeStartPrivateKeyShellJson(
        handle,
        profile.name,
        profile.host,
        profile.port,
        profile.username,
        profile.trustedHostKeySha256.orEmpty(),
        privateKeyPem,
        passphrase,
      )
    }

  fun pollLiveShell() = decode { ShellowNative.nativePollLiveShellJson(handle) }

  fun disconnectLiveShell() = decode { ShellowNative.nativeDisconnectLiveShellJson(handle) }

  override fun close() {
    val current = handle
    if (current != 0L) {
      ShellowNative.nativeDestroy(current)
      handle = 0L
    }
  }

  private fun decode(body: () -> String): TerminalSession =
    if (handle == 0L) {
      TerminalSession.bridgeFailure(initFailure ?: "native engine is not available")
    } else {
      decodeNative(body)
    }

  private fun decodeNative(body: () -> String): TerminalSession =
    try {
      TerminalSession.fromJson(body())
    } catch (error: Throwable) {
      TerminalSession.bridgeFailure(error.message ?: error.toString())
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
  external fun nativeRendererInfoJson(handle: Long): String
  external fun nativeSetRendererOverlayJson(handle: Long, overlayJson: String): String
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
}

private fun <T> JSONArray.mapObjects(transform: (JSONObject) -> T): List<T> =
  List(length()) { index -> transform(getJSONObject(index)) }

private fun JSONArray.mapStrings(): List<String> =
  List(length()) { index -> optString(index) }

private fun JSONArray.mapInts(): List<Int> =
  List(length()) { index -> optInt(index) }
