package xyz.zinglix.shellow.ui

import android.graphics.Paint
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.Typeface
import android.content.Context
import android.os.Environment
import android.util.Base64
import android.util.Log
import android.view.Surface
import android.view.SurfaceHolder
import android.view.SurfaceView
import xyz.zinglix.shellow.BuildConfig
import androidx.activity.BackEventCompat
import androidx.activity.compose.BackHandler
import androidx.activity.compose.PredictiveBackHandler
import androidx.compose.ui.viewinterop.AndroidView
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.gestures.detectDragGesturesAfterLongPress
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.ime
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.Checkbox
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color as ComposeColor
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.TransformOrigin
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.graphics.drawscope.drawIntoCanvas
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.nativeCanvas
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEvent
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.isAltPressed
import androidx.compose.ui.input.key.isCtrlPressed
import androidx.compose.ui.input.key.isMetaPressed
import androidx.compose.ui.input.key.isShiftPressed
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.input.key.utf16CodePoint
import androidx.compose.ui.input.nestedscroll.NestedScrollConnection
import androidx.compose.ui.input.nestedscroll.NestedScrollSource
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.unit.Velocity
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.zIndex
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.LifecycleEventEffect
import androidx.core.content.edit
import xyz.zinglix.shellow.core.AuthenticationKind
import xyz.zinglix.shellow.core.CodexApproval
import xyz.zinglix.shellow.core.CodexApprovalKind
import xyz.zinglix.shellow.core.CodexDirectoryEntry
import xyz.zinglix.shellow.core.CodexMarkdownBlock
import xyz.zinglix.shellow.core.CodexMarkdownBlockKind
import xyz.zinglix.shellow.core.CodexMarkdownInlineRun
import xyz.zinglix.shellow.core.CodexMarkdownInlineStyle
import xyz.zinglix.shellow.core.CodexMarkdownTableCell
import xyz.zinglix.shellow.core.CodexMessage
import xyz.zinglix.shellow.core.CodexMessageKind
import xyz.zinglix.shellow.core.CodexMessageDelivery
import xyz.zinglix.shellow.core.CodexMessageRole
import xyz.zinglix.shellow.core.CodexMessageVisibility
import xyz.zinglix.shellow.core.CodexModelOption
import xyz.zinglix.shellow.core.CodexRateLimitWindow
import xyz.zinglix.shellow.core.CodexSnapshot
import xyz.zinglix.shellow.core.CodexStatus
import xyz.zinglix.shellow.core.CodexThreadSummary
import xyz.zinglix.shellow.core.CodexUsageState
import xyz.zinglix.shellow.core.ConnectionState
import xyz.zinglix.shellow.core.HostProfile
import xyz.zinglix.shellow.core.IntegrationReport
import xyz.zinglix.shellow.core.PersistentTerminalBackend
import xyz.zinglix.shellow.core.PersistentTerminalConfiguration
import xyz.zinglix.shellow.core.ProfileLaunchKind
import xyz.zinglix.shellow.core.RemoteComponentSupportLevel
import xyz.zinglix.shellow.core.RemoteHostCapabilityProbe
import xyz.zinglix.shellow.core.RemoteHostProbeOutcome
import xyz.zinglix.shellow.core.RemoteTerminalSessionCatalog
import xyz.zinglix.shellow.core.RemoteTerminalSessionProbe
import xyz.zinglix.shellow.core.RemoteTerminalSessionSummary
import xyz.zinglix.shellow.core.ShellowCoreSession
import xyz.zinglix.shellow.core.SSHSecretKind
import xyz.zinglix.shellow.core.SSHSecretStore
import xyz.zinglix.shellow.core.TerminalCursorShape
import xyz.zinglix.shellow.core.TerminalGridSnapshot
import xyz.zinglix.shellow.core.TerminalGridColor
import xyz.zinglix.shellow.core.TerminalGridRun
import xyz.zinglix.shellow.core.TerminalGridStyle
import xyz.zinglix.shellow.core.TerminalRow
import xyz.zinglix.shellow.core.TerminalRowStyle
import xyz.zinglix.shellow.core.TerminalScreenKind
import xyz.zinglix.shellow.core.TerminalScrollDirection
import xyz.zinglix.shellow.core.TerminalSession
import xyz.zinglix.shellow.core.duplicated
import xyz.zinglix.shellow.core.scrollInputSequence
import xyz.zinglix.shellow.theme.ShellowColorScheme
import xyz.zinglix.shellow.theme.ShellowColors
import xyz.zinglix.shellow.theme.ShellowTheme
import java.io.File
import java.net.URL
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import java.util.UUID
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.json.JSONArray
import org.json.JSONObject
import kotlin.math.abs
import kotlin.math.floor
import kotlin.math.roundToInt

private fun Modifier.predictiveBackTransform(
  progress: Float,
  swipeEdge: Int,
): Modifier =
  graphicsLayer {
    val clampedProgress = progress.coerceIn(0f, 1f)
    val direction = if (swipeEdge == BackEventCompat.EDGE_RIGHT) -1f else 1f
    val scale = 1f - (0.08f * clampedProgress)
    scaleX = scale
    scaleY = scale
    translationX = size.width * 0.06f * clampedProgress * direction
    transformOrigin =
      TransformOrigin(
        pivotFractionX = if (direction > 0f) 1f else 0f,
        pivotFractionY = 0.5f,
      )
    alpha = 1f - (0.04f * clampedProgress)
  }
private enum class TerminalDestructiveAction {
  Clear,
  Reset,
}

private enum class HostConnectMode(val passwordTitle: String) {
  Terminal("Terminal Password"),
  Codex("Codex Password"),
  Claude("Claude Code Password"),
}

private const val TerminalDirectInputSentinel = "\u2060"
private const val RendererLogTag = "ShellowRenderer"
private const val TerminalKeyboardLayoutCommitDelayMs = 260L

private data class TerminalSelectionPoint(
  val row: Int,
  val column: Int,
) {
  companion object {
    const val LineEndColumn: Int = Int.MAX_VALUE / 4
  }
}

private sealed class TerminalSelection {
  data class Grid(val anchor: TerminalSelectionPoint, val focus: TerminalSelectionPoint) : TerminalSelection()
  data class History(val anchor: Int, val focus: Int) : TerminalSelection()

  companion object
}

private data class TerminalCellRange(
  val start: Int,
  val endExclusive: Int,
) {
  fun overlaps(
    segmentStart: Int,
    segmentEndExclusive: Int,
  ): Boolean =
    start < segmentEndExclusive && segmentStart < endExclusive
}

private sealed class TerminalSearchHit {
  data class Grid(val row: Int, val range: TerminalCellRange) : TerminalSearchHit()
  data class History(val row: Int) : TerminalSearchHit()
}

private data class TerminalSearchPresentation(
  val query: String,
  val hits: List<TerminalSearchHit>,
  val activeHit: TerminalSearchHit?,
) {
  val isEmpty: Boolean
    get() = query.isBlank()

  val activeOrdinal: Int
    get() = activeHit?.let { hits.indexOf(it) + 1 } ?: 0

  val activeGridRow: Int?
    get() = (activeHit as? TerminalSearchHit.Grid)?.row

  val activeGridRange: TerminalCellRange?
    get() = (activeHit as? TerminalSearchHit.Grid)?.range

  fun containsGrid(row: Int): Boolean =
    hits.any { it is TerminalSearchHit.Grid && it.row == row }

  fun containsHistory(row: Int): Boolean =
    hits.contains(TerminalSearchHit.History(row))

  fun gridRanges(row: Int): List<TerminalCellRange> =
    hits.mapNotNull { hit ->
      if (hit is TerminalSearchHit.Grid && hit.row == row) hit.range else null
    }
}

private data class RemoteClipboardRequest(
  val sequence: Long,
  val text: String,
)

private data class TranscriptSaveResult(
  val title: String,
  val message: String,
)

private data class AppDisplaySettings(
  val fontSizeSp: Float = 14f,
  val lineHeightScale: Float = 1f,
  val colorScheme: ShellowColorScheme = ShellowColorScheme.System,
  val terminalTheme: TerminalThemeSelection = TerminalThemeSelection.ShellowDark,
  val confirmPaste: Boolean = true,
  val showKeyboardToolbar: Boolean = true,
  val keepAliveSeconds: Float = 30f,
  val detectRemotePorts: Boolean = false,
)

private enum class TerminalThemeSelection(
  val wire: String,
  val title: String,
  val background: ComposeColor,
) {
  ShellowDark("shellow_dark", "Shellow Dark", ComposeColor(0xFF0D0F0E)),
  Midnight("midnight", "Midnight", ComposeColor(0xFF0B1220)),
  Amber("amber", "Amber", ComposeColor(0xFF17130D)),
  PaperLight("paper_light", "Paper Light", ComposeColor(0xFFFAF8F2));

  companion object {
    fun fromWire(value: String?) = entries.firstOrNull { it.wire == value } ?: ShellowDark
  }
}

private data class SSHKeyCredential(
  val id: String = UUID.randomUUID().toString(),
  val name: String,
)

private data class StoredPrivateKeyAuth(
  val credential: SSHKeyCredential,
  val privateKeyPem: String,
  val passphrase: String,
)

private data class PasswordPromptRequest(
  val profile: HostProfile,
  val mode: HostConnectMode,
  val reason: String?,
)

private data class ConnectionNotice(
  val title: String,
  val message: String,
)

private data class PendingHostKeyTrust(
  val fingerprint: String,
  val mode: HostConnectMode,
)

private const val HostKeyConfirmationPrefix = "ssh host key confirmation required: "
private sealed class ReconnectTarget {
  data class Preview(val profile: HostProfile) : ReconnectTarget()
  data class Password(
    val profile: HostProfile,
    val password: String,
    val startupCommand: String,
  ) : ReconnectTarget()
  data class PrivateKey(
    val profile: HostProfile,
    val privateKeyPem: String,
    val passphrase: String,
    val startupCommand: String,
  ) : ReconnectTarget()
}

private sealed class CodexReconnectTarget {
  data class Password(
    val profile: HostProfile,
    val password: String,
    val cwd: String,
    val threadId: String? = null,
  ) : CodexReconnectTarget()

  data class PrivateKey(
    val profile: HostProfile,
    val privateKeyPem: String,
    val passphrase: String,
    val cwd: String,
    val threadId: String? = null,
  ) : CodexReconnectTarget()
}

private fun ReconnectTarget.profile(): HostProfile =
  when (this) {
    is ReconnectTarget.Preview -> profile
    is ReconnectTarget.Password -> profile
    is ReconnectTarget.PrivateKey -> profile
  }

private fun ReconnectTarget.withProfile(profile: HostProfile): ReconnectTarget =
  when (this) {
    is ReconnectTarget.Preview -> copy(profile = profile)
    is ReconnectTarget.Password -> copy(profile = profile)
    is ReconnectTarget.PrivateKey -> copy(profile = profile)
  }

private fun CodexReconnectTarget.profile(): HostProfile =
  when (this) {
    is CodexReconnectTarget.Password -> profile
    is CodexReconnectTarget.PrivateKey -> profile
  }

private fun CodexReconnectTarget.withProfile(profile: HostProfile): CodexReconnectTarget =
  when (this) {
    is CodexReconnectTarget.Password -> copy(profile = profile)
    is CodexReconnectTarget.PrivateKey -> copy(profile = profile)
  }

private fun CodexReconnectTarget.withCwd(cwd: String): CodexReconnectTarget =
  when (this) {
    is CodexReconnectTarget.Password -> copy(cwd = cwd)
    is CodexReconnectTarget.PrivateKey -> copy(cwd = cwd)
  }

private fun CodexReconnectTarget.withThreadId(threadId: String?): CodexReconnectTarget =
  when (this) {
    is CodexReconnectTarget.Password -> copy(threadId = threadId)
    is CodexReconnectTarget.PrivateKey -> copy(threadId = threadId)
  }

private fun HostProfile.matchesProfileIdentity(other: HostProfile): Boolean =
  name == other.name &&
    host == other.host &&
    port == other.port &&
    username == other.username &&
    authentication == other.authentication

@Composable
fun ShellowApp() {
  val core = remember { ShellowCoreSession() }
  val context = LocalContext.current
  val secretStore = remember { SSHSecretStore(context) }
  val scope = rememberCoroutineScope()
  val initialDisplaySettings = remember(context) { loadDisplaySettings(context) }
  var displaySettings by remember { mutableStateOf(initialDisplaySettings) }
  val profiles =
    remember {
      mutableStateListOf<HostProfile>().also { profiles ->
        profiles.addAll(loadHostProfiles(context))
      }
    }
  val sshKeys =
    remember {
      mutableStateListOf<SSHKeyCredential>().also { keys ->
        keys.addAll(loadSSHKeyCredentials(context))
      }
    }
  var navigation by remember { mutableStateOf(AppNavigationState()) }
  val screen = navigation.currentScreen
  var topLevelBackProgress by remember { mutableStateOf(0f) }
  var topLevelBackEdge by remember { mutableStateOf(BackEventCompat.EDGE_LEFT) }
  var session by remember {
    core.setTerminalTheme(initialDisplaySettings.terminalTheme.wire)
    core.setTransportOptions(
      initialDisplaySettings.keepAliveSeconds.roundToInt().toLong(),
      initialDisplaySettings.detectRemotePorts,
    )
    mutableStateOf(core.snapshot())
  }
  var codexSnapshot by remember { mutableStateOf(CodexSnapshot.disconnected()) }
  var claudeSnapshot by remember { mutableStateOf(CodexSnapshot.disconnected()) }
  var reconnectTarget by remember { mutableStateOf<ReconnectTarget?>(null) }
  var codexReconnectTarget by remember { mutableStateOf<CodexReconnectTarget?>(null) }
  var claudeReconnectTarget by remember { mutableStateOf<CodexReconnectTarget?>(null) }
  var passwordPrompt by remember { mutableStateOf<PasswordPromptRequest?>(null) }
  var connectionNotice by remember { mutableStateOf<ConnectionNotice?>(null) }

  fun navigateTo(destination: AppScreen) {
    navigation = navigation.navigateTo(destination)
  }

  fun navigateBack() {
    navigation = navigation.navigateBack()
  }
  var codexBootstrapPromptEndpoint by remember { mutableStateOf<String?>(null) }
  var codexBootstrapError by remember { mutableStateOf<String?>(null) }
  var pendingHostKeyTrust by remember { mutableStateOf<PendingHostKeyTrust?>(null) }
  var reconnectTerminalAfterBackground by remember { mutableStateOf(false) }
  var reconnectCodexAfterBackground by remember { mutableStateOf(false) }

  fun updateStoredProfile(updated: HostProfile) {
    val index = profiles.indexOfFirst { it.id == updated.id }
    if (index < 0) return
    profiles[index] = updated
    saveHostProfiles(context, profiles)
    reconnectTarget = reconnectTarget?.takeIf { it.profile().id == updated.id }?.withProfile(updated) ?: reconnectTarget
    codexReconnectTarget =
      codexReconnectTarget?.takeIf { it.profile().id == updated.id }?.withProfile(updated) ?: codexReconnectTarget
    claudeReconnectTarget =
      claudeReconnectTarget?.takeIf { it.profile().id == updated.id }?.withProfile(updated) ?: claudeReconnectTarget
  }

  fun applyCapabilityOutcome(profile: HostProfile, outcome: RemoteHostProbeOutcome) {
    val report = outcome.report ?: return
    val stored = profiles.firstOrNull { it.id == profile.id } ?: profile
    val updated =
      stored.copy(
        trustedHostKeySha256 =
          stored.trustedHostKeySha256
            ?: outcome.observedHostKeySha256?.trim()?.takeIf { it.isNotEmpty() },
        capabilityReport = report,
      )
    updateStoredProfile(updated)
  }

  suspend fun probeWithPassword(profile: HostProfile, password: String): RemoteHostProbeOutcome =
    withContext(Dispatchers.IO) {
      ShellowCoreSession().use { probeCore ->
        RemoteHostCapabilityProbe.outcome(
          probeCore.connectPasswordExec(profile, password, RemoteHostCapabilityProbe.command),
        )
      }
    }

  suspend fun probeWithPrivateKey(
    profile: HostProfile,
    privateKeyPem: String,
    passphrase: String,
  ): RemoteHostProbeOutcome =
    withContext(Dispatchers.IO) {
      ShellowCoreSession().use { probeCore ->
        RemoteHostCapabilityProbe.outcome(
          probeCore.connectPrivateKeyExec(
            profile,
            privateKeyPem,
            passphrase,
            RemoteHostCapabilityProbe.command,
          ),
        )
      }
    }

  suspend fun probeWithStoredCredential(profile: HostProfile): RemoteHostProbeOutcome {
    val savedPassword = withContext(Dispatchers.IO) { secretStore.loadSecret(profile, SSHSecretKind.Password) }
    if (!savedPassword.isNullOrBlank()) {
      return probeWithPassword(profile, savedPassword)
    }

    val savedKeys =
      withContext(Dispatchers.IO) {
        sshKeys.mapNotNull { credential ->
          val privateKeyPem = secretStore.loadKeySecret(credential.id, SSHSecretKind.PrivateKey)
          if (privateKeyPem.isNullOrBlank() || !privateKeyLooksUsable(privateKeyPem)) {
            null
          } else {
            StoredPrivateKeyAuth(
              credential,
              privateKeyPem,
              secretStore.loadKeySecret(credential.id, SSHSecretKind.Passphrase).orEmpty(),
            )
          }
        }
      }
    if (savedKeys.isEmpty()) {
      return RemoteHostProbeOutcome(errorMessage = "Connect once or save an SSH credential before detecting this host.")
    }

    var lastOutcome = RemoteHostProbeOutcome(errorMessage = "Saved SSH keys did not authenticate.")
    savedKeys.forEach { key ->
      val outcome = probeWithPrivateKey(profile, key.privateKeyPem, key.passphrase)
      if (outcome.report != null) return outcome
      lastOutcome = outcome
    }
    return lastOutcome
  }

  suspend fun loadRemoteTerminalSessions(
    profile: HostProfile,
    configuration: PersistentTerminalConfiguration,
  ): RemoteTerminalSessionCatalog {
    val target = reconnectTarget
    if (target == null || target.profile().id != profile.id) {
      return RemoteTerminalSessionCatalog(errorMessage = "Reconnect this profile to load remote sessions.")
    }

    val command = RemoteTerminalSessionProbe.command(configuration.backend)
    val result =
      withContext(Dispatchers.IO) {
        ShellowCoreSession().use { probeCore ->
          when (target) {
            is ReconnectTarget.Password ->
              probeCore.connectPasswordExec(profile, target.password, command)
            is ReconnectTarget.PrivateKey ->
              probeCore.connectPrivateKeyExec(
                profile,
                target.privateKeyPem,
                target.passphrase,
                command,
              )
            is ReconnectTarget.Preview -> null
          }
        }
      }
      ?: return RemoteTerminalSessionCatalog(errorMessage = "Remote sessions are unavailable in preview mode.")

    val output = result.rows.joinToString("\n") { it.text }
    return RemoteTerminalSessionProbe.parse(output)
      ?: RemoteTerminalSessionCatalog(
        errorMessage =
          result.rows
            .asReversed()
            .firstOrNull { it.text.isNotBlank() }
            ?.text
            ?: "The host did not return a recognizable session list.",
      )
  }

  fun captureObservedHostKeyIfNeeded(next: TerminalSession) {
    val observed = next.observedHostKeySha256?.trim().takeUnless { it.isNullOrEmpty() } ?: return
    val target = reconnectTarget ?: return
    val profile = target.profile()
    if (!profile.trustedHostKeySha256.isNullOrBlank()) return

    val updated = profile.copy(trustedHostKeySha256 = observed)
    val index = profiles.indexOfFirst { it.matchesProfileIdentity(profile) }
    if (index >= 0) {
      profiles[index] = updated
      saveHostProfiles(context, profiles)
    }
    reconnectTarget = target.withProfile(updated)
  }

  fun captureObservedHostKeyIfNeeded(next: CodexSnapshot) {
    val observed = next.observedHostKeySha256?.trim().takeUnless { it.isNullOrEmpty() } ?: return
    val target = codexReconnectTarget ?: return
    val profile = target.profile()
    if (!profile.trustedHostKeySha256.isNullOrBlank()) return

    val updated = profile.copy(trustedHostKeySha256 = observed)
    val index = profiles.indexOfFirst { it.matchesProfileIdentity(profile) }
    if (index >= 0) {
      profiles[index] = updated
      saveHostProfiles(context, profiles)
    }
    codexReconnectTarget = target.withProfile(updated)
  }

  fun updateSession(next: TerminalSession) {
    session = next
    if (pendingHostKeyTrust == null && reconnectTarget?.profile()?.trustedHostKeySha256.isNullOrBlank()) {
      next.rows.asReversed().firstNotNullOfOrNull { row ->
        row.text.hostKeyFingerprintOrNull()
      }?.let { fingerprint ->
        pendingHostKeyTrust = PendingHostKeyTrust(fingerprint, HostConnectMode.Terminal)
      }
    }
    captureObservedHostKeyIfNeeded(next)
  }

  fun rememberCodexResumePoint(next: CodexSnapshot) {
    var target = codexReconnectTarget ?: return
    next.cwd?.trim()?.takeUnless { it.isEmpty() }?.let { cwd ->
      target = target.withCwd(cwd)
    }
    next.threadId?.trim()?.takeUnless { it.isEmpty() }?.let { threadId ->
      target = target.withThreadId(threadId)
    }
    codexReconnectTarget = target
  }

  fun updateCodexSnapshot(next: CodexSnapshot) {
    val resolved =
      if (!next.messagesReplaceAll && next.messagesStartIndex <= codexSnapshot.messages.size) {
        next.copy(
          messages = codexSnapshot.messages.take(next.messagesStartIndex) + next.messages,
          messagesStartIndex = 0,
          messagesReplaceAll = true,
        )
      } else {
        next
      }
    codexSnapshot = resolved
    if (pendingHostKeyTrust == null && codexReconnectTarget?.profile()?.trustedHostKeySha256.isNullOrBlank()) {
      (next.lastError?.hostKeyFingerprintOrNull()
        ?: next.messages.asReversed().firstNotNullOfOrNull { it.text.hostKeyFingerprintOrNull() })
        ?.let { fingerprint ->
          pendingHostKeyTrust = PendingHostKeyTrust(fingerprint, HostConnectMode.Codex)
        }
    }
    rememberCodexResumePoint(resolved)
    captureObservedHostKeyIfNeeded(resolved)
    if (resolved.lastError?.contains("daemon bootstrap --remote-control") == true &&
      codexBootstrapPromptEndpoint == null && codexBootstrapError == null
    ) {
      codexBootstrapPromptEndpoint = codexReconnectTarget?.profile()?.endpoint ?: resolved.endpoint
    }
  }

  fun updateClaudeSnapshot(next: CodexSnapshot) {
    claudeSnapshot = next
    var target = claudeReconnectTarget
    if (pendingHostKeyTrust == null && target?.profile()?.trustedHostKeySha256.isNullOrBlank()) {
      (next.lastError?.hostKeyFingerprintOrNull()
        ?: next.messages.asReversed().firstNotNullOfOrNull { it.text.hostKeyFingerprintOrNull() })
        ?.let { fingerprint ->
          pendingHostKeyTrust = PendingHostKeyTrust(fingerprint, HostConnectMode.Claude)
        }
    }
    next.cwd?.trim()?.takeUnless { it.isEmpty() }?.let { cwd -> target = target?.withCwd(cwd) }
    next.threadId?.trim()?.takeUnless { it.isEmpty() }?.let { sessionId -> target = target?.withThreadId(sessionId) }
    val observed = next.observedHostKeySha256?.trim().takeUnless { it.isNullOrEmpty() }
    val targetForHostKey = target
    if (observed != null && targetForHostKey != null && targetForHostKey.profile().trustedHostKeySha256.isNullOrBlank()) {
      val profile = targetForHostKey.profile()
      val updated = profile.copy(trustedHostKeySha256 = observed)
      val index = profiles.indexOfFirst { it.matchesProfileIdentity(profile) }
      if (index >= 0) {
        profiles[index] = updated
        saveHostProfiles(context, profiles)
      }
      target = targetForHostKey.withProfile(updated)
    }
    claudeReconnectTarget = target
  }

  LaunchedEffect(core) {
    var lastLiveRevision = withContext(Dispatchers.IO) { core.liveShellEventRevision() }
    var lastCodexRevision = withContext(Dispatchers.IO) { core.codexEventRevision() }
    var lastClaudeRevision = withContext(Dispatchers.IO) { core.claudeEventRevision() }
    var codexConnectingPollTicks = 0
    while (true) {
      delay(50)
      val revision = withContext(Dispatchers.IO) { core.liveShellEventRevision() }
      if (revision != lastLiveRevision) {
        lastLiveRevision = revision
        val next = withContext(Dispatchers.IO) { core.pollLiveShell() }
        if (next != session) {
          updateSession(next)
        }
      }

      val codexRevision = withContext(Dispatchers.IO) { core.codexEventRevision() }
      if (codexRevision != lastCodexRevision) {
        lastCodexRevision = codexRevision
        codexConnectingPollTicks = 0
        val next = withContext(Dispatchers.IO) { core.pollCodex() }
        if (next != codexSnapshot) {
          updateCodexSnapshot(next)
        }
      } else if (codexSnapshot.status == CodexStatus.Connecting) {
        codexConnectingPollTicks += 1
        if (codexConnectingPollTicks >= 5) {
          codexConnectingPollTicks = 0
          updateCodexSnapshot(withContext(Dispatchers.IO) { core.pollCodex() })
        }
      }

      val claudeRevision = withContext(Dispatchers.IO) { core.claudeEventRevision() }
      if (claudeRevision != lastClaudeRevision) {
        lastClaudeRevision = claudeRevision
        val next = withContext(Dispatchers.IO) { core.pollClaude() }
        if (next != claudeSnapshot) {
          updateClaudeSnapshot(next)
        }
      }
    }
  }

  DisposableEffect(core) {
    onDispose { core.close() }
  }

  LaunchedEffect(displaySettings) {
    saveDisplaySettings(context, displaySettings)
    core.setTransportOptions(
      displaySettings.keepAliveSeconds.roundToInt().toLong(),
      displaySettings.detectRemotePorts,
    )
  }

  fun connectPasswordShell(profile: HostProfile, password: String, startupCommand: String) {
    session = TerminalSession.connecting(profile)
    navigateTo(AppScreen.Terminal)
    scope.launch {
      updateSession(withContext(Dispatchers.IO) { core.startPasswordShell(profile, password) })
      val command = startupCommand.trim()
      if (command.isNotEmpty() && session.state != ConnectionState.Disconnected) {
        delay(450)
        updateSession(withContext(Dispatchers.IO) { core.pollLiveShell() })
      }
      if (command.isNotEmpty() && session.state != ConnectionState.Disconnected) {
        updateSession(withContext(Dispatchers.IO) { core.sendTerminalInput("$command\r") })
      }
      if (profile.capabilityReport?.isStale != false) {
        applyCapabilityOutcome(profile, probeWithPassword(profile, password))
      }
    }
  }

  fun connectPrivateKeyShell(
    profile: HostProfile,
    privateKeyPem: String,
    passphrase: String,
    startupCommand: String,
  ) {
    session = TerminalSession.connecting(profile)
    navigateTo(AppScreen.Terminal)
    scope.launch {
      updateSession(withContext(Dispatchers.IO) { core.startPrivateKeyShell(profile, privateKeyPem, passphrase) })
      val command = startupCommand.trim()
      if (command.isNotEmpty() && session.state != ConnectionState.Disconnected) {
        delay(450)
        updateSession(withContext(Dispatchers.IO) { core.pollLiveShell() })
      }
      if (command.isNotEmpty() && session.state != ConnectionState.Disconnected) {
        updateSession(withContext(Dispatchers.IO) { core.sendTerminalInput("$command\r") })
      }
      if (profile.capabilityReport?.isStale != false) {
        applyCapabilityOutcome(profile, probeWithPrivateKey(profile, privateKeyPem, passphrase))
      }
    }
  }

  fun startCodexPassword(profile: HostProfile, password: String, cwd: String) {
    codexSnapshot = CodexSnapshot.connecting(profile, cwd)
    navigateTo(AppScreen.Codex)
    scope.launch {
      updateCodexSnapshot(withContext(Dispatchers.IO) { core.startCodexPassword(profile, password, cwd) })
    }
  }

  fun startCodexPrivateKey(
    profile: HostProfile,
    privateKeyPem: String,
    passphrase: String,
    cwd: String,
  ) {
    codexSnapshot = CodexSnapshot.connecting(profile, cwd)
    navigateTo(AppScreen.Codex)
    scope.launch {
      updateCodexSnapshot(withContext(Dispatchers.IO) { core.startCodexPrivateKey(profile, privateKeyPem, passphrase, cwd) })
    }
  }

  fun startClaudePassword(
    profile: HostProfile,
    password: String,
    cwd: String,
    sessionId: String = "",
  ) {
    claudeSnapshot = CodexSnapshot.connecting(profile, cwd)
    navigateTo(AppScreen.Claude)
    scope.launch {
      updateClaudeSnapshot(
        withContext(Dispatchers.IO) { core.startClaudePassword(profile, password, cwd, sessionId) },
      )
    }
  }

  fun startClaudePrivateKey(
    profile: HostProfile,
    privateKeyPem: String,
    passphrase: String,
    cwd: String,
    sessionId: String = "",
  ) {
    claudeSnapshot = CodexSnapshot.connecting(profile, cwd)
    navigateTo(AppScreen.Claude)
    scope.launch {
      updateClaudeSnapshot(
        withContext(Dispatchers.IO) {
          core.startClaudePrivateKey(profile, privateKeyPem, passphrase, cwd, sessionId)
        },
      )
    }
  }

  fun startPasswordConnection(
    profile: HostProfile,
    password: String,
    mode: HostConnectMode,
  ) {
    when (mode) {
      HostConnectMode.Terminal -> {
        reconnectTarget = ReconnectTarget.Password(profile, password, profile.terminalStartupCommand)
        connectPasswordShell(profile, password, profile.terminalStartupCommand)
      }
      HostConnectMode.Codex -> {
        codexReconnectTarget = CodexReconnectTarget.Password(profile, password, "")
        startCodexPassword(profile, password, "")
      }
      HostConnectMode.Claude -> {
        claudeReconnectTarget = CodexReconnectTarget.Password(profile, password, "")
        startClaudePassword(profile, password, "")
      }
    }
  }

  fun storedPrivateKeyAuths(profile: HostProfile? = null): List<StoredPrivateKeyAuth> =
    sshKeys.mapNotNull { credential ->
      if (
        profile?.authentication == AuthenticationKind.PrivateKey &&
          profile.preferredKeyId != null &&
          profile.preferredKeyId != credential.id
      ) {
        return@mapNotNull null
      }
      val privateKeyPem = secretStore.loadKeySecret(credential.id, SSHSecretKind.PrivateKey)
      if (privateKeyPem.isNullOrBlank() || !privateKeyLooksUsable(privateKeyPem)) {
        null
      } else {
        StoredPrivateKeyAuth(
          credential = credential,
          privateKeyPem = privateKeyPem,
          passphrase = secretStore.loadKeySecret(credential.id, SSHSecretKind.Passphrase).orEmpty(),
        )
      }
    }

  suspend fun waitForTerminalConnectionResult(): TerminalSession {
    val deadline = System.currentTimeMillis() + 8_000
    var current = session
    while (current.state == ConnectionState.Connecting && System.currentTimeMillis() < deadline) {
      delay(200)
      current = withContext(Dispatchers.IO) { core.pollLiveShell() }
      updateSession(current)
    }
    return current
  }

  suspend fun waitForCodexConnectionResult(): CodexSnapshot {
    val deadline = System.currentTimeMillis() + 10_000
    var current = codexSnapshot
    while (current.status == CodexStatus.Connecting && System.currentTimeMillis() < deadline) {
      delay(250)
      current = withContext(Dispatchers.IO) { core.pollCodex() }
      updateCodexSnapshot(current)
    }
    return current
  }

  suspend fun waitForClaudeConnectionResult(): CodexSnapshot {
    val deadline = System.currentTimeMillis() + 10_000
    var current = claudeSnapshot
    while (current.status == CodexStatus.Connecting && System.currentTimeMillis() < deadline) {
      delay(250)
      current = withContext(Dispatchers.IO) { core.pollClaude() }
      updateClaudeSnapshot(current)
    }
    return current
  }

  suspend fun resumeCodexThreadAfterReconnect(threadId: String?) {
    val resumeThreadId = threadId?.trim().takeUnless { it.isNullOrEmpty() } ?: return
    if (codexSnapshot.status == CodexStatus.Connecting) {
      waitForCodexConnectionResult()
    }
    if (codexSnapshot.status == CodexStatus.Connected) {
      updateCodexSnapshot(withContext(Dispatchers.IO) { core.resumeCodexThread(resumeThreadId) })
    }
  }

  suspend fun reconnectCodexPassword(
    profile: HostProfile,
    password: String,
    cwd: String,
    threadId: String?,
  ) {
    codexSnapshot = CodexSnapshot.connecting(profile, cwd)
    navigateTo(AppScreen.Codex)
    updateCodexSnapshot(withContext(Dispatchers.IO) { core.startCodexPassword(profile, password, cwd) })
    resumeCodexThreadAfterReconnect(threadId)
  }

  suspend fun reconnectCodexPrivateKey(
    profile: HostProfile,
    privateKeyPem: String,
    passphrase: String,
    cwd: String,
    threadId: String?,
  ) {
    codexSnapshot = CodexSnapshot.connecting(profile, cwd)
    navigateTo(AppScreen.Codex)
    updateCodexSnapshot(withContext(Dispatchers.IO) { core.startCodexPrivateKey(profile, privateKeyPem, passphrase, cwd) })
    resumeCodexThreadAfterReconnect(threadId)
  }

  suspend fun tryPrivateKeysForTerminal(
    profile: HostProfile,
    keys: List<StoredPrivateKeyAuth>,
  ): Boolean {
    session = TerminalSession.connecting(profile)
    navigateTo(AppScreen.Terminal)

    keys.forEach { key ->
      reconnectTarget =
        ReconnectTarget.PrivateKey(
          profile = profile,
          privateKeyPem = key.privateKeyPem,
          passphrase = key.passphrase,
          startupCommand = profile.terminalStartupCommand,
        )
      updateSession(
        withContext(Dispatchers.IO) {
          core.startPrivateKeyShell(profile, key.privateKeyPem, key.passphrase)
        },
      )

      val result = waitForTerminalConnectionResult()
      if (result.state == ConnectionState.Connected) {
        val command = profile.terminalStartupCommand.trim()
        if (command.isNotEmpty()) {
          delay(450)
          updateSession(withContext(Dispatchers.IO) { core.sendTerminalInput("$command\r") })
        }
        if (profile.capabilityReport?.isStale != false) {
          applyCapabilityOutcome(profile, probeWithPrivateKey(profile, key.privateKeyPem, key.passphrase))
        }
        return true
      }

      if (pendingHostKeyTrust != null) return true

      withContext(Dispatchers.IO) { core.disconnectLiveShell() }
    }

    return false
  }

  suspend fun tryPrivateKeysForCodex(
    profile: HostProfile,
    keys: List<StoredPrivateKeyAuth>,
  ): Boolean {
    codexSnapshot = CodexSnapshot.connecting(profile, "")
    navigateTo(AppScreen.Codex)

    keys.forEach { key ->
      codexReconnectTarget =
        CodexReconnectTarget.PrivateKey(
          profile = profile,
          privateKeyPem = key.privateKeyPem,
          passphrase = key.passphrase,
          cwd = "",
        )
      updateCodexSnapshot(
        withContext(Dispatchers.IO) {
          core.startCodexPrivateKey(profile, key.privateKeyPem, key.passphrase, "")
        },
      )

      val result = waitForCodexConnectionResult()
      if (result.status == CodexStatus.Connected) {
        return true
      }

      if (pendingHostKeyTrust != null) return true

      withContext(Dispatchers.IO) { core.disconnectCodex() }
    }

    return false
  }

  suspend fun tryPrivateKeysForClaude(
    profile: HostProfile,
    keys: List<StoredPrivateKeyAuth>,
  ): Boolean {
    claudeSnapshot = CodexSnapshot.connecting(profile, "")
    navigateTo(AppScreen.Claude)

    keys.forEach { key ->
      claudeReconnectTarget =
        CodexReconnectTarget.PrivateKey(
          profile = profile,
          privateKeyPem = key.privateKeyPem,
          passphrase = key.passphrase,
          cwd = "",
        )
      updateClaudeSnapshot(
        withContext(Dispatchers.IO) {
          core.startClaudePrivateKey(profile, key.privateKeyPem, key.passphrase, "")
        },
      )

      if (waitForClaudeConnectionResult().status == CodexStatus.Connected) return true
      withContext(Dispatchers.IO) { core.disconnectClaude() }
    }
    return false
  }

  fun connectHost(
    profile: HostProfile,
    mode: HostConnectMode,
  ) {
    scope.launch {
      val savedPassword = withContext(Dispatchers.IO) { secretStore.loadSecret(profile, SSHSecretKind.Password) }
      if (profile.authentication == AuthenticationKind.Password) {
        if (!savedPassword.isNullOrBlank()) {
          startPasswordConnection(profile, savedPassword, mode)
          return@launch
        }
        passwordPrompt =
          PasswordPromptRequest(
            profile = profile,
            mode = mode,
            reason = "Enter the password for this host. You can save it for faster connections next time.",
          )
        return@launch
      }

      val keys = withContext(Dispatchers.IO) { storedPrivateKeyAuths(profile) }
      if (keys.isEmpty()) {
        if (profile.authentication == AuthenticationKind.Automatic) {
          if (!savedPassword.isNullOrBlank()) {
            startPasswordConnection(profile, savedPassword, mode)
          } else {
            passwordPrompt =
              PasswordPromptRequest(
                profile = profile,
                mode = mode,
                reason = "No saved SSH key is available. Enter the password for this host.",
              )
          }
        } else {
          connectionNotice =
            ConnectionNotice(
              title = "SSH Key Unavailable",
              message =
                if (profile.preferredKeyId == null) "This profile only uses SSH keys, but no saved key is available."
                else "The SSH key selected for this profile is no longer available.",
            )
        }
        return@launch
      }

      val didConnect =
        when (mode) {
          HostConnectMode.Terminal -> tryPrivateKeysForTerminal(profile, keys)
          HostConnectMode.Codex -> tryPrivateKeysForCodex(profile, keys)
          HostConnectMode.Claude -> tryPrivateKeysForClaude(profile, keys)
        }

      if (!didConnect) {
        if (pendingHostKeyTrust != null) return@launch
        reconnectTarget = null
        codexReconnectTarget = null
        claudeReconnectTarget = null
        navigateTo(AppScreen.Hosts)
        if (profile.authentication == AuthenticationKind.Automatic) {
          if (!savedPassword.isNullOrBlank()) {
            startPasswordConnection(profile, savedPassword, mode)
          } else {
            passwordPrompt =
              PasswordPromptRequest(
                profile = profile,
                mode = mode,
                reason = "Saved SSH keys did not authenticate. Enter a password to continue.",
              )
          }
        } else {
          connectionNotice =
            ConnectionNotice(
              title = "SSH Key Authentication Failed",
              message = "None of the SSH keys selected for this profile authenticated successfully. Password fallback is disabled.",
            )
        }
      }
    }
  }

  fun reconnect() {
    when (val target = reconnectTarget) {
      is ReconnectTarget.Preview -> {
        updateSession(core.connectPreview(target.profile))
        navigateTo(AppScreen.Terminal)
      }
      is ReconnectTarget.Password ->
        connectPasswordShell(
          target.profile,
          target.password,
          target.startupCommand,
        )
      is ReconnectTarget.PrivateKey ->
        connectPrivateKeyShell(
          target.profile,
          target.privateKeyPem,
          target.passphrase,
          target.startupCommand,
        )
      null -> Unit
    }
  }

  fun reconnectCodex() {
    when (val target = codexReconnectTarget) {
      is CodexReconnectTarget.Password -> {
        val resumeThreadId = target.threadId ?: codexSnapshot.threadId
        scope.launch {
          reconnectCodexPassword(target.profile, target.password, target.cwd, resumeThreadId)
        }
      }
      is CodexReconnectTarget.PrivateKey -> {
        val resumeThreadId = target.threadId ?: codexSnapshot.threadId
        scope.launch {
          reconnectCodexPrivateKey(
            target.profile,
            target.privateKeyPem,
            target.passphrase,
            target.cwd,
            resumeThreadId,
          )
        }
      }
      null -> Unit
    }
  }

  fun reconnectClaude() {
    when (val target = claudeReconnectTarget) {
      is CodexReconnectTarget.Password ->
        startClaudePassword(
          target.profile,
          target.password,
          target.cwd,
          target.threadId ?: claudeSnapshot.threadId.orEmpty(),
        )
      is CodexReconnectTarget.PrivateKey ->
        startClaudePrivateKey(
          target.profile,
          target.privateKeyPem,
          target.passphrase,
          target.cwd,
          target.threadId ?: claudeSnapshot.threadId.orEmpty(),
        )
      null -> Unit
    }
  }

  suspend fun startFreshClaude(cwd: String, initialMessage: String? = null) {
    val target = claudeReconnectTarget ?: return
    claudeReconnectTarget = target.withCwd(cwd).withThreadId(null)
    claudeSnapshot = CodexSnapshot.connecting(target.profile(), cwd)
    navigateTo(AppScreen.Claude)
    val started =
      withContext(Dispatchers.IO) {
        when (target) {
          is CodexReconnectTarget.Password ->
            core.startClaudePassword(target.profile, target.password, cwd)
          is CodexReconnectTarget.PrivateKey ->
            core.startClaudePrivateKey(target.profile, target.privateKeyPem, target.passphrase, cwd)
        }
      }
    updateClaudeSnapshot(started)
    val connected = waitForClaudeConnectionResult()
    if (connected.status == CodexStatus.Connected && !initialMessage.isNullOrBlank()) {
      updateClaudeSnapshot(withContext(Dispatchers.IO) { core.sendClaudeMessage(initialMessage) })
    }
  }

  LifecycleEventEffect(Lifecycle.Event.ON_PAUSE) {
    reconnectTerminalAfterBackground = session.state != ConnectionState.Disconnected && reconnectTarget != null
    reconnectCodexAfterBackground = codexSnapshot.status != CodexStatus.Disconnected && codexReconnectTarget != null
  }
  LifecycleEventEffect(Lifecycle.Event.ON_RESUME) {
    scope.launch {
      val terminal = withContext(Dispatchers.IO) { core.pollLiveShell() }
      updateSession(terminal)
      val codex = withContext(Dispatchers.IO) { core.pollCodex() }
      updateCodexSnapshot(codex)
      if (reconnectTerminalAfterBackground && terminal.state == ConnectionState.Disconnected) reconnect()
      if (reconnectCodexAfterBackground &&
        (codex.status == CodexStatus.Disconnected || codex.status == CodexStatus.Failed)
      ) reconnectCodex()
      reconnectTerminalAfterBackground = false
      reconnectCodexAfterBackground = false
    }
  }

  val activeTerminalProfile = reconnectTarget?.profile()

  PredictiveBackHandler(enabled = screen == AppScreen.Terminal || screen == AppScreen.Settings) { events ->
    try {
      events.collect { event ->
        topLevelBackProgress = event.progress
        topLevelBackEdge = event.swipeEdge
      }
      navigateBack()
    } finally {
      topLevelBackProgress = 0f
    }
  }
  BackHandler(enabled = navigation.canNavigateBack && screen != AppScreen.Terminal && screen != AppScreen.Settings) {
    navigateBack()
  }

  ShellowTheme(colorScheme = displaySettings.colorScheme) {
    val hostsContent: @Composable () -> Unit = {
      HostsScreen(
        profiles = profiles,
        sshKeys = sshKeys,
        secretStore = secretStore,
        onOpenSettings = { navigateTo(AppScreen.Settings) },
        onAddProfile = { profile ->
          profiles.add(profile)
          saveHostProfiles(context, profiles)
        },
        onUpdateProfile = ::updateStoredProfile,
        onDeleteProfile = { profile ->
          if (reconnectTarget?.profile()?.id == profile.id) reconnectTarget = null
          if (codexReconnectTarget?.profile()?.id == profile.id) codexReconnectTarget = null
          if (claudeReconnectTarget?.profile()?.id == profile.id) claudeReconnectTarget = null
          secretStore.deleteSecret(profile, SSHSecretKind.Password)
          secretStore.deleteSecret(profile, SSHSecretKind.PrivateKey)
          secretStore.deleteSecret(profile, SSHSecretKind.Passphrase)
          profiles.removeAll { it.id == profile.id }
          saveHostProfiles(context, profiles)
        },
        onProbeCapabilities = { profile ->
          val outcome = probeWithStoredCredential(profile)
          applyCapabilityOutcome(profile, outcome)
          outcome
        },
        onAddKey = { credential ->
          sshKeys.add(credential)
          saveSSHKeyCredentials(context, sshKeys)
        },
        onDeleteKey = { credential ->
          sshKeys.removeAll { it.id == credential.id }
          secretStore.deleteKeySecret(credential.id, SSHSecretKind.PrivateKey)
          secretStore.deleteKeySecret(credential.id, SSHSecretKind.Passphrase)
          saveSSHKeyCredentials(context, sshKeys)
        },
        onConnectTerminal = { profile ->
          connectHost(profile, HostConnectMode.Terminal)
        },
        onConnectCodex = { profile ->
          connectHost(profile, HostConnectMode.Codex)
        },
        onConnectClaude = { profile ->
          connectHost(profile, HostConnectMode.Claude)
        },
      )
    }

    Box(
      modifier =
        Modifier
          .fillMaxSize()
          .background(ShellowColors.TerminalBackground)
          .statusBarsPadding()
          .navigationBarsPadding()
    ) {
      if (screen == AppScreen.Hosts || topLevelBackProgress > 0f) {
        hostsContent()
      }

      val outgoingScreenModifier =
        Modifier
          .fillMaxSize()
          .predictiveBackTransform(topLevelBackProgress, topLevelBackEdge)

      when (screen) {
        AppScreen.Terminal ->
          Box(outgoingScreenModifier) {
            TerminalScreen(
              session = session,
              displaySettings = displaySettings,
              profileName = activeTerminalProfile?.name ?: session.title,
              persistentTerminal = activeTerminalProfile?.persistentTerminal,
              loadPersistentSessions =
                activeTerminalProfile?.let { profile ->
                  profile.persistentTerminal?.let { configuration ->
                    {
                      loadRemoteTerminalSessions(profile, configuration)
                    }
                  }
                },
              onBackToHosts = ::navigateBack,
              onInput = { input -> updateSession(core.sendTerminalInput(input)) },
              onReconnect = if (reconnectTarget == null) null else ::reconnect,
              onDisconnect = { updateSession(core.disconnectLiveShell()) },
              onResize = { cols, rows -> updateSession(core.resizeTerminal(cols, rows)) },
              onAttachRendererSurface = { surface, width, height -> core.attachAndroidSurface(surface, width, height) },
              onSetRendererOverlay = { overlay -> core.setRendererOverlayJson(overlay) },
              onRenderRendererSurface = { width, height, firstRow, rowCount ->
                core.renderRendererSurfaceFrame(width, height, firstRow, rowCount)
              },
              onDetachRendererSurface = { core.detachRendererSurface() },
              onClearTerminal = { updateSession(core.clearTerminal()) },
              onResetTerminal = { updateSession(core.resetTerminal()) },
            )
          }
        AppScreen.Codex ->
          Box(outgoingScreenModifier) {
            CodexScreen(
              snapshot = codexSnapshot,
              onBackToHosts = ::navigateBack,
              onRootBackProgress = { progress, edge ->
                topLevelBackProgress = progress
                topLevelBackEdge = edge
              },
              onSendMessage = { message ->
                updateCodexSnapshot(core.sendCodexMessage(message))
              },
              onUpdateSettings = { model, reasoningEffort, serviceTier, approvalPolicy, sandbox ->
                updateCodexSnapshot(
                  core.updateCodexSettings(
                    model,
                    reasoningEffort,
                    serviceTier,
                    approvalPolicy,
                    sandbox,
                  ),
                )
              },
              onBrowseDirectory = { path ->
                updateCodexSnapshot(withContext(Dispatchers.IO) { core.browseCodexDirectory(path) })
              },
              onListThreads = { cwd, searchTerm, cursor, archived, append ->
                updateCodexSnapshot(
                  withContext(Dispatchers.IO) {
                    core.listCodexThreadsPage(cwd, searchTerm, cursor, archived, append)
                  },
                )
              },
              onStartThread = { cwd ->
                codexReconnectTarget = codexReconnectTarget?.withCwd(cwd)?.withThreadId(null)
                updateCodexSnapshot(withContext(Dispatchers.IO) { core.startCodexThread(cwd) })
              },
              onStartThreadAndSend = { cwd, message ->
                codexReconnectTarget = codexReconnectTarget?.withCwd(cwd)?.withThreadId(null)
                val started = withContext(Dispatchers.IO) { core.startCodexThread(cwd) }
                updateCodexSnapshot(started)
                if (started.threadId != null && started.operation.lastError == null) {
                  updateCodexSnapshot(core.sendCodexMessage(message))
                }
              },
              onResumeThread = { threadId ->
                updateCodexSnapshot(withContext(Dispatchers.IO) { core.resumeCodexThread(threadId) })
                codexSnapshot.cwd?.let { cwd ->
                  codexReconnectTarget = codexReconnectTarget?.withCwd(cwd)
                }
              },
              onReadThread = { threadId ->
                updateCodexSnapshot(withContext(Dispatchers.IO) { core.readCodexThread(threadId) })
              },
              onLoadMoreThreadTurns = { threadId, cursor ->
                updateCodexSnapshot(withContext(Dispatchers.IO) { core.loadMoreCodexThreadTurns(threadId, cursor) })
              },
              onRenameThread = { threadId, name ->
                updateCodexSnapshot(withContext(Dispatchers.IO) { core.renameCodexThread(threadId, name) })
              },
              onArchiveThread = { threadId ->
                updateCodexSnapshot(withContext(Dispatchers.IO) { core.archiveCodexThread(threadId) })
              },
              onUnarchiveThread = { threadId ->
                updateCodexSnapshot(withContext(Dispatchers.IO) { core.unarchiveCodexThread(threadId) })
              },
              onDeleteThread = { threadId ->
                updateCodexSnapshot(withContext(Dispatchers.IO) { core.deleteCodexThread(threadId) })
              },
              onForkThread = { threadId, cwd ->
                updateCodexSnapshot(withContext(Dispatchers.IO) { core.forkCodexThread(threadId, cwd) })
                codexSnapshot.cwd?.let { nextCwd ->
                  codexReconnectTarget = codexReconnectTarget?.withCwd(nextCwd)
                }
              },
              onInterruptTurn = {
                updateCodexSnapshot(core.interruptCodexTurn())
              },
              onApprovalDecision = { requestId, decision ->
                updateCodexSnapshot(core.answerCodexApproval(requestId, decision))
              },
              onDisconnect = {
                updateCodexSnapshot(core.disconnectCodex())
              },
              onReconnect = if (codexReconnectTarget == null) null else ::reconnectCodex,
            )
          }
        AppScreen.Claude ->
          Box(outgoingScreenModifier) {
            CodexScreen(
              snapshot = claudeSnapshot,
              onBackToHosts = ::navigateBack,
              onRootBackProgress = { progress, edge ->
                topLevelBackProgress = progress
                topLevelBackEdge = edge
              },
              onSendMessage = { message -> updateClaudeSnapshot(core.sendClaudeMessage(message)) },
              onUpdateSettings = { model, _, _, approvalPolicy, _ ->
                updateClaudeSnapshot(core.updateClaudeSettings(model, approvalPolicy))
              },
              onBrowseDirectory = { _ -> Unit },
              onListThreads = { _, _, _, _, _ -> Unit },
              onStartThread = { cwd -> startFreshClaude(cwd) },
              onStartThreadAndSend = { cwd, message -> startFreshClaude(cwd, message) },
              onResumeThread = { _ -> Unit },
              onReadThread = { _ -> Unit },
              onLoadMoreThreadTurns = { _, _ -> Unit },
              onRenameThread = { _, _ -> Unit },
              onArchiveThread = { _ -> Unit },
              onUnarchiveThread = { _ -> Unit },
              onDeleteThread = { _ -> Unit },
              onForkThread = { _, _ -> Unit },
              onInterruptTurn = { updateClaudeSnapshot(core.interruptClaudeTurn()) },
              onApprovalDecision = { requestId, decision ->
                updateClaudeSnapshot(core.answerClaudeApproval(requestId, decision))
              },
              onDisconnect = { updateClaudeSnapshot(core.disconnectClaude()) },
              onReconnect = if (claudeReconnectTarget == null) null else ::reconnectClaude,
            )
          }
        AppScreen.Hosts -> Unit
        AppScreen.Settings ->
          Box(outgoingScreenModifier) {
            SettingsScreen(
              report = session.integration,
              displaySettings = displaySettings,
              onBack = ::navigateBack,
              onDisplaySettingsChange = { updated ->
                if (updated.terminalTheme != displaySettings.terminalTheme) {
                  core.setTerminalTheme(updated.terminalTheme.wire)
                }
                displaySettings = updated
              },
            )
          }
      }

      passwordPrompt?.let { request ->
        PasswordPromptDialog(
          request = request,
          secretStore = secretStore,
          onDismiss = { passwordPrompt = null },
          onConnect = { password ->
            passwordPrompt = null
            startPasswordConnection(request.profile, password, request.mode)
          },
        )
      }

      connectionNotice?.let { notice ->
        AlertDialog(
          onDismissRequest = { connectionNotice = null },
          containerColor = ShellowColors.PanelBackground,
          title = { Text(notice.title) },
          text = { Text(notice.message) },
          confirmButton = {
            TextButton(onClick = { connectionNotice = null }) { Text("OK") }
          },
        )
      }

      pendingHostKeyTrust?.let { pending ->
        AlertDialog(
          onDismissRequest = { pendingHostKeyTrust = null },
          title = { Text("Trust SSH host key?") },
          text = {
            Text("Verify this SHA-256 fingerprint with the server administrator before continuing:\n\n${pending.fingerprint}")
          },
          confirmButton = {
            TextButton(
              onClick = {
                pendingHostKeyTrust = null
                when (pending.mode) {
                  HostConnectMode.Terminal -> {
                    val target = reconnectTarget ?: return@TextButton
                    val updated = target.profile().copy(trustedHostKeySha256 = pending.fingerprint)
                    updateStoredProfile(updated)
                    reconnectTarget = target.withProfile(updated)
                    reconnect()
                  }
                  HostConnectMode.Codex -> {
                    val target = codexReconnectTarget ?: return@TextButton
                    val updated = target.profile().copy(trustedHostKeySha256 = pending.fingerprint)
                    updateStoredProfile(updated)
                    codexReconnectTarget = target.withProfile(updated)
                    reconnectCodex()
                  }
                  HostConnectMode.Claude -> {
                    val target = claudeReconnectTarget ?: return@TextButton
                    val updated = target.profile().copy(trustedHostKeySha256 = pending.fingerprint)
                    updateStoredProfile(updated)
                    claudeReconnectTarget = target.withProfile(updated)
                    reconnectClaude()
                  }
                }
              },
            ) { Text("Trust and connect") }
          },
          dismissButton = {
            TextButton(
              onClick = {
                pendingHostKeyTrust = null
                when (pending.mode) {
                  HostConnectMode.Terminal -> {
                    reconnectTarget = null
                    updateSession(core.disconnectLiveShell())
                  }
                  HostConnectMode.Codex -> {
                    codexReconnectTarget = null
                    updateCodexSnapshot(core.disconnectCodex())
                  }
                  HostConnectMode.Claude -> {
                    claudeReconnectTarget = null
                    updateClaudeSnapshot(core.disconnectClaude())
                  }
                }
              },
            ) { Text("Cancel") }
          },
        )
      }

      codexBootstrapError?.let { error ->
        AlertDialog(
          onDismissRequest = { codexBootstrapError = null },
          title = { Text("Could not enable remote Codex") },
          text = { Text(error) },
          confirmButton = {
            TextButton(onClick = { codexBootstrapError = null }) { Text("OK") }
          },
        )
      }

      session.detectedRemotePorts.firstOrNull()?.let { port ->
        AlertDialog(
          onDismissRequest = { updateSession(core.dismissDetectedRemotePort(port)) },
          title = { Text("New remote port detected") },
          text = {
            Text(
              "The remote host started listening on port $port. " +
                "Shellow has not exposed or forwarded this port.",
            )
          },
          confirmButton = {
            TextButton(onClick = { updateSession(core.dismissDetectedRemotePort(port)) }) {
              Text("Got it")
            }
          },
        )
      }
    }
  }
}

@Composable
private fun CodexScreen(
  snapshot: CodexSnapshot,
  onBackToHosts: () -> Unit,
  onRootBackProgress: (Float, Int) -> Unit,
  onSendMessage: (String) -> Unit,
  onUpdateSettings: (String, String, String, String, String) -> Unit,
  onBrowseDirectory: suspend (String) -> Unit,
  onListThreads: suspend (String, String, String, Boolean, Boolean) -> Unit,
  onStartThread: suspend (String) -> Unit,
  onStartThreadAndSend: suspend (String, String) -> Unit,
  onResumeThread: suspend (String) -> Unit,
  onReadThread: suspend (String) -> Unit,
  onLoadMoreThreadTurns: suspend (String, String) -> Unit,
  onRenameThread: suspend (String, String) -> Unit,
  onArchiveThread: suspend (String) -> Unit,
  onUnarchiveThread: suspend (String) -> Unit,
  onDeleteThread: suspend (String) -> Unit,
  onForkThread: suspend (String, String) -> Unit,
  onInterruptTurn: () -> Unit,
  onApprovalDecision: (String, String) -> Unit,
  onDisconnect: () -> Unit,
  onReconnect: (() -> Unit)?,
) {
  var draft by remember { mutableStateOf("") }
  var selectedPath by remember { mutableStateOf("") }
  var historySearch by remember { mutableStateOf("") }
  var homeRoute by remember { mutableStateOf(CodexHomeRoute.Overview) }
  var draftReturnRoute by remember { mutableStateOf(CodexHomeRoute.Overview) }
  var threadReturnRoute by remember { mutableStateOf(CodexHomeRoute.Overview) }
  var threadReturnScope by remember { mutableStateOf(CodexHistoryScope.AllProjects) }
  var isShowingThread by remember { mutableStateOf(false) }
  var historyScope by remember { mutableStateOf(CodexHistoryScope.AllProjects) }
  var showArchivedThreads by remember { mutableStateOf(false) }
  var didLoadProjectState by remember { mutableStateOf(false) }
  var showSettings by remember { mutableStateOf(false) }
  var showSessionSwitcher by remember { mutableStateOf(false) }
  var showDirectoryPicker by remember { mutableStateOf(false) }
  var settingsModel by remember { mutableStateOf("") }
  var settingsReasoningEffort by remember { mutableStateOf("") }
  var settingsServiceTier by remember { mutableStateOf("") }
  var settingsApprovalPolicy by remember { mutableStateOf("") }
  var settingsSandbox by remember { mutableStateOf("") }
  var renameTarget by remember { mutableStateOf<CodexThreadSummary?>(null) }
  var renameText by remember { mutableStateOf("") }
  var deleteTarget by remember { mutableStateOf<CodexThreadSummary?>(null) }
  var openingThreadId by remember { mutableStateOf<String?>(null) }
  var isStartingDraftThread by remember { mutableStateOf(false) }
  var codexActionsExpanded by remember { mutableStateOf(false) }
  var chatAutoFollow by remember { mutableStateOf(true) }
  var nestedBackProgress by remember { mutableStateOf(0f) }
  var nestedBackEdge by remember { mutableStateOf(BackEventCompat.EDGE_LEFT) }
  val listState = rememberLazyListState()
  val density = LocalDensity.current
  val imeBottomInset = WindowInsets.ime.getBottom(density)
  val scope = rememberCoroutineScope()
  val selectedProjectPath = selectedPath.trim()
  val historyCwd = if (historyScope == CodexHistoryScope.CurrentProject) selectedProjectPath else ""
  val canSend =
    snapshot.status == CodexStatus.Connected &&
      snapshot.threadId != null &&
      draft.trim().isNotEmpty()
  val canSendInitialDraft =
    snapshot.status == CodexStatus.Connected &&
      !isStartingDraftThread &&
      selectedProjectPath.isNotEmpty() &&
      draft.trim().isNotEmpty()
  val canUseProjectActions =
    snapshot.status == CodexStatus.Connected && selectedProjectPath.isNotEmpty()
  val canUseHistoryActions =
    snapshot.status == CodexStatus.Connected &&
      (historyScope == CodexHistoryScope.AllProjects || selectedProjectPath.isNotEmpty())
  val modelOptions =
    remember(snapshot.settings.availableModels, snapshot.settings.model) {
      codexModelPickerOptions(snapshot.settings.availableModels, snapshot.settings.model.orEmpty())
    }
  val settingsCanApply =
    settingsModel.trim() != snapshot.settings.model.orEmpty().trim() ||
      settingsReasoningEffort != snapshot.settings.reasoningEffort.orEmpty() ||
      settingsServiceTier != snapshot.settings.serviceTier.orEmpty() ||
      settingsApprovalPolicy != snapshot.settings.approvalPolicy.orEmpty() ||
      settingsSandbox != snapshot.settings.sandbox.orEmpty()
  val showCodexSettings = {
    settingsModel = snapshot.settings.model.orEmpty()
    settingsReasoningEffort = snapshot.settings.reasoningEffort.orEmpty()
    settingsServiceTier = snapshot.settings.serviceTier.orEmpty()
    settingsApprovalPolicy = snapshot.settings.approvalPolicy.orEmpty()
    settingsSandbox = snapshot.settings.sandbox.orEmpty()
    showSettings = true
  }
  val codexSessionThreads =
    remember(snapshot.threads.threads, snapshot.threadDetail.thread) {
      buildList {
        snapshot.threadDetail.thread?.let { add(it) }
        snapshot.threads.threads.forEach { thread ->
          if (none { it.id == thread.id }) add(thread)
        }
      }
    }
  val visibleChatMessages =
    remember(snapshot.messages) {
      snapshot.messages.filter { it.isVisibleInChat }.distinctBy { it.id }
    }

  val chatScrollSignature =
    remember(visibleChatMessages, snapshot.pendingApprovals, snapshot.turnActive) {
      codexChatScrollSignature(visibleChatMessages, snapshot.pendingApprovals.size, snapshot.turnActive)
    }

  val chatItemCount =
    snapshot.pendingApprovals.size +
      visibleChatMessages.size +
      1
  val isAtChatBottom by remember(listState, chatItemCount) {
    derivedStateOf {
      listState.layoutInfo.visibleItemsInfo.lastOrNull()?.index?.let { it >= chatItemCount - 1 } ?: true
    }
  }

  LaunchedEffect(snapshot.threadId, chatScrollSignature, chatAutoFollow, imeBottomInset) {
    if (snapshot.threadId != null && chatAutoFollow) {
      delay(80)
      listState.scrollToItem(chatItemCount - 1)
    }
  }

  LaunchedEffect(listState.isScrollInProgress, isAtChatBottom) {
    if (isAtChatBottom) {
      chatAutoFollow = true
    } else if (listState.isScrollInProgress) {
      chatAutoFollow = false
    }
  }

  LaunchedEffect(snapshot.cwd, snapshot.projects.current, snapshot.projects.recent) {
    if (selectedPath.trim().isEmpty()) {
      selectedPath =
        snapshot.projects.current
          ?: snapshot.cwd
          ?: snapshot.projects.recent.firstOrNull()
          ?: ""
    }
  }

  LaunchedEffect(snapshot.settings) {
    settingsModel = snapshot.settings.model.orEmpty()
    settingsReasoningEffort = snapshot.settings.reasoningEffort.orEmpty()
    settingsServiceTier = snapshot.settings.serviceTier.orEmpty()
    settingsApprovalPolicy = snapshot.settings.approvalPolicy.orEmpty()
    settingsSandbox = snapshot.settings.sandbox.orEmpty()
  }

  LaunchedEffect(snapshot.status, snapshot.threadId) {
    draft = ""
    chatAutoFollow = true
    if (snapshot.status != CodexStatus.Connected) {
      didLoadProjectState = false
    } else if (snapshot.threadId == null && !didLoadProjectState) {
      didLoadProjectState = true
      val path =
        snapshot.projects.current
          ?: snapshot.cwd
          ?: snapshot.projects.recent.firstOrNull()
          ?: selectedPath
      if (path.trim().isNotEmpty()) {
        selectedPath = path
      }
      onListThreads("", historySearch, "", showArchivedThreads, false)
    }
    if (
      snapshot.threadId != null &&
        (homeRoute == CodexHomeRoute.Draft || isShowingThread)
    ) {
      isShowingThread = true
    } else if (snapshot.status == CodexStatus.Connected) {
      isShowingThread = false
    }
  }

  val returnToThreadOrigin: () -> Unit = {
    isShowingThread = false
    draft = ""
    chatAutoFollow = true
    when (threadReturnRoute) {
      CodexHomeRoute.Project -> {
        if (selectedProjectPath.isNotEmpty()) {
          homeRoute = CodexHomeRoute.Project
          historyScope = CodexHistoryScope.CurrentProject
          scope.launch { onListThreads(selectedProjectPath, historySearch, "", showArchivedThreads, false) }
        } else {
          homeRoute = CodexHomeRoute.Overview
          historyScope = CodexHistoryScope.AllProjects
          scope.launch { onListThreads("", historySearch, "", showArchivedThreads, false) }
        }
      }
      CodexHomeRoute.Overview -> {
        homeRoute = CodexHomeRoute.Overview
        historyScope = threadReturnScope
        val cwd = if (threadReturnScope == CodexHistoryScope.CurrentProject) selectedProjectPath else ""
        scope.launch { onListThreads(cwd, historySearch, "", showArchivedThreads, false) }
      }
      CodexHomeRoute.Draft -> {
        homeRoute = draftReturnRoute
      }
    }
  }

  val handleCodexBack: () -> Unit = {
    if (isShowingThread) {
      returnToThreadOrigin()
    } else {
      when (homeRoute) {
        CodexHomeRoute.Overview -> onBackToHosts()
        CodexHomeRoute.Project -> {
          homeRoute = CodexHomeRoute.Overview
          historyScope = CodexHistoryScope.AllProjects
          scope.launch { onListThreads("", historySearch, "", showArchivedThreads, false) }
        }
        CodexHomeRoute.Draft -> {
          draft = ""
          if (draftReturnRoute == CodexHomeRoute.Project && selectedProjectPath.isNotEmpty()) {
            homeRoute = CodexHomeRoute.Project
            historyScope = CodexHistoryScope.CurrentProject
            scope.launch { onListThreads(selectedProjectPath, historySearch, "", showArchivedThreads, false) }
          } else {
            homeRoute = CodexHomeRoute.Overview
            historyScope = CodexHistoryScope.AllProjects
            scope.launch { onListThreads("", historySearch, "", showArchivedThreads, false) }
          }
        }
      }
    }
  }

  PredictiveBackHandler { events ->
    val returnsToHosts = !isShowingThread && homeRoute == CodexHomeRoute.Overview
    try {
      events.collect { event ->
        if (returnsToHosts) {
          onRootBackProgress(event.progress, event.swipeEdge)
        } else {
          nestedBackProgress = event.progress
          nestedBackEdge = event.swipeEdge
        }
      }
      handleCodexBack()
    } finally {
      if (returnsToHosts) {
        onRootBackProgress(0f, BackEventCompat.EDGE_LEFT)
      } else {
        nestedBackProgress = 0f
      }
    }
  }

  if (showDirectoryPicker) {
    CodexDirectoryPickerDialog(
      snapshot = snapshot,
      selectedPath = selectedProjectPath,
      onOpenDirectory = { path ->
        scope.launch { onBrowseDirectory(path) }
      },
      onSelectDirectory = { path ->
        selectedPath = path
        showDirectoryPicker = false
      },
      onDismiss = { showDirectoryPicker = false },
    )
  }

  if (showSettings) {
    CodexSettingsDialog(
      model = settingsModel,
      modelOptions = modelOptions,
      isLoadingModels = snapshot.settings.isLoadingModels,
      modelsError = snapshot.settings.modelsError,
      reasoningEffort = settingsReasoningEffort,
      serviceTier = settingsServiceTier,
      approvalPolicy = settingsApprovalPolicy,
      sandbox = settingsSandbox,
      canApply = settingsCanApply,
      onModelChange = {
        settingsModel = it
        settingsReasoningEffort = ""
        settingsServiceTier = ""
      },
      onReasoningEffortChange = { settingsReasoningEffort = it },
      onServiceTierChange = { settingsServiceTier = it },
      onApprovalPolicyChange = { settingsApprovalPolicy = it },
      onSandboxChange = { settingsSandbox = it },
      onDismiss = { showSettings = false },
      onApply = {
        onUpdateSettings(
          settingsModel.trim(),
          settingsReasoningEffort,
          settingsServiceTier,
          settingsApprovalPolicy,
          settingsSandbox,
        )
        showSettings = false
      },
    )
  }

  if (showSessionSwitcher) {
    CodexSessionSwitcherDialog(
      profileName = snapshot.title,
      threads = codexSessionThreads,
      selectedThreadId = snapshot.threadId.takeIf { isShowingThread },
      pendingApprovalCount = snapshot.pendingApprovals.size,
      loading = snapshot.threads.isLoading,
      errorMessage = snapshot.threads.error,
      onDismiss = { showSessionSwitcher = false },
      onRefresh = {
        scope.launch { onListThreads("", "", "", false, false) }
      },
      onNewConversation = {
        showSessionSwitcher = false
        draftReturnRoute = homeRoute
        draft = ""
        chatAutoFollow = true
        homeRoute = CodexHomeRoute.Draft
        isShowingThread = false
      },
      onResume = { thread ->
        showSessionSwitcher = false
        threadReturnRoute = homeRoute
        threadReturnScope = historyScope
        openingThreadId = thread.id
        draft = ""
        chatAutoFollow = true
        scope.launch {
          onResumeThread(thread.id)
          isShowingThread = true
          if (openingThreadId == thread.id) openingThreadId = null
        }
      },
    )
  }

  renameTarget?.let { thread ->
    AlertDialog(
      onDismissRequest = { renameTarget = null },
      containerColor = ShellowColors.PanelBackground,
      titleContentColor = ShellowColors.TerminalText,
      textContentColor = ShellowColors.TerminalText,
      title = { Text("Rename Thread") },
      text = {
        OutlinedTextField(
          value = renameText,
          onValueChange = { renameText = it },
          label = { Text("Name") },
          singleLine = true,
        )
      },
      confirmButton = {
        TextButton(
          onClick = {
            val nextName = renameText.trim()
            scope.launch { onRenameThread(thread.id, nextName) }
            renameTarget = null
          },
        ) { Text("Save") }
      },
      dismissButton = {
        TextButton(onClick = { renameTarget = null }) { Text("Cancel") }
      },
    )
  }

  deleteTarget?.let { thread ->
    AlertDialog(
      onDismissRequest = { deleteTarget = null },
      containerColor = ShellowColors.PanelBackground,
      titleContentColor = ShellowColors.TerminalText,
      textContentColor = ShellowColors.TerminalText,
      title = { Text("Delete thread?") },
      text = { Text(thread.displayTitle) },
      confirmButton = {
        TextButton(
          onClick = {
            scope.launch { onDeleteThread(thread.id) }
            deleteTarget = null
          },
        ) { Text("Delete") }
      },
      dismissButton = {
        TextButton(onClick = { deleteTarget = null }) { Text("Cancel") }
      },
    )
  }

  Column(
    modifier =
      Modifier
        .fillMaxSize()
        .predictiveBackTransform(nestedBackProgress, nestedBackEdge)
        .background(ShellowColors.TerminalBackground),
  ) {
    Column(
      modifier = Modifier.fillMaxWidth().padding(horizontal = 14.dp, vertical = 10.dp),
    ) {
      Row(
        modifier = Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(8.dp),
      ) {
        NavigationBackButton(onClick = handleCodexBack)
        Column(
          Modifier
            .weight(1f)
            .clickable(enabled = snapshot.status == CodexStatus.Connected) {
              showSessionSwitcher = true
              scope.launch { onListThreads("", "", "", false, false) }
            },
        ) {
          Text(
            codexHeaderTitle(snapshot, homeRoute, selectedProjectPath, isShowingThread),
            color = ShellowColors.TerminalText,
            style = MaterialTheme.typography.titleMedium,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
          )
          Text(
            buildString {
              append(codexHeaderSubtitle(snapshot, homeRoute, selectedProjectPath, isShowingThread))
              if (snapshot.status == CodexStatus.Connected) {
                append(" · ${codexSessionThreads.size} ${if (codexSessionThreads.size == 1) "session" else "sessions"} ▾")
              }
            },
            color = ShellowColors.TerminalMuted,
            style = MaterialTheme.typography.labelSmall,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
          )
        }
        Box {
          OverflowMenuButton(
            contentDescription = "Codex Actions",
            onClick = { codexActionsExpanded = true },
          )
          DropdownMenu(
            expanded = codexActionsExpanded,
            onDismissRequest = { codexActionsExpanded = false },
          ) {
            snapshot.threadId?.let { threadId ->
              val cursor = snapshot.threadDetail.turnsNextCursor.orEmpty()
              if (cursor.isNotEmpty()) {
                DropdownMenuItem(
                  text = { Text("Load More History") },
                  enabled = !snapshot.threadDetail.isLoadingMore,
                  onClick = {
                    codexActionsExpanded = false
                    scope.launch { onLoadMoreThreadTurns(threadId, cursor) }
                  },
                )
              }
              DropdownMenuItem(
                text = { Text("Fork Thread") },
                onClick = {
                  codexActionsExpanded = false
                  scope.launch { onForkThread(threadId, selectedProjectPath.ifBlank { snapshot.cwd.orEmpty() }) }
                },
              )
            }
            if (!isShowingThread && homeRoute == CodexHomeRoute.Project) {
              DropdownMenuItem(
                text = { Text(if (showArchivedThreads) "Hide Archived" else "Show Archived") },
                onClick = {
                  codexActionsExpanded = false
                  val nextArchived = !showArchivedThreads
                  showArchivedThreads = nextArchived
                  if (canUseProjectActions) {
                    scope.launch { onListThreads(selectedProjectPath, historySearch, "", nextArchived, false) }
                  }
                },
              )
              DropdownMenuItem(
                text = { Text("Refresh") },
                enabled = canUseProjectActions,
                onClick = {
                  codexActionsExpanded = false
                  if (canUseProjectActions) {
                    scope.launch { onListThreads(selectedProjectPath, historySearch, "", showArchivedThreads, false) }
                  }
                },
              )
            }
            DropdownMenuItem(
              text = { Text("Settings") },
              onClick = {
                codexActionsExpanded = false
                showCodexSettings()
              },
            )
            if (snapshot.status == CodexStatus.Disconnected || snapshot.status == CodexStatus.Failed) onReconnect?.let {
              DropdownMenuItem(
                text = { Text("Reconnect") },
                onClick = {
                  codexActionsExpanded = false
                  it()
                },
              )
            }
            if (snapshot.status != CodexStatus.Disconnected) {
              DropdownMenuItem(
                text = { Text("Disconnect") },
                onClick = {
                  codexActionsExpanded = false
                  onDisconnect()
                },
              )
            }
          }
        }
      }
    }

    (snapshot.operation.lastError ?: snapshot.lastError)?.let { error ->
      CodexInlineStatusRow(
        text = error,
        tone = CodexInlineStatusTone.Warning,
        modifier = Modifier.padding(horizontal = 14.dp),
      )
    } ?: snapshot.operation.lastSuccess
      ?.takeIf { (!isShowingThread || snapshot.threadId == null) && !it.isRoutineCodexOperationSuccess() }
      ?.let { message ->
      CodexInlineStatusRow(
        text = message,
        tone = CodexInlineStatusTone.Success,
        modifier = Modifier.padding(horizontal = 14.dp),
      )
    }

    CodexUsageSummary(snapshot.usage)

    if (!isShowingThread || snapshot.threadId == null) {
      val refreshCurrentHistory = {
        if (canUseHistoryActions) {
          scope.launch { onListThreads(historyCwd, historySearch, "", showArchivedThreads, false) }
        }
      }
      val openThread: (String) -> Unit = { threadId ->
        threadReturnRoute = homeRoute
        threadReturnScope = historyScope
        openingThreadId = threadId
        draft = ""
        chatAutoFollow = true
        scope.launch {
          onResumeThread(threadId)
          isShowingThread = true
          if (openingThreadId == threadId) {
            openingThreadId = null
          }
        }
      }
      val beginDraftChat = {
        draftReturnRoute = homeRoute
        draft = ""
        chatAutoFollow = true
        homeRoute = CodexHomeRoute.Draft
      }
      val sendInitialDraft = {
        val message = draft.trim()
        val path = selectedProjectPath
        if (message.isNotEmpty() && path.isNotEmpty() && !isStartingDraftThread) {
          threadReturnRoute = draftReturnRoute
          threadReturnScope = historyScope
          draft = ""
          chatAutoFollow = true
          isStartingDraftThread = true
          scope.launch {
            try {
              onStartThreadAndSend(path, message)
              isShowingThread = true
            } finally {
              isStartingDraftThread = false
            }
          }
        }
      }

      when (homeRoute) {
        CodexHomeRoute.Overview ->
          CodexProjectHistoryPanel(
            snapshot = snapshot,
            selectedPath = selectedPath,
            historySearch = historySearch,
            onHistorySearchChange = { historySearch = it },
            historyScope = historyScope,
            onHistoryScopeChange = { nextScope ->
              historyScope = nextScope
              val nextCwd = if (nextScope == CodexHistoryScope.CurrentProject) selectedProjectPath else ""
              if (snapshot.status == CodexStatus.Connected && (nextScope == CodexHistoryScope.AllProjects || selectedProjectPath.isNotEmpty())) {
                scope.launch { onListThreads(nextCwd, historySearch, "", showArchivedThreads, false) }
              }
            },
            showArchivedThreads = showArchivedThreads,
            onToggleArchivedThreads = {
              val nextArchived = !showArchivedThreads
              showArchivedThreads = nextArchived
              if (canUseHistoryActions) {
                scope.launch { onListThreads(historyCwd, historySearch, "", nextArchived, false) }
              }
            },
            onSelectProject = { path ->
              selectedPath = path
              historyScope = CodexHistoryScope.CurrentProject
              homeRoute = CodexHomeRoute.Project
              scope.launch {
                onListThreads(path, historySearch, "", showArchivedThreads, false)
              }
            },
            canUseHistoryActions = canUseHistoryActions,
            onRefreshHistory = refreshCurrentHistory,
            onLoadMoreHistory = { cursor ->
              if (canUseHistoryActions) {
                scope.launch { onListThreads(historyCwd, historySearch, cursor, showArchivedThreads, true) }
              }
            },
            onStartThread = beginDraftChat,
            onResumeThread = openThread,
            onReadThread = { threadId -> scope.launch { onReadThread(threadId) } },
            onRenameThread = { thread ->
              renameTarget = thread
              renameText = thread.displayTitle
            },
            onArchiveThread = { threadId -> scope.launch { onArchiveThread(threadId) } },
            onUnarchiveThread = { threadId -> scope.launch { onUnarchiveThread(threadId) } },
            onDeleteThread = { thread -> deleteTarget = thread },
            onForkThread = { thread ->
              scope.launch { onForkThread(thread.id, selectedProjectPath.ifBlank { thread.cwd }) }
            },
            openingThreadId = openingThreadId,
            modifier = Modifier.weight(1f),
          )

        CodexHomeRoute.Project ->
          CodexProjectThreadsPanel(
            snapshot = snapshot,
            selectedPath = selectedProjectPath,
            historySearch = historySearch,
            onHistorySearchChange = { historySearch = it },
            showArchivedThreads = showArchivedThreads,
            onRefreshHistory = {
              if (canUseProjectActions) {
                scope.launch { onListThreads(selectedProjectPath, historySearch, "", showArchivedThreads, false) }
              }
            },
            onLoadMoreHistory = { cursor ->
              if (canUseProjectActions) {
                scope.launch { onListThreads(selectedProjectPath, historySearch, cursor, showArchivedThreads, true) }
              }
            },
            onStartThread = beginDraftChat,
            onResumeThread = openThread,
            onRenameThread = { thread ->
              renameTarget = thread
              renameText = thread.displayTitle
            },
            onArchiveThread = { threadId -> scope.launch { onArchiveThread(threadId) } },
            onUnarchiveThread = { threadId -> scope.launch { onUnarchiveThread(threadId) } },
            onDeleteThread = { thread -> deleteTarget = thread },
            onForkThread = { thread ->
              scope.launch { onForkThread(thread.id, selectedProjectPath.ifBlank { thread.cwd }) }
            },
            openingThreadId = openingThreadId,
            modifier = Modifier.weight(1f),
          )

        CodexHomeRoute.Draft ->
          CodexDraftChatPanel(
            selectedPath = selectedPath,
            draft = draft,
            onDraftChange = { draft = it },
            canSend = canSendInitialDraft,
            isStarting = isStartingDraftThread,
            onSend = sendInitialDraft,
            onChooseDirectory = {
              if (snapshot.status == CodexStatus.Connected && selectedProjectPath.isNotEmpty()) {
                showDirectoryPicker = true
                scope.launch { onBrowseDirectory(selectedProjectPath) }
              }
            },
            modifier = Modifier.weight(1f),
          )
      }
    } else {
      Box(modifier = Modifier.weight(1f).fillMaxWidth()) {
        LazyColumn(
          modifier = Modifier.fillMaxSize().padding(horizontal = 12.dp),
          state = listState,
          verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
          items(snapshot.pendingApprovals, key = { "approval-${it.requestId}" }) { approval ->
            CodexApprovalCard(
              approval = approval,
              onDecision = { decision -> onApprovalDecision(approval.requestId, decision) },
            )
          }

          items(visibleChatMessages, key = { it.id }) { message ->
            CodexMessageBubble(message)
          }

          item("thread-bottom") {
            Spacer(modifier = Modifier.height(1.dp))
          }
        }

        if (!chatAutoFollow && !isAtChatBottom) {
          TextButton(
            onClick = {
              chatAutoFollow = true
              scope.launch { listState.animateScrollToItem(chatItemCount - 1) }
            },
            modifier =
              Modifier
                .align(Alignment.BottomEnd)
                .padding(12.dp)
                .background(ShellowColors.Accent, RoundedCornerShape(18.dp)),
          ) {
            Text("Latest", color = ComposeColor.White, fontWeight = FontWeight.SemiBold)
          }
        }
      }

      Column(
        modifier =
          Modifier
            .fillMaxWidth()
            .imePadding()
            .padding(horizontal = 12.dp, vertical = 8.dp),
        verticalArrangement = Arrangement.spacedBy(4.dp),
      ) {
        if (snapshot.turnActive) {
          CodexTurnStatusRow(onStop = onInterruptTurn)
        }

        Row(
          verticalAlignment = Alignment.Bottom,
          horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
          CodexMessageInput(
            value = draft,
            onValueChange = { draft = it },
            placeholder = if (snapshot.turnActive) "Steer Codex" else "Message Codex",
            isActiveTurn = snapshot.turnActive,
            modifier = Modifier.weight(1f),
          )
          if (canSend) {
            TextButton(
              onClick = {
                val message = draft.trim()
                if (message.isNotEmpty()) {
                  draft = ""
                  chatAutoFollow = true
                  scope.launch { onSendMessage(message) }
                }
              },
            ) {
              Text(if (snapshot.turnActive) "Steer" else "Send", fontWeight = FontWeight.SemiBold)
            }
          }
        }
      }
    }
  }
}

private data class CodexUsageMetricValue(
  val id: String,
  val title: String,
  val value: String,
  val detail: String,
  val progress: Float?,
)

@Composable
private fun CodexUsageSummary(usage: CodexUsageState) {
  val metrics = remember(usage) { codexUsageMetrics(usage) }
  if (metrics.isEmpty()) return

  Row(
    modifier =
      Modifier
        .fillMaxWidth()
        .horizontalScroll(rememberScrollState())
        .background(ShellowColors.PanelBackground.copy(alpha = 0.72f))
        .padding(horizontal = 14.dp, vertical = 8.dp),
    horizontalArrangement = Arrangement.spacedBy(10.dp),
  ) {
    metrics.forEach { metric ->
      Column(
        modifier = Modifier.width(116.dp).semantics(mergeDescendants = true) {},
        verticalArrangement = Arrangement.spacedBy(3.dp),
      ) {
        Text(
          metric.title,
          color = ShellowColors.TerminalMuted,
          style = MaterialTheme.typography.labelSmall,
          maxLines = 1,
        )
        Text(
          metric.value,
          color = ShellowColors.TerminalText,
          style = MaterialTheme.typography.labelMedium,
          fontWeight = FontWeight.SemiBold,
          maxLines = 1,
        )
        metric.progress?.let { progress ->
          LinearProgressIndicator(
            progress = { progress },
            modifier = Modifier.width(104.dp).height(3.dp),
            color = if (progress >= 0.9f) ShellowColors.Warning else ShellowColors.Accent,
            trackColor = ShellowColors.KeyBackground,
          )
        }
        Text(
          metric.detail,
          color = ShellowColors.TerminalMuted,
          style = MaterialTheme.typography.labelSmall,
          maxLines = 1,
          overflow = TextOverflow.Ellipsis,
        )
      }
    }
  }
}

private fun codexUsageMetrics(usage: CodexUsageState): List<CodexUsageMetricValue> =
  buildList {
    usage.thread?.let { thread ->
      val used = thread.last.totalTokens.coerceAtLeast(0)
      val window = thread.modelContextWindow?.takeIf { it > 0 }
      val progress = window?.let { (used.toDouble() / it.toDouble()).coerceIn(0.0, 1.0).toFloat() }
      add(
        CodexUsageMetricValue(
          id = "context",
          title = "Context",
          value = window?.let { "${compactCodexCount(used)} / ${compactCodexCount(it)}" } ?: compactCodexCount(used),
          detail = progress?.let { "${(it * 100).roundToInt()}% used" } ?: "Latest turn",
          progress = progress,
        ),
      )
    }

    usage.rateLimits?.let { limits ->
      limits.primary?.let { add(codexRateLimitMetric(it, "Primary limit", "primary")) }
      limits.secondary?.let { add(codexRateLimitMetric(it, "Secondary limit", "secondary")) }
      limits.credits?.takeIf { it.unlimited || it.balance != null }?.let { credits ->
        add(
          CodexUsageMetricValue(
            id = "credits",
            title = "Credits",
            value = if (credits.unlimited) "Unlimited" else credits.balance.orEmpty().ifEmpty { "—" },
            detail = if (credits.hasCredits || credits.unlimited) "Available" else "Depleted",
            progress = null,
          ),
        )
      }
      limits.individualLimit?.let { spend ->
        add(
          CodexUsageMetricValue(
            id = "spend",
            title = "Spend limit",
            value = "${spend.used} / ${spend.limit}",
            detail = codexResetLabel(spend.resetsAt),
            progress = ((100 - spend.remainingPercent.coerceIn(0, 100)) / 100f),
          ),
        )
      }
    }
  }

private fun codexRateLimitMetric(
  window: CodexRateLimitWindow,
  fallbackTitle: String,
  id: String,
): CodexUsageMetricValue {
  val used = window.usedPercent.coerceIn(0, 100)
  return CodexUsageMetricValue(
    id = id,
    title = window.windowDurationMins?.let(::codexLimitTitle) ?: fallbackTitle,
    value = "${100 - used}% left",
    detail = window.resetsAt?.let(::codexResetLabel) ?: "Reset time unavailable",
    progress = used / 100f,
  )
}

private fun codexLimitTitle(minutes: Long): String =
  when {
    minutes == 300L -> "5h limit"
    minutes == 10_080L -> "Weekly limit"
    minutes >= 1_440L && minutes % 1_440L == 0L -> "${minutes / 1_440L}d limit"
    minutes >= 60L && minutes % 60L == 0L -> "${minutes / 60L}h limit"
    else -> "${minutes}m limit"
  }

private fun compactCodexCount(value: Long): String =
  when {
    value >= 1_000_000L -> String.format(Locale.getDefault(), "%.1fM", value / 1_000_000.0)
    value >= 1_000L -> String.format(Locale.getDefault(), "%.1fK", value / 1_000.0)
    else -> value.toString()
  }

private fun codexResetLabel(timestampSeconds: Long): String {
  val formatted = SimpleDateFormat("MMM d, HH:mm", Locale.getDefault()).format(Date(timestampSeconds * 1_000))
  return "Resets $formatted"
}

private enum class CodexHomeRoute {
  Overview,
  Project,
  Draft,
}

private enum class CodexHistoryScope {
  CurrentProject,
  AllProjects,
}

private fun codexModelPickerOptions(
  options: List<CodexModelOption>,
  current: String,
): List<CodexModelOption> {
  val normalized = normalizeCodexModel(current)
  return if (normalized != null && options.none { it.id == normalized }) {
    options + CodexModelOption(normalized, normalized)
  } else {
    options
  }
}

private fun normalizeCodexModel(value: String?): String? =
  value?.trim()?.takeIf { it.isNotEmpty() }

private fun codexHeaderSubtitle(
  snapshot: CodexSnapshot,
  homeRoute: CodexHomeRoute,
  selectedProjectPath: String,
  isShowingThread: Boolean,
): String {
  val location =
    if (!isShowingThread && (homeRoute == CodexHomeRoute.Project || homeRoute == CodexHomeRoute.Draft) && selectedProjectPath.isNotBlank()) {
      codexCompactPath(selectedProjectPath)
    } else {
      snapshot.cwd?.takeIf { it.isNotBlank() }?.let(::lastPathComponent) ?: snapshot.endpoint
    }
  return "${snapshot.status.title} · $location"
}

private fun codexHeaderTitle(
  snapshot: CodexSnapshot,
  homeRoute: CodexHomeRoute,
  selectedProjectPath: String,
  isShowingThread: Boolean,
): String =
  if (isShowingThread && snapshot.threadId != null) {
    snapshot.threadDetail.thread?.displayTitle ?: snapshot.title
  } else {
    when (homeRoute) {
      CodexHomeRoute.Overview -> snapshot.title
      CodexHomeRoute.Project -> selectedProjectPath.takeIf { it.isNotBlank() }?.let(::lastPathComponent) ?: snapshot.title
      CodexHomeRoute.Draft -> "New Conversation"
    }
  }

@Composable
private fun CodexProjectHistoryPanel(
  snapshot: CodexSnapshot,
  selectedPath: String,
  historySearch: String,
  onHistorySearchChange: (String) -> Unit,
  historyScope: CodexHistoryScope,
  onHistoryScopeChange: (CodexHistoryScope) -> Unit,
  showArchivedThreads: Boolean,
  onToggleArchivedThreads: () -> Unit,
  onSelectProject: (String) -> Unit,
  canUseHistoryActions: Boolean,
  onRefreshHistory: () -> Unit,
  onLoadMoreHistory: (String) -> Unit,
  onStartThread: () -> Unit,
  onResumeThread: (String) -> Unit,
  onReadThread: (String) -> Unit,
  onRenameThread: (CodexThreadSummary) -> Unit,
  onArchiveThread: (String) -> Unit,
  onUnarchiveThread: (String) -> Unit,
  onDeleteThread: (CodexThreadSummary) -> Unit,
  onForkThread: (CodexThreadSummary) -> Unit,
  openingThreadId: String?,
  modifier: Modifier = Modifier,
) {
  val selectedProjectPath = selectedPath.trim()
  val homeSearchTerm = historySearch.trim()
  val knownProjectPaths =
    mergeProjects(
      snapshot.projects.recent,
      listOfNotNull(snapshot.projects.current, snapshot.cwd),
    )
  val visibleProjectPaths = knownProjectPaths.filter { matchesHomeSearch(it, homeSearchTerm) }
  val visibleThreads =
    snapshot.threads.threads.filter { thread ->
      homeSearchTerm.isBlank() ||
        matchesHomeSearch(thread.displayTitle, homeSearchTerm) ||
        matchesHomeSearch(thread.preview, homeSearchTerm) ||
        matchesHomeSearch(thread.cwd, homeSearchTerm)
    }

  Column(modifier = modifier.fillMaxWidth()) {
    LazyColumn(
      modifier = Modifier.weight(1f).fillMaxWidth().padding(horizontal = 12.dp),
      verticalArrangement = Arrangement.spacedBy(18.dp),
    ) {
      item("home-search") {
        CodexSearchBarRow(
          searchValue = historySearch,
          onSearchValueChange = onHistorySearchChange,
          searchPlaceholder = "Search projects or sessions",
          onSearch = onRefreshHistory,
          newConversationEnabled = snapshot.status == CodexStatus.Connected,
          onNewConversation = onStartThread,
        )
      }

      item("projects") {
        CodexProjectsSection(
          snapshot = snapshot,
          visibleProjectPaths = visibleProjectPaths,
          homeSearchTerm = homeSearchTerm,
          onSelectProject = onSelectProject,
          selectedProjectPath = selectedProjectPath,
        )
      }

      item("recent-conversations") {
        CodexRecentConversationsSection(
          snapshot = snapshot,
          visibleThreads = visibleThreads,
          homeSearchTerm = homeSearchTerm,
          historyScope = historyScope,
          onHistoryScopeChange = onHistoryScopeChange,
          showArchivedThreads = showArchivedThreads,
          onToggleArchivedThreads = onToggleArchivedThreads,
          onRefreshHistory = onRefreshHistory,
          onLoadMoreHistory = onLoadMoreHistory,
          onResumeThread = onResumeThread,
          onReadThread = onReadThread,
          onRenameThread = onRenameThread,
          onArchiveThread = onArchiveThread,
          onUnarchiveThread = onUnarchiveThread,
          onDeleteThread = onDeleteThread,
          onForkThread = onForkThread,
          openingThreadId = openingThreadId,
          canRefresh = canUseHistoryActions,
        )
      }
    }
  }
}

@Composable
private fun CodexProjectThreadsPanel(
  snapshot: CodexSnapshot,
  selectedPath: String,
  historySearch: String,
  onHistorySearchChange: (String) -> Unit,
  showArchivedThreads: Boolean,
  onRefreshHistory: () -> Unit,
  onLoadMoreHistory: (String) -> Unit,
  onStartThread: () -> Unit,
  onResumeThread: (String) -> Unit,
  onRenameThread: (CodexThreadSummary) -> Unit,
  onArchiveThread: (String) -> Unit,
  onUnarchiveThread: (String) -> Unit,
  onDeleteThread: (CodexThreadSummary) -> Unit,
  onForkThread: (CodexThreadSummary) -> Unit,
  openingThreadId: String?,
  modifier: Modifier = Modifier,
) {
  val homeSearchTerm = historySearch.trim()
  val visibleThreads =
    snapshot.threads.threads.filter { thread ->
      homeSearchTerm.isBlank() ||
        matchesHomeSearch(thread.displayTitle, homeSearchTerm) ||
        matchesHomeSearch(thread.preview, homeSearchTerm) ||
        matchesHomeSearch(thread.cwd, homeSearchTerm)
    }

  Column(modifier = modifier.fillMaxWidth()) {
    LazyColumn(
      modifier = Modifier.weight(1f).fillMaxWidth().padding(horizontal = 12.dp),
      verticalArrangement = Arrangement.spacedBy(10.dp),
    ) {
      item("project-search") {
        CodexSearchBarRow(
          searchValue = historySearch,
          onSearchValueChange = onHistorySearchChange,
          searchPlaceholder = "Search this project",
          onSearch = onRefreshHistory,
          newConversationEnabled = selectedPath.isNotBlank(),
          onNewConversation = onStartThread,
        )
      }

      item("project-conversations-header") {
        CodexSectionHeader(title = if (showArchivedThreads) "Archived Conversations" else "Conversations")
      }

      if (snapshot.threads.isLoading) {
        item("project-loading") {
          CodexInlineStatusRow(text = "Loading history", isLoading = true)
        }
      }
      snapshot.threads.error?.let { error ->
        item("project-error") {
          CodexInlineStatusRow(text = error, tone = CodexInlineStatusTone.Warning)
        }
      }
      items(visibleThreads, key = { it.id }) { thread ->
        CodexThreadRow(
          thread = thread,
          archived = showArchivedThreads,
          isOpening = openingThreadId == thread.id,
          showProjectContext = false,
          onResume = { onResumeThread(thread.id) },
          onRename = { onRenameThread(thread) },
          onArchive = { onArchiveThread(thread.id) },
          onUnarchive = { onUnarchiveThread(thread.id) },
          onDelete = { onDeleteThread(thread) },
          onFork = { onForkThread(thread) },
        )
      }
      snapshot.threads.nextCursor?.let { cursor ->
        if (homeSearchTerm.isBlank()) {
          item("project-load-more") {
            CodexLoadMoreButton(
              isLoading = snapshot.threads.isLoadingMore,
              onClick = { onLoadMoreHistory(cursor) },
              modifier = Modifier.fillMaxWidth(),
            )
          }
        }
      }
      if (visibleThreads.isEmpty() && !snapshot.threads.isLoading && snapshot.threads.error == null) {
        item("project-empty") {
          CodexEmptyState(
            title =
              if (homeSearchTerm.isBlank()) {
                if (showArchivedThreads) "No Archived Conversations" else "No Conversations"
              } else {
                "No Matches"
              },
            detail =
              if (homeSearchTerm.isBlank()) {
                if (showArchivedThreads) "Archived conversations will appear here." else "Start a chat in this project when you're ready."
              } else {
                "Try a different search."
              },
          )
        }
      }
    }
  }
}

@Composable
private fun CodexNewConversationPrompt(
  directoryName: String?,
  onChooseDirectory: () -> Unit,
  modifier: Modifier = Modifier,
) {
  Box(
    modifier = modifier.padding(horizontal = 24.dp),
    contentAlignment = Alignment.Center,
  ) {
    if (directoryName.isNullOrBlank()) {
      Text(
        "What should we build?",
        color = ShellowColors.TerminalText,
        style = MaterialTheme.typography.titleLarge,
        fontWeight = FontWeight.SemiBold,
        textAlign = TextAlign.Center,
      )
    } else {
      Row(
        modifier = Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.Center,
      ) {
        Text(
          "What should we build in ",
          color = ShellowColors.TerminalText,
          style = MaterialTheme.typography.titleLarge,
          fontWeight = FontWeight.SemiBold,
          maxLines = 1,
        )
        Text(
          directoryName,
          modifier =
            Modifier
              .weight(1f, fill = false)
              .clickable(onClick = onChooseDirectory)
              .semantics { contentDescription = "Choose directory, current directory $directoryName" },
          color = ShellowColors.Accent,
          style = MaterialTheme.typography.titleLarge,
          fontWeight = FontWeight.SemiBold,
          textDecoration = TextDecoration.Underline,
          maxLines = 1,
          overflow = TextOverflow.Ellipsis,
        )
        Text(
          "?",
          color = ShellowColors.TerminalText,
          style = MaterialTheme.typography.titleLarge,
          fontWeight = FontWeight.SemiBold,
        )
      }
    }
  }
}

@Composable
private fun CodexDraftChatPanel(
  selectedPath: String,
  draft: String,
  onDraftChange: (String) -> Unit,
  canSend: Boolean,
  isStarting: Boolean,
  onSend: () -> Unit,
  onChooseDirectory: () -> Unit,
  modifier: Modifier = Modifier,
) {
  Column(modifier = modifier.fillMaxWidth()) {
    CodexNewConversationPrompt(
      directoryName = selectedPath.trim().takeIf { it.isNotEmpty() }?.let(::lastPathComponent),
      onChooseDirectory = onChooseDirectory,
      modifier = Modifier.weight(1f).fillMaxWidth(),
    )

    Row(
      modifier =
        Modifier
          .fillMaxWidth()
          .imePadding()
          .background(ShellowColors.PanelBackground)
          .padding(12.dp),
      verticalAlignment = Alignment.Bottom,
      horizontalArrangement = Arrangement.spacedBy(8.dp),
    ) {
      CodexMessageInput(
        value = draft,
        onValueChange = onDraftChange,
        modifier = Modifier.weight(1f),
      )
      if (canSend || isStarting) {
        TextButton(onClick = onSend, enabled = canSend) {
          Text(if (isStarting) "Starting" else "Send", fontWeight = FontWeight.SemiBold)
        }
      }
    }
  }
}

@Composable
private fun CodexDirectoryPickerDialog(
  snapshot: CodexSnapshot,
  selectedPath: String,
  onOpenDirectory: (String) -> Unit,
  onSelectDirectory: (String) -> Unit,
  onDismiss: () -> Unit,
) {
  val currentPath = snapshot.directory.path?.trim().orEmpty().ifBlank { selectedPath.trim() }
  var searchQuery by remember { mutableStateOf("") }
  var showHidden by remember { mutableStateOf(false) }
  val quickPaths = remember(snapshot.projects.favorites, snapshot.projects.recent) {
    (snapshot.projects.favorites + snapshot.projects.recent).distinct().take(5)
  }

  AlertDialog(
    onDismissRequest = onDismiss,
    containerColor = ShellowColors.PanelBackground,
    titleContentColor = ShellowColors.TerminalText,
    textContentColor = ShellowColors.TerminalText,
    title = { Text("Choose Directory") },
    text = {
      Column(
        modifier = Modifier.fillMaxWidth(),
        verticalArrangement = Arrangement.spacedBy(8.dp),
      ) {
        if (currentPath.isNotEmpty()) {
          Text(
            currentPath,
            color = ShellowColors.TerminalMuted,
            style = MaterialTheme.typography.labelSmall,
            fontFamily = FontFamily.Monospace,
            maxLines = 2,
            overflow = TextOverflow.Ellipsis,
          )
        }
        if (quickPaths.isNotEmpty()) {
          Row(
            modifier = Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()),
            horizontalArrangement = Arrangement.spacedBy(6.dp),
          ) {
            quickPaths.forEach { path ->
              TextButton(onClick = { onOpenDirectory(path) }) {
                Text(lastPathComponent(path), maxLines = 1)
              }
            }
          }
        }
        CodexInlineTextField(
          value = searchQuery,
          onValueChange = { searchQuery = it },
          placeholder = "Search folders",
          imeAction = ImeAction.Search,
          onSubmit = {},
          modifier = Modifier.fillMaxWidth(),
        )
        Row(verticalAlignment = Alignment.CenterVertically) {
          Checkbox(checked = showHidden, onCheckedChange = { showHidden = it })
          Text("Show hidden folders", style = MaterialTheme.typography.bodySmall)
        }
        CodexDirectoryList(
          snapshot = snapshot,
          onOpenDirectory = onOpenDirectory,
          searchQuery = searchQuery,
          showHidden = showHidden,
          modifier = Modifier.fillMaxWidth().heightIn(min = 180.dp, max = 420.dp),
        )
      }
    },
    confirmButton = {
      TextButton(
        onClick = { onSelectDirectory(currentPath) },
        enabled = currentPath.isNotEmpty() && !snapshot.directory.isLoading,
      ) {
        Text("Choose")
      }
    },
    dismissButton = {
      TextButton(onClick = onDismiss) {
        Text("Cancel")
      }
    },
  )
}

@Composable
private fun CodexProjectsSection(
  snapshot: CodexSnapshot,
  visibleProjectPaths: List<String>,
  homeSearchTerm: String,
  onSelectProject: (String) -> Unit,
  selectedProjectPath: String,
) {
  Column(
    modifier = Modifier.fillMaxWidth().padding(top = 12.dp),
    verticalArrangement = Arrangement.spacedBy(10.dp),
  ) {
    CodexSectionHeader(title = "Projects")

    visibleProjectPaths.forEach { path ->
      CodexDirectoryRow(lastPathComponent(path), path) { onSelectProject(path) }
    }

    if (
      visibleProjectPaths.isEmpty() &&
        !snapshot.threads.isLoading
    ) {
      CodexEmptyState(
        title = if (homeSearchTerm.isBlank()) "No Projects" else "No Matches",
        detail = if (homeSearchTerm.isBlank()) "Start a chat to enter a workspace path." else "Try a different search.",
      )
    }
  }
}

@Composable
private fun CodexRecentConversationsSection(
  snapshot: CodexSnapshot,
  visibleThreads: List<CodexThreadSummary>,
  homeSearchTerm: String,
  historyScope: CodexHistoryScope,
  onHistoryScopeChange: (CodexHistoryScope) -> Unit,
  showArchivedThreads: Boolean,
  onToggleArchivedThreads: () -> Unit,
  onRefreshHistory: () -> Unit,
  onLoadMoreHistory: (String) -> Unit,
  onResumeThread: (String) -> Unit,
  onReadThread: (String) -> Unit,
  onRenameThread: (CodexThreadSummary) -> Unit,
  onArchiveThread: (String) -> Unit,
  onUnarchiveThread: (String) -> Unit,
  onDeleteThread: (CodexThreadSummary) -> Unit,
  onForkThread: (CodexThreadSummary) -> Unit,
  openingThreadId: String?,
  canRefresh: Boolean,
) {
  var recentActionsExpanded by remember { mutableStateOf(false) }

  Column(
    modifier = Modifier.fillMaxWidth().padding(bottom = 12.dp),
    verticalArrangement = Arrangement.spacedBy(10.dp),
  ) {
    Row(
      modifier = Modifier.fillMaxWidth(),
      horizontalArrangement = Arrangement.spacedBy(10.dp),
      verticalAlignment = Alignment.CenterVertically,
    ) {
      CodexSectionHeader(
        title = if (showArchivedThreads) "Archived Sessions" else "Recent Sessions",
        detail = if (historyScope == CodexHistoryScope.CurrentProject) "Current project" else null,
        modifier = Modifier.weight(1f),
      )

      Box {
        OverflowMenuButton(
          contentDescription = "Session Actions",
          onClick = { recentActionsExpanded = true },
        )
        DropdownMenu(
          expanded = recentActionsExpanded,
          onDismissRequest = { recentActionsExpanded = false },
        ) {
          DropdownMenuItem(
            text = { Text("Current Project") },
            onClick = {
              recentActionsExpanded = false
              onHistoryScopeChange(CodexHistoryScope.CurrentProject)
            },
          )
          DropdownMenuItem(
            text = { Text("All Projects") },
            onClick = {
              recentActionsExpanded = false
              onHistoryScopeChange(CodexHistoryScope.AllProjects)
            },
          )
          DropdownMenuItem(
            text = { Text(if (showArchivedThreads) "Hide Archived" else "Show Archived") },
            onClick = {
              recentActionsExpanded = false
              onToggleArchivedThreads()
            },
          )
          DropdownMenuItem(
            text = { Text("Refresh") },
            enabled = canRefresh,
            onClick = {
              recentActionsExpanded = false
              onRefreshHistory()
            },
          )
        }
      }
    }

    if (snapshot.threads.isLoading) {
      CodexInlineStatusRow(text = "Loading history", isLoading = true)
    }
    snapshot.threads.error?.let { error ->
      CodexInlineStatusRow(text = error, tone = CodexInlineStatusTone.Warning)
    }
    visibleThreads.forEach { thread ->
      CodexThreadRow(
        thread = thread,
        archived = showArchivedThreads,
        isOpening = openingThreadId == thread.id,
        onResume = { onResumeThread(thread.id) },
        onRename = { onRenameThread(thread) },
        onArchive = { onArchiveThread(thread.id) },
        onUnarchive = { onUnarchiveThread(thread.id) },
        onDelete = { onDeleteThread(thread) },
        onFork = { onForkThread(thread) },
      )
    }
    snapshot.threads.nextCursor?.let { cursor ->
      if (homeSearchTerm.isBlank()) {
        CodexLoadMoreButton(
          isLoading = snapshot.threads.isLoadingMore,
          onClick = { onLoadMoreHistory(cursor) },
          modifier = Modifier.fillMaxWidth(),
        )
      }
    }
    if (visibleThreads.isEmpty() && !snapshot.threads.isLoading && snapshot.threads.error == null) {
      CodexEmptyState(
        title =
          if (homeSearchTerm.isBlank()) {
            if (showArchivedThreads) "No Archived Sessions" else "No Recent Sessions"
          } else {
            "No Matches"
          },
        detail =
          if (homeSearchTerm.isBlank()) {
            if (showArchivedThreads) "Archived sessions will appear here." else "Start a chat from a project to see it here."
          } else {
            "Try a different search."
          },
      )
    }
  }
}

@Composable
private fun CodexLoadMoreButton(
  isLoading: Boolean,
  onClick: () -> Unit,
  modifier: Modifier = Modifier,
) {
  Row(
    modifier =
      modifier
        .clickable(enabled = !isLoading, onClick = onClick)
        .padding(vertical = 8.dp),
    horizontalArrangement = Arrangement.Center,
    verticalAlignment = Alignment.CenterVertically,
  ) {
    Text(
      if (isLoading) "Loading" else "Load More",
      color = if (isLoading) ShellowColors.TerminalMuted else ShellowColors.Accent,
      style = MaterialTheme.typography.labelMedium,
    )
  }
}

@Composable
private fun CodexMessageInput(
  value: String,
  onValueChange: (String) -> Unit,
  modifier: Modifier = Modifier,
  placeholder: String = "Message Codex",
  isActiveTurn: Boolean = false,
) {
  val inputShape = RoundedCornerShape(8.dp)
  val inputBackground =
    if (isActiveTurn) ShellowColors.Accent.copy(alpha = 0.08f) else ShellowColors.KeyBackground
  val inputStroke =
    if (isActiveTurn) ShellowColors.Accent.copy(alpha = 0.28f) else ComposeColor.Transparent

  BasicTextField(
    value = value,
    onValueChange = onValueChange,
    modifier =
      modifier
        .heightIn(min = 40.dp, max = 132.dp)
        .background(inputBackground, inputShape)
        .border(1.dp, inputStroke, inputShape)
        .padding(horizontal = 10.dp, vertical = 8.dp)
        .semantics { contentDescription = placeholder },
    textStyle = MaterialTheme.typography.bodyMedium.copy(color = ShellowColors.TerminalText),
    singleLine = false,
    minLines = 1,
    maxLines = 5,
    decorationBox = { innerTextField ->
      Box(modifier = Modifier.fillMaxWidth(), contentAlignment = Alignment.CenterStart) {
        if (value.isEmpty()) {
          Text(
            placeholder,
            color = ShellowColors.TerminalMuted,
            style = MaterialTheme.typography.bodyMedium,
          )
        }
        innerTextField()
      }
    },
  )
}

@Composable
private fun CodexSearchBarRow(
  searchValue: String,
  onSearchValueChange: (String) -> Unit,
  searchPlaceholder: String,
  onSearch: () -> Unit,
  newConversationEnabled: Boolean,
  onNewConversation: () -> Unit,
) {
  Row(
    modifier = Modifier.fillMaxWidth(),
    horizontalArrangement = Arrangement.spacedBy(8.dp),
    verticalAlignment = Alignment.CenterVertically,
  ) {
    CodexSearchField(
      value = searchValue,
      onValueChange = onSearchValueChange,
      modifier = Modifier.weight(1f),
      placeholder = searchPlaceholder,
      onSearch = onSearch,
    )
    CodexNewConversationButton(
      enabled = newConversationEnabled,
      onClick = onNewConversation,
    )
  }
}

@Composable
private fun CodexSearchField(
  value: String,
  onValueChange: (String) -> Unit,
  placeholder: String,
  onSearch: () -> Unit,
  modifier: Modifier = Modifier,
) {
  CodexInlineTextField(
    value = value,
    onValueChange = onValueChange,
    placeholder = placeholder,
    imeAction = ImeAction.Search,
    onSubmit = onSearch,
    modifier = modifier,
  )
}

private enum class CodexInlineStatusTone {
  Neutral,
  Success,
  Warning,
}

@Composable
private fun CodexInlineStatusRow(
  text: String,
  modifier: Modifier = Modifier,
  tone: CodexInlineStatusTone = CodexInlineStatusTone.Neutral,
  isLoading: Boolean = false,
) {
  val color =
    when (tone) {
      CodexInlineStatusTone.Neutral -> ShellowColors.TerminalMuted
      CodexInlineStatusTone.Success -> ShellowColors.Success
      CodexInlineStatusTone.Warning -> ShellowColors.Warning
    }
  Row(
    modifier = modifier.fillMaxWidth().padding(horizontal = 4.dp, vertical = 6.dp),
    horizontalArrangement = Arrangement.spacedBy(8.dp),
    verticalAlignment = Alignment.CenterVertically,
  ) {
    if (isLoading) {
      CircularProgressIndicator(
        modifier = Modifier.size(14.dp),
        strokeWidth = 2.dp,
        color = ShellowColors.TerminalMuted,
      )
    }
    Text(text, color = color, style = MaterialTheme.typography.bodySmall, maxLines = 2, overflow = TextOverflow.Ellipsis)
  }
}

@Composable
private fun CodexTurnStatusRow(onStop: () -> Unit) {
  Row(
    modifier = Modifier.fillMaxWidth().padding(horizontal = 4.dp, vertical = 1.dp).semantics {
      liveRegion = LiveRegionMode.Polite
    },
    horizontalArrangement = Arrangement.spacedBy(6.dp),
    verticalAlignment = Alignment.CenterVertically,
  ) {
    CircularProgressIndicator(
      modifier = Modifier.size(13.dp),
      strokeWidth = 2.dp,
      color = ShellowColors.TerminalMuted,
    )
    Text(
      "Working",
      color = ShellowColors.TerminalMuted,
      modifier = Modifier.weight(1f),
      style = MaterialTheme.typography.labelSmall,
    )
    TextButton(onClick = onStop, modifier = Modifier.semantics { contentDescription = "Interrupt Codex Turn" }) {
      Text("Stop", color = ShellowColors.Warning, style = MaterialTheme.typography.labelSmall, fontWeight = FontWeight.SemiBold)
    }
  }
}

@Composable
private fun CodexInlineTextField(
  value: String,
  onValueChange: (String) -> Unit,
  placeholder: String,
  imeAction: ImeAction,
  onSubmit: () -> Unit,
  modifier: Modifier = Modifier,
) {
  BasicTextField(
    value = value,
    onValueChange = onValueChange,
    modifier =
      modifier
        .heightIn(min = 40.dp)
        .background(ShellowColors.KeyBackground, RoundedCornerShape(8.dp))
        .padding(horizontal = 10.dp, vertical = 8.dp)
        .semantics { contentDescription = placeholder },
    textStyle = MaterialTheme.typography.bodyMedium.copy(color = ShellowColors.TerminalText),
    singleLine = true,
    keyboardOptions = KeyboardOptions(imeAction = imeAction),
    keyboardActions = KeyboardActions(
      onSearch = { onSubmit() },
      onGo = { onSubmit() },
      onDone = { onSubmit() },
    ),
    decorationBox = { innerTextField ->
      Box(modifier = Modifier.fillMaxWidth(), contentAlignment = Alignment.CenterStart) {
        if (value.isEmpty()) {
          Text(
            placeholder,
            color = ShellowColors.TerminalMuted,
            style = MaterialTheme.typography.bodyMedium,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
          )
        }
        innerTextField()
      }
    },
  )
}

@Composable
private fun CodexEmptyState(
  title: String,
  detail: String,
) {
  Column(
    modifier =
      Modifier
        .fillMaxWidth()
        .padding(horizontal = 14.dp, vertical = 18.dp),
    verticalArrangement = Arrangement.spacedBy(4.dp),
    horizontalAlignment = Alignment.CenterHorizontally,
  ) {
    Text(
      title,
      color = ShellowColors.TerminalText,
      style = MaterialTheme.typography.bodyMedium,
      fontWeight = FontWeight.SemiBold,
    )
    Text(
      detail,
      color = ShellowColors.TerminalMuted,
      style = MaterialTheme.typography.bodySmall,
    )
  }
}

@Composable
private fun CodexNewConversationButton(
  enabled: Boolean,
  onClick: () -> Unit,
) {
  IconButton(
    onClick = onClick,
    enabled = enabled,
    modifier =
      Modifier
        .background(
          ShellowColors.KeyBackground,
          RoundedCornerShape(8.dp),
        )
        .semantics { contentDescription = "New Conversation" },
  ) {
    Text(
      "+",
      color = if (enabled) ShellowColors.Accent else ShellowColors.TerminalMuted,
      style = MaterialTheme.typography.titleMedium,
      fontWeight = FontWeight.SemiBold,
    )
  }
}

@Composable
private fun CodexSessionSwitcherDialog(
  profileName: String,
  threads: List<CodexThreadSummary>,
  selectedThreadId: String?,
  pendingApprovalCount: Int,
  loading: Boolean,
  errorMessage: String?,
  onDismiss: () -> Unit,
  onRefresh: () -> Unit,
  onNewConversation: () -> Unit,
  onResume: (CodexThreadSummary) -> Unit,
) {
  AlertDialog(
    onDismissRequest = onDismiss,
    containerColor = ShellowColors.PanelBackground,
    titleContentColor = ShellowColors.TerminalText,
    textContentColor = ShellowColors.TerminalText,
    title = { Text("Sessions") },
    text = {
      Column(
        modifier = Modifier.heightIn(max = 520.dp).verticalScroll(rememberScrollState()),
        verticalArrangement = Arrangement.spacedBy(10.dp),
      ) {
        Text("Codex on $profileName", color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.labelSmall)
        when {
          loading && threads.isEmpty() -> {
            Row(horizontalArrangement = Arrangement.spacedBy(10.dp), verticalAlignment = Alignment.CenterVertically) {
              CircularProgressIndicator(modifier = Modifier.size(18.dp), strokeWidth = 2.dp)
              Text("Loading conversations…", color = ShellowColors.TerminalMuted)
            }
          }
          threads.isEmpty() -> {
            Text(
              errorMessage ?: "No conversations yet.",
              color = ShellowColors.TerminalMuted,
              style = MaterialTheme.typography.bodySmall,
            )
          }
          else -> {
            threads.forEach { thread ->
              Row(
                modifier =
                  Modifier
                    .fillMaxWidth()
                    .background(ShellowColors.KeyBackground.copy(alpha = 0.42f), RoundedCornerShape(10.dp))
                    .clickable(enabled = thread.id != selectedThreadId) { onResume(thread) }
                    .padding(horizontal = 12.dp, vertical = 10.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(10.dp),
              ) {
                Text(
                  if (thread.id == selectedThreadId) "●" else "○",
                  color = if (thread.id == selectedThreadId) ShellowColors.Accent else ShellowColors.TerminalMuted,
                )
                Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                  Text(
                    thread.displayTitle,
                    color = ShellowColors.TerminalText,
                    fontWeight = FontWeight.SemiBold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                  )
                  Text(
                    codexCompactPath(thread.cwd),
                    color = ShellowColors.TerminalMuted,
                    style = MaterialTheme.typography.labelSmall,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                  )
                }
                thread.statusIndicator?.let { indicator ->
                  CodexThreadStatusBadge(indicator)
                }
              }
            }
          }
        }
      }
    },
    confirmButton = { TextButton(onClick = onDismiss) { Text("Done") } },
    dismissButton = {
      Row {
        TextButton(enabled = !loading, onClick = onRefresh) { Text("Refresh") }
        TextButton(onClick = onNewConversation) { Text("New") }
      }
    },
  )
}

@Composable
private fun NavigationBackButton(onClick: () -> Unit) {
  IconButton(
    onClick = onClick,
    modifier =
      Modifier
        .size(48.dp)
        .semantics { contentDescription = "Navigate back" },
  ) {
    Canvas(Modifier.size(24.dp)) {
      val direction = if (layoutDirection == LayoutDirection.Ltr) 1f else -1f
      val tipX = center.x - direction * 7.dp.toPx()
      val tailX = center.x + direction * 7.dp.toPx()
      val shoulderX = tipX + direction * 6.dp.toPx()
      val headOffset = 6.dp.toPx()
      val strokeWidth = 2.dp.toPx()

      drawLine(
        color = ShellowColors.TerminalText,
        start = Offset(tipX, center.y),
        end = Offset(tailX, center.y),
        strokeWidth = strokeWidth,
        cap = StrokeCap.Round,
      )
      drawLine(
        color = ShellowColors.TerminalText,
        start = Offset(tipX, center.y),
        end = Offset(shoulderX, center.y - headOffset),
        strokeWidth = strokeWidth,
        cap = StrokeCap.Round,
      )
      drawLine(
        color = ShellowColors.TerminalText,
        start = Offset(tipX, center.y),
        end = Offset(shoulderX, center.y + headOffset),
        strokeWidth = strokeWidth,
        cap = StrokeCap.Round,
      )
    }
  }
}

@Composable
private fun CodexForwardButton(
  contentDescription: String,
  enabled: Boolean,
  onClick: () -> Unit,
) {
  IconButton(
    onClick = onClick,
    enabled = enabled,
    modifier =
      Modifier
        .size(36.dp)
        .background(ShellowColors.KeyBackground, RoundedCornerShape(8.dp))
        .semantics { this.contentDescription = contentDescription },
  ) {
    Text(
      ">",
      color = if (enabled) ShellowColors.Accent else ShellowColors.TerminalMuted,
      style = MaterialTheme.typography.titleMedium,
      fontWeight = FontWeight.SemiBold,
    )
  }
}

@Composable
private fun OverflowMenuButton(
  contentDescription: String,
  onClick: () -> Unit,
) {
  IconButton(
    onClick = onClick,
    modifier = Modifier.semantics { this.contentDescription = contentDescription },
  ) {
    Text("...", color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.titleMedium)
  }
}

@Composable
private fun CodexSectionHeader(
  title: String,
  detail: String? = null,
  modifier: Modifier = Modifier,
) {
  Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(2.dp)) {
    Text(title, color = ShellowColors.TerminalText, style = MaterialTheme.typography.titleSmall)
    if (!detail.isNullOrBlank()) {
      Text(detail, color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.labelSmall, maxLines = 1, overflow = TextOverflow.Ellipsis)
    }
  }
}

@Composable
private fun CodexProjectChip(
  path: String,
  favorite: Boolean,
  onClick: () -> Unit,
) {
  Column(
    modifier =
      Modifier
        .clickable(onClick = onClick)
        .background(ShellowColors.PanelBackground, RoundedCornerShape(8.dp))
        .padding(horizontal = 10.dp, vertical = 7.dp),
    verticalArrangement = Arrangement.spacedBy(2.dp),
  ) {
    Text(
      if (favorite) "★ ${lastPathComponent(path)}" else lastPathComponent(path),
      color = ShellowColors.TerminalText,
      style = MaterialTheme.typography.labelMedium,
      maxLines = 1,
      overflow = TextOverflow.Ellipsis,
    )
    Text(path, color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.labelSmall, maxLines = 1)
  }
}

@Composable
private fun CodexDirectoryList(
  snapshot: CodexSnapshot,
  onOpenDirectory: (String) -> Unit,
  searchQuery: String,
  showHidden: Boolean,
  modifier: Modifier = Modifier,
) {
  LazyColumn(
    modifier = modifier.fillMaxWidth().padding(horizontal = 12.dp),
    verticalArrangement = Arrangement.spacedBy(8.dp),
  ) {
    if (snapshot.directory.isLoading) {
      item("directory-loading") {
        Text("Loading folders...", color = ShellowColors.TerminalMuted, modifier = Modifier.padding(8.dp))
      }
    }
    snapshot.directory.error?.let { error ->
      item("directory-error") {
        Text(error, color = ShellowColors.Warning, modifier = Modifier.padding(8.dp))
      }
    }
    snapshot.directory.parent?.let { parent ->
      item("parent") {
        CodexDirectoryRow("..", parent) { onOpenDirectory(parent) }
      }
    }
    val query = searchQuery.trim()
    val folders = snapshot.directory.entries.filter { entry ->
      entry.isDirectory &&
        (showHidden || !entry.name.startsWith(".")) &&
        (query.isEmpty() || entry.name.contains(query, ignoreCase = true))
    }
    items(folders, key = { it.path }) { entry ->
      CodexDirectoryRow(entry.name, entry.path) { onOpenDirectory(entry.path) }
    }
    if (folders.isEmpty() && !snapshot.directory.isLoading && snapshot.directory.error == null) {
      item("empty-folders") {
        Text("No folders", color = ShellowColors.TerminalMuted, modifier = Modifier.fillMaxWidth().padding(24.dp))
      }
    }
  }
}

@Composable
private fun CodexDirectoryRow(
  title: String,
  path: String,
  onClick: () -> Unit,
) {
  Column(
    modifier =
      Modifier
        .fillMaxWidth()
        .clickable(onClick = onClick)
        .padding(horizontal = 4.dp, vertical = 8.dp)
        .semantics { contentDescription = "$title, $path" },
    verticalArrangement = Arrangement.spacedBy(3.dp),
  ) {
    Text(title, color = ShellowColors.TerminalText, style = MaterialTheme.typography.bodyMedium, fontWeight = FontWeight.SemiBold)
    Text(
      codexCompactPath(path),
      color = ShellowColors.TerminalMuted,
      style = MaterialTheme.typography.labelSmall,
      maxLines = 1,
      overflow = TextOverflow.Ellipsis,
    )
  }
}

@Composable
private fun CodexHistoryList(
  snapshot: CodexSnapshot,
  historySearch: String,
  onHistorySearchChange: (String) -> Unit,
  historyScope: CodexHistoryScope,
  onHistoryScopeChange: (CodexHistoryScope) -> Unit,
  showArchivedThreads: Boolean,
  onToggleArchivedThreads: () -> Unit,
  onRefreshHistory: () -> Unit,
  onLoadMoreHistory: (String) -> Unit,
  onResumeThread: (String) -> Unit,
  onReadThread: (String) -> Unit,
  onRenameThread: (CodexThreadSummary) -> Unit,
  onArchiveThread: (String) -> Unit,
  onUnarchiveThread: (String) -> Unit,
  onDeleteThread: (CodexThreadSummary) -> Unit,
  onForkThread: (CodexThreadSummary) -> Unit,
  canRefresh: Boolean,
  modifier: Modifier = Modifier,
) {
  var historyActionsExpanded by remember(historyScope, showArchivedThreads) { mutableStateOf(false) }

  Column(modifier = modifier.fillMaxWidth()) {
    Column(
      modifier = Modifier.fillMaxWidth().padding(horizontal = 12.dp, vertical = 10.dp),
      verticalArrangement = Arrangement.spacedBy(10.dp),
    ) {
      Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(10.dp),
        verticalAlignment = Alignment.CenterVertically,
      ) {
        CodexSectionHeader(
          title = "History",
          detail = if (historyScope == CodexHistoryScope.CurrentProject) "Current project" else null,
          modifier = Modifier.weight(1f),
        )
        Box {
          OverflowMenuButton(
            contentDescription = "History Actions",
            onClick = { historyActionsExpanded = true },
          )
          DropdownMenu(
            expanded = historyActionsExpanded,
            onDismissRequest = { historyActionsExpanded = false },
          ) {
            DropdownMenuItem(
              text = { Text("Current Project") },
              onClick = {
                historyActionsExpanded = false
                onHistoryScopeChange(CodexHistoryScope.CurrentProject)
              },
            )
            DropdownMenuItem(
              text = { Text("All Projects") },
              onClick = {
                historyActionsExpanded = false
                onHistoryScopeChange(CodexHistoryScope.AllProjects)
              },
            )
            DropdownMenuItem(
              text = { Text(if (showArchivedThreads) "Hide Archived" else "Show Archived") },
              onClick = {
                historyActionsExpanded = false
                onToggleArchivedThreads()
              },
            )
            DropdownMenuItem(
              text = { Text("Refresh") },
              enabled = canRefresh,
              onClick = {
                historyActionsExpanded = false
                onRefreshHistory()
              },
            )
          }
        }
      }
      CodexSearchField(
        value = historySearch,
        onValueChange = onHistorySearchChange,
        placeholder = "Search conversations",
        onSearch = onRefreshHistory,
        modifier = Modifier.fillMaxWidth(),
      )
    }

    LazyColumn(
      modifier = Modifier.weight(1f).fillMaxWidth().padding(horizontal = 12.dp),
      verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
      if (snapshot.threads.isLoading) {
        item("history-loading") {
          CodexInlineStatusRow(text = "Loading history", isLoading = true)
        }
      }
      snapshot.threads.error?.let { error ->
        item("history-error") {
          CodexInlineStatusRow(text = error, tone = CodexInlineStatusTone.Warning)
        }
      }
      items(snapshot.threads.threads, key = { it.id }) { thread ->
        CodexThreadRow(
          thread = thread,
          archived = showArchivedThreads,
          onResume = { onResumeThread(thread.id) },
          onRename = { onRenameThread(thread) },
          onArchive = { onArchiveThread(thread.id) },
          onUnarchive = { onUnarchiveThread(thread.id) },
          onDelete = { onDeleteThread(thread) },
          onFork = { onForkThread(thread) },
        )
      }
      snapshot.threads.nextCursor?.let { cursor ->
        item("load-more-history") {
          CodexLoadMoreButton(
            isLoading = snapshot.threads.isLoadingMore,
            onClick = { onLoadMoreHistory(cursor) },
            modifier = Modifier.fillMaxWidth(),
          )
        }
      }
      if (snapshot.threads.threads.isEmpty() && !snapshot.threads.isLoading && snapshot.threads.error == null) {
        item("empty-history") {
          CodexEmptyState(
            title = if (historySearch.isBlank()) "No History" else "No Matches",
            detail = if (historySearch.isBlank()) "Conversations appear here after you start using Codex." else "Try a different search.",
          )
        }
      }
    }
  }
}

private enum class CodexThreadStatusIndicatorKind {
  Running,
  Approval,
  UserInput,
  Failed,
  SystemError,
}

private data class CodexThreadStatusIndicator(
  val kind: CodexThreadStatusIndicatorKind,
  val title: String,
  val accessibilityLabel: String,
)

private val CodexThreadSummary.statusIndicator: CodexThreadStatusIndicator?
  get() {
    if (status == "systemError") {
      return CodexThreadStatusIndicator(CodexThreadStatusIndicatorKind.SystemError, "Error", "Codex system error")
    }
    if (status == "active") {
      if (pendingApprovalCount > 0 || "waitingOnApproval" in activeFlags) {
        val count = pendingApprovalCount.coerceAtLeast(1)
        return CodexThreadStatusIndicator(
          CodexThreadStatusIndicatorKind.Approval,
          if (count > 1) "Approval $count" else "Approval",
          "$count pending Codex approval${if (count == 1) "" else "s"}",
        )
      }
      if ("waitingOnUserInput" in activeFlags) {
        return CodexThreadStatusIndicator(CodexThreadStatusIndicatorKind.UserInput, "Reply needed", "Codex needs a reply")
      }
      return CodexThreadStatusIndicator(CodexThreadStatusIndicatorKind.Running, "Running", "Codex is running")
    }
    if (pendingApprovalCount > 0 || "waitingOnApproval" in activeFlags) {
      val count = pendingApprovalCount.coerceAtLeast(1)
      return CodexThreadStatusIndicator(
        CodexThreadStatusIndicatorKind.Approval,
        if (count > 1) "Approval $count" else "Approval",
        "$count pending Codex approval${if (count == 1) "" else "s"}",
      )
    }
    if ("waitingOnUserInput" in activeFlags) {
      return CodexThreadStatusIndicator(CodexThreadStatusIndicatorKind.UserInput, "Reply needed", "Codex needs a reply")
    }
    if (lastTurnStatus == "failed") {
      return CodexThreadStatusIndicator(
        CodexThreadStatusIndicatorKind.Failed,
        "Failed",
        lastTurnError?.let { "Codex failed: $it" } ?: "Codex failed",
      )
    }
    if (lastTurnStatus == "inProgress") {
      return CodexThreadStatusIndicator(CodexThreadStatusIndicatorKind.Running, "Running", "Codex is running")
    }
    return null
  }

@Composable
private fun CodexThreadStatusBadge(indicator: CodexThreadStatusIndicator) {
  val tint =
    when (indicator.kind) {
      CodexThreadStatusIndicatorKind.Running -> ShellowColors.Accent
      CodexThreadStatusIndicatorKind.Approval,
      CodexThreadStatusIndicatorKind.UserInput -> ShellowColors.Warning
      CodexThreadStatusIndicatorKind.Failed,
      CodexThreadStatusIndicatorKind.SystemError -> MaterialTheme.colorScheme.error
    }
  Row(
    modifier =
      Modifier
        .background(tint.copy(alpha = 0.13f), RoundedCornerShape(12.dp))
        .semantics { contentDescription = indicator.accessibilityLabel }
        .padding(horizontal = 7.dp, vertical = 4.dp),
    horizontalArrangement = Arrangement.spacedBy(4.dp),
    verticalAlignment = Alignment.CenterVertically,
  ) {
    if (indicator.kind == CodexThreadStatusIndicatorKind.Running) {
      CircularProgressIndicator(
        modifier = Modifier.size(10.dp),
        color = tint,
        strokeWidth = 1.5.dp,
      )
    }
    Text(
      indicator.title,
      color = tint,
      style = MaterialTheme.typography.labelSmall,
      fontWeight = FontWeight.SemiBold,
      maxLines = 1,
    )
  }
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
private fun CodexThreadRow(
  thread: CodexThreadSummary,
  archived: Boolean,
  isOpening: Boolean = false,
  showProjectContext: Boolean = true,
  onResume: () -> Unit,
  onRename: () -> Unit,
  onArchive: () -> Unit,
  onUnarchive: () -> Unit,
  onDelete: () -> Unit,
  onFork: () -> Unit,
) {
  var actionsExpanded by remember(thread.id, archived) { mutableStateOf(false) }

  Box(
    modifier =
      Modifier
        .fillMaxWidth(),
  ) {
    Row(
      modifier =
        Modifier
          .fillMaxWidth()
          .combinedClickable(
            enabled = !isOpening,
            onClick = onResume,
            onLongClick = { actionsExpanded = true },
          )
          .padding(horizontal = 4.dp, vertical = 8.dp),
      horizontalArrangement = Arrangement.spacedBy(8.dp),
      verticalAlignment = Alignment.Top,
    ) {
      Column(
        modifier = Modifier.weight(1f),
        verticalArrangement = Arrangement.spacedBy(4.dp),
      ) {
        Text(
          thread.displayTitle,
          color = ShellowColors.TerminalText,
          style = MaterialTheme.typography.bodyMedium,
          fontWeight = FontWeight.SemiBold,
          maxLines = 1,
          overflow = TextOverflow.Ellipsis,
        )
        Text(
          formatCodexThreadMeta(thread, showProjectContext),
          color = ShellowColors.TerminalMuted,
          style = MaterialTheme.typography.labelSmall,
          maxLines = 1,
          overflow = TextOverflow.Ellipsis,
        )
      }
      if (isOpening) {
        Text("Opening", color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.labelMedium, modifier = Modifier.padding(horizontal = 12.dp, vertical = 10.dp))
      } else {
        thread.statusIndicator?.let { indicator ->
          CodexThreadStatusBadge(indicator)
        }
      }
    }
    DropdownMenu(
      expanded = actionsExpanded,
      onDismissRequest = { actionsExpanded = false },
    ) {
      DropdownMenuItem(
        text = { Text("Rename") },
        onClick = {
          actionsExpanded = false
          onRename()
        },
      )
      DropdownMenuItem(
        text = { Text("Fork") },
        onClick = {
          actionsExpanded = false
          onFork()
        },
      )
      DropdownMenuItem(
        text = { Text(if (archived) "Unarchive" else "Archive") },
        onClick = {
          actionsExpanded = false
          if (archived) onUnarchive() else onArchive()
        },
      )
      DropdownMenuItem(
        text = { Text("Delete") },
        onClick = {
          actionsExpanded = false
          onDelete()
        },
      )
    }
  }
}

private fun formatCodexThreadMeta(thread: CodexThreadSummary, showProjectContext: Boolean = true): String {
  val timestampMs = thread.updatedAt.coerceAtLeast(0L) * 1000L
  val formatted = SimpleDateFormat("MMM d, HH:mm", Locale.getDefault()).format(Date(timestampMs))
  val fork = if (thread.forkedFromId != null) "fork" else ""
  val project = if (showProjectContext) lastPathComponent(thread.cwd) else ""
  return listOf(project, formatted, fork).filter { it.isNotBlank() }.joinToString(" · ")
}

@Composable
private fun CodexSettingsDialog(
  model: String,
  modelOptions: List<CodexModelOption>,
  isLoadingModels: Boolean,
  modelsError: String?,
  reasoningEffort: String,
  serviceTier: String,
  approvalPolicy: String,
  sandbox: String,
  onModelChange: (String) -> Unit,
  onReasoningEffortChange: (String) -> Unit,
  onServiceTierChange: (String) -> Unit,
  onApprovalPolicyChange: (String) -> Unit,
  onSandboxChange: (String) -> Unit,
  onDismiss: () -> Unit,
  canApply: Boolean,
  onApply: () -> Unit,
) {
  val modelChoices =
    remember(modelOptions, model) {
      listOf("" to "Default") +
        codexModelPickerOptions(modelOptions, model).map { it.id to it.name }
    }
  val selectedModel = modelOptions.firstOrNull { it.id == model }
  val reasoningChoices =
    listOf("" to "Use model default") +
      selectedModel.orEmptyReasoningEfforts().map { it.id to it.name }
  val speedChoices =
    listOf("" to "Standard") +
      selectedModel.orEmptyServiceTiers().map { it.id to it.name }

  AlertDialog(
    onDismissRequest = onDismiss,
    containerColor = ShellowColors.PanelBackground,
    titleContentColor = ShellowColors.TerminalText,
    textContentColor = ShellowColors.TerminalText,
    title = { Text("Codex Settings") },
    text = {
      Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
        CodexOptionRow("Model", model, modelChoices, onModelChange)
        if (isLoadingModels) {
          CodexInlineStatusRow(
            text = "Loading models",
            isLoading = true,
            modifier = Modifier.padding(bottom = 4.dp),
          )
        } else if (!modelsError.isNullOrBlank()) {
          CodexInlineStatusRow(
            text = modelsError,
            tone = CodexInlineStatusTone.Warning,
            modifier = Modifier.padding(bottom = 4.dp),
          )
        }
        CodexOptionRow("Reasoning", reasoningEffort, reasoningChoices, onReasoningEffortChange)
        CodexOptionRow("Speed", serviceTier, speedChoices, onServiceTierChange)
        if (selectedModel?.serviceTiers.isNullOrEmpty()) {
          CodexInlineStatusRow(
            text = "Fast mode is unavailable for this model.",
            modifier = Modifier.padding(bottom = 4.dp),
          )
        }
        CodexOptionRow("Approval", approvalPolicy, listOf("" to "Default", "untrusted" to "Untrusted", "on-request" to "On request", "never" to "Never"), onApprovalPolicyChange)
        CodexOptionRow("Sandbox", sandbox, listOf("" to "Default", "read-only" to "Read only", "workspace-write" to "Workspace write", "danger-full-access" to "Danger full access"), onSandboxChange)
        Text(
          "Model, reasoning, speed, and approval apply to the next turn. Sandbox applies when a thread starts or resumes.",
          color = ShellowColors.TerminalMuted,
          style = MaterialTheme.typography.labelSmall,
          modifier = Modifier.padding(top = 8.dp),
        )
      }
    },
    confirmButton = { TextButton(onClick = onApply, enabled = canApply) { Text("Apply") } },
    dismissButton = { TextButton(onClick = onDismiss) { Text("Cancel") } },
  )
}

private fun CodexModelOption?.orEmptyReasoningEfforts() = this?.reasoningEfforts.orEmpty()

private fun CodexModelOption?.orEmptyServiceTiers() = this?.serviceTiers.orEmpty()

@Composable
private fun CodexOptionRow(
  title: String,
  selected: String,
  options: List<Pair<String, String>>,
  onSelected: (String) -> Unit,
) {
  var expanded by remember(title, selected) { mutableStateOf(false) }
  val selectedLabel =
    options.firstOrNull { (value, _) -> value == selected }?.second
      ?: selected.ifBlank { "Default" }

  Box(modifier = Modifier.fillMaxWidth()) {
    Row(
      modifier =
        Modifier
          .fillMaxWidth()
          .clickable { expanded = true }
          .padding(vertical = 12.dp),
      horizontalArrangement = Arrangement.spacedBy(12.dp),
      verticalAlignment = Alignment.CenterVertically,
    ) {
      Text(
        title,
        color = ShellowColors.TerminalText,
        style = MaterialTheme.typography.bodyMedium,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
        modifier = Modifier.weight(0.8f),
      )
      Text(
        selectedLabel,
        color = ShellowColors.Accent,
        style = MaterialTheme.typography.bodyMedium,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
        textAlign = TextAlign.End,
        modifier = Modifier.weight(1.2f),
      )
    }

    DropdownMenu(
      expanded = expanded,
      onDismissRequest = { expanded = false },
    ) {
      options.forEach { (value, label) ->
        DropdownMenuItem(
          text = {
            Text(
              label,
              fontWeight = if (selected == value) FontWeight.SemiBold else FontWeight.Normal,
            )
          },
          onClick = {
            expanded = false
            onSelected(value)
          },
        )
      }
    }
  }
}

private val CodexThreadSummary.displayTitle: String
  get() = name?.takeIf { it.isNotBlank() } ?: preview.ifBlank { id }

private fun matchesHomeSearch(value: String, query: String): Boolean =
  query.isBlank() || value.contains(query, ignoreCase = true)

private fun mergeProjects(vararg groups: List<String>): List<String> {
  val result = mutableListOf<String>()
  groups.asList().flatten().forEach { path ->
    val trimmed = path.trim()
    if (trimmed.isNotEmpty() && trimmed !in result) {
      result += trimmed
    }
  }
  return result.take(20)
}

private fun lastPathComponent(path: String): String =
  path.trim('/').split('/').lastOrNull()?.takeIf { it.isNotBlank() } ?: path

private fun codexCompactPath(path: String): String {
  val trimmed = path.trim()
  if (trimmed.isEmpty()) return path
  val components = trimmed.trim('/').split('/').filter { it.isNotBlank() }
  if (components.isEmpty()) return trimmed
  if (components.size >= 2 && components.first() == "Users") {
    val remainder = components.drop(2)
    return if (remainder.isEmpty()) "~" else "~/${remainder.joinToString("/")}"
  }
  return if (trimmed.startsWith("/")) {
    "/${components.joinToString("/")}"
  } else {
    components.joinToString("/")
  }
}

private val CodexMessage.isVisibleInChat: Boolean
  get() = (visibility == CodexMessageVisibility.Primary || visibility == CodexMessageVisibility.Compact) && !isRoutineLifecycleStatus

private fun String.isRoutineCodexOperationSuccess(): Boolean =
  trim() == "Codex thread resumed."

private fun codexChatScrollSignature(
  messages: List<CodexMessage>,
  pendingApprovalCount: Int,
  turnActive: Boolean,
): Int {
  var signature = pendingApprovalCount
  signature = signature * 31 + if (turnActive) 1 else 0
  messages.filter { it.isVisibleInChat }.forEach { message ->
    signature = signature * 31 + message.id.length
    signature = signature * 31 + message.text.length
    signature = signature * 31 + (message.title?.length ?: 0)
    signature = signature * 31 + (message.detail?.length ?: 0)
    signature = signature * 31 + (message.transcript?.length ?: 0)
    signature = signature * 31 + if (message.isStreaming) 1 else 0
    signature = signature * 31 + message.blocks.sumOf { it.scrollContentLength() }
  }
  return signature
}

private fun CodexMarkdownBlock.scrollContentLength(): Int =
  id.length +
    text.length +
    (imageAlt?.length ?: 0) +
    runs.sumOf { it.text.length } +
    items.sumOf { item -> item.text.length + item.runs.sumOf { it.text.length } } +
    tableHeaders.sumOf { cell -> cell.text.length + cell.runs.sumOf { it.text.length } } +
    tableRows.sumOf { row ->
      row.sumOf { cell -> cell.text.length + cell.runs.sumOf { it.text.length } }
    }

private val CodexMessage.isRoutineLifecycleStatus: Boolean
  get() {
    if (kind != CodexMessageKind.Status || visibility != CodexMessageVisibility.Compact) return false
    return text.ifBlank { detail.orEmpty() }.trim() == "Codex thread resumed."
  }

@Composable
private fun CodexMessageBubble(message: CodexMessage) {
  if (message.visibility == CodexMessageVisibility.Compact) {
    CodexCompactMessageRow(message)
    return
  }

  val container =
    when (message.role) {
      CodexMessageRole.User -> ShellowColors.UserMessageBackground
      CodexMessageRole.Assistant -> ShellowColors.AssistantMessageBackground
      CodexMessageRole.Status -> ShellowColors.StatusMessageBackground
      CodexMessageRole.Tool,
      CodexMessageRole.CommandOutput -> ShellowColors.ToolMessageBackground
    }
  val label =
    when (message.role) {
      CodexMessageRole.User -> "You"
      CodexMessageRole.Assistant -> "Codex"
      CodexMessageRole.Status -> "Status"
      CodexMessageRole.Tool -> "Tool"
      CodexMessageRole.CommandOutput -> "Output"
    }
  val hasContainer =
    when (message.role) {
      CodexMessageRole.User,
      CodexMessageRole.Tool,
      CodexMessageRole.CommandOutput -> true
      CodexMessageRole.Assistant,
      CodexMessageRole.Status -> false
    }
  val horizontalPadding = if (hasContainer) 10.dp else 4.dp
  val verticalPadding = if (hasContainer) 10.dp else 6.dp
  Column(
    modifier =
      Modifier
        .fillMaxWidth()
        .then(if (hasContainer) Modifier.background(container, RoundedCornerShape(8.dp)) else Modifier)
        .padding(horizontal = horizontalPadding, vertical = verticalPadding),
    verticalArrangement = Arrangement.spacedBy(4.dp),
  ) {
    if (hasContainer) {
      Text(label, color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.labelSmall)
    }
    CodexMarkdownContent(message)
    if (message.role == CodexMessageRole.User) {
      when (message.delivery) {
        CodexMessageDelivery.Queued -> Text("Queued…", color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.labelSmall)
        CodexMessageDelivery.Sent -> Text("Sending…", color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.labelSmall)
        CodexMessageDelivery.Failed -> Text(
          message.detail?.let { "Failed · $it" } ?: "Failed to send",
          color = MaterialTheme.colorScheme.error,
          style = MaterialTheme.typography.labelSmall,
        )
        CodexMessageDelivery.Committed, null -> Unit
      }
    }
  }
}

@Composable
private fun CodexCompactMessageRow(message: CodexMessage) {
  var expanded by remember(message.id) { mutableStateOf(false) }
  val title = message.title ?: compactMessageTitle(message)
  val rawBody = message.text.ifBlank { message.detail.orEmpty() }
  val isStatus = message.kind == CodexMessageKind.Status
  val body = if (isStatus) normalizedCompactStatusText(rawBody) else rawBody
  val hasNormalizedBody = rawBody != body
  val hidesSecondaryText = message.hidesCompactSecondaryText
  val isRoutineCommandCompletion = hidesSecondaryText && !message.isStreaming
  val hasDetails =
    !message.transcript.isNullOrBlank() ||
      hasNormalizedBody ||
      (hidesSecondaryText && body.isNotBlank()) ||
      (!message.detail.isNullOrBlank() && message.detail != body)

  Column(
    modifier =
      Modifier
        .fillMaxWidth()
        .clickable(enabled = hasDetails) { expanded = !expanded }
        .semantics {
          if (hasDetails) {
            role = Role.Button
            stateDescription = if (expanded) "Expanded" else "Collapsed"
          }
        }
        .padding(horizontal = 4.dp, vertical = if (isRoutineCommandCompletion) 3.dp else 6.dp),
    verticalArrangement = Arrangement.spacedBy(if (isRoutineCommandCompletion) 4.dp else 5.dp),
  ) {
    Row(
      modifier = Modifier.fillMaxWidth(),
      horizontalArrangement = Arrangement.spacedBy(8.dp),
      verticalAlignment = Alignment.Top,
    ) {
      Text(
        compactMessageGlyph(message),
        color = ShellowColors.TerminalMuted,
        style = MaterialTheme.typography.labelSmall,
        fontWeight = if (isRoutineCommandCompletion) FontWeight.Normal else FontWeight.SemiBold,
        modifier = Modifier.width(16.dp).padding(top = 1.dp),
      )
      if (isStatus) {
        Text(
          body.ifBlank { title },
          color = ShellowColors.TerminalMuted,
          style = MaterialTheme.typography.bodySmall,
          maxLines = if (expanded) Int.MAX_VALUE else 2,
          overflow = TextOverflow.Ellipsis,
          modifier = Modifier.weight(1f),
        )
      } else {
        Column(modifier = Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
          Text(
            title,
            color = if (isRoutineCommandCompletion) ShellowColors.TerminalMuted else ShellowColors.TerminalText,
            style = if (isRoutineCommandCompletion) MaterialTheme.typography.labelSmall else MaterialTheme.typography.labelMedium,
            fontWeight = if (isRoutineCommandCompletion) FontWeight.Normal else FontWeight.SemiBold,
          )
          if (body.isNotBlank() && !hidesSecondaryText) {
            Text(
              body,
              color = ShellowColors.TerminalMuted,
              style = MaterialTheme.typography.bodySmall,
              maxLines = if (expanded) Int.MAX_VALUE else 2,
              overflow = TextOverflow.Ellipsis,
            )
          }
        }
      }
      if (message.isStreaming) {
        Text("live", color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.labelSmall)
      } else if (hasDetails) {
        Text(if (expanded) "less" else "more", color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.labelSmall)
      }
    }

    if (expanded) {
      body.takeIf { hidesSecondaryText && it.isNotBlank() }?.let { detail ->
        Text(detail, color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.bodySmall)
      } ?: message.detail?.takeIf { it.isNotBlank() && it != body }?.let { detail ->
        Text(detail, color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.bodySmall)
      } ?: rawBody.takeIf { hasNormalizedBody }?.let { detail ->
        Text(detail, color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.bodySmall)
      }
      message.transcript?.takeIf { it.isNotBlank() }?.let { transcript ->
        Text(
          transcript,
          color = ShellowColors.TerminalText,
          style = MaterialTheme.typography.bodySmall.copy(fontFamily = FontFamily.Monospace),
          modifier =
            Modifier
              .fillMaxWidth()
              .background(ShellowColors.CodeBackground, RoundedCornerShape(6.dp))
              .padding(8.dp),
        )
      }
    }
  }
}

private val CodexMessage.hidesCompactSecondaryText: Boolean
  get() = title?.trim()?.startsWith("Completed ·") == true

private fun normalizedCompactStatusText(text: String): String {
  val trimmed = text.trim()
  if (!trimmed.startsWith("app-server sent non-JSON output")) return text
  val byteCount = trimmed.substringAfter("(", "").substringBefore(")", "")
  return if (byteCount.isNotBlank()) {
    "Server output was not JSON ($byteCount)"
  } else {
    "Server output was not JSON"
  }
}

private fun compactMessageTitle(message: CodexMessage): String =
  when (message.kind) {
    CodexMessageKind.Command -> "Command"
    CodexMessageKind.CommandOutput -> "Command output"
    CodexMessageKind.FileChange -> "File change"
    CodexMessageKind.ReasoningSummary -> "Thinking"
    CodexMessageKind.Status -> "Status"
    CodexMessageKind.ToolCall,
    CodexMessageKind.ToolResult -> "Tool"
    CodexMessageKind.Plan -> "Plan"
    CodexMessageKind.Commentary,
    CodexMessageKind.FinalAnswer -> "Codex"
    CodexMessageKind.UserMessage -> "You"
  }

private fun compactMessageGlyph(message: CodexMessage): String =
  when (message.kind) {
    CodexMessageKind.Command,
    CodexMessageKind.CommandOutput -> "$"
    CodexMessageKind.FileChange -> "+"
    CodexMessageKind.ReasoningSummary -> "..."
    CodexMessageKind.Status -> "i"
    CodexMessageKind.ToolCall,
    CodexMessageKind.ToolResult -> ">"
    CodexMessageKind.Plan -> "#"
    CodexMessageKind.Commentary,
    CodexMessageKind.FinalAnswer -> "*"
    CodexMessageKind.UserMessage -> "@"
  }

@Composable
private fun CodexMarkdownContent(message: CodexMessage) {
  Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
    if (message.blocks.isEmpty()) {
      Text(
        message.text.ifBlank { "..." },
        color = if (message.role == CodexMessageRole.Status) ShellowColors.TerminalMuted else ShellowColors.TerminalText,
        style =
          if (message.role == CodexMessageRole.CommandOutput) {
            MaterialTheme.typography.bodySmall.copy(fontFamily = FontFamily.Monospace)
          } else {
            MaterialTheme.typography.bodyMedium
          },
      )
    } else {
      message.blocks.forEach { block ->
        CodexMarkdownBlockView(block)
      }
    }

    if (message.isStreaming) {
      Text("Streaming", color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.labelSmall)
    }
  }
}

@Composable
private fun CodexMarkdownBlockView(block: CodexMarkdownBlock) {
  when (block.kind) {
    CodexMarkdownBlockKind.Paragraph ->
      Text(
        markdownAnnotatedString(block.runs, block.text),
        color = ShellowColors.TerminalText,
        style = MaterialTheme.typography.bodyMedium,
      )
    CodexMarkdownBlockKind.Heading ->
      Text(
        markdownAnnotatedString(block.runs, block.text),
        color = ShellowColors.TerminalText,
        style =
          when (block.level ?: 2) {
            1 -> MaterialTheme.typography.titleMedium
            2 -> MaterialTheme.typography.titleSmall
            else -> MaterialTheme.typography.bodyMedium.copy(fontWeight = FontWeight.SemiBold)
          },
      )
    CodexMarkdownBlockKind.List ->
      Column(verticalArrangement = Arrangement.spacedBy(5.dp)) {
        block.items.forEachIndexed { index, item ->
          Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.Top) {
            Text(
              if (block.ordered) "${index + 1}." else "•",
              color = ShellowColors.TerminalMuted,
              style = MaterialTheme.typography.bodyMedium.copy(fontFamily = FontFamily.Monospace),
              modifier = Modifier.width(24.dp),
            )
            Text(
              markdownAnnotatedString(item.runs, item.text),
              color = ShellowColors.TerminalText,
              style = MaterialTheme.typography.bodyMedium,
              modifier = Modifier.weight(1f),
            )
          }
        }
      }
    CodexMarkdownBlockKind.BlockQuote ->
      Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.Top) {
        Box(
          modifier =
            Modifier
              .width(3.dp)
              .height(22.dp)
              .background(ShellowColors.TerminalMuted, RoundedCornerShape(2.dp)),
        )
        Text(
          markdownAnnotatedString(block.runs, block.text),
          color = ShellowColors.TerminalMuted,
          style = MaterialTheme.typography.bodyMedium,
          modifier = Modifier.weight(1f),
        )
      }
    CodexMarkdownBlockKind.CodeBlock -> CodexCodeBlock(block)
    CodexMarkdownBlockKind.Table -> CodexTableBlock(block)
    CodexMarkdownBlockKind.HorizontalRule ->
      Box(
        modifier =
          Modifier
            .fillMaxWidth()
            .height(1.dp)
            .background(ShellowColors.TerminalMuted.copy(alpha = 0.35f)),
      )
    CodexMarkdownBlockKind.Image -> CodexImageBlock(block)
  }
}

@Composable
private fun CodexImageBlock(block: CodexMarkdownBlock) {
  val source = block.imageUrl ?: block.text
  val alt = block.imageAlt ?: block.text
  val bitmapState =
    produceState<Bitmap?>(initialValue = null, source) {
      value = withContext(Dispatchers.IO) { loadCodexBitmap(source) }
    }

  Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
    val bitmap = bitmapState.value
    if (bitmap != null) {
      Image(
        bitmap = bitmap.asImageBitmap(),
        contentDescription = alt.ifBlank { null },
        modifier = Modifier.fillMaxWidth().heightIn(max = 280.dp),
        contentScale = ContentScale.Fit,
      )
    } else {
      Row(
        modifier =
          Modifier
            .fillMaxWidth()
            .heightIn(min = 76.dp)
            .background(ShellowColors.CodeBackground, RoundedCornerShape(8.dp))
            .padding(10.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalAlignment = Alignment.CenterVertically,
      ) {
        Text("Image", color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.labelMedium)
        Text(source.ifBlank { "Image unavailable" }, color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.labelSmall, maxLines = 2, overflow = TextOverflow.Ellipsis)
      }
    }
    if (alt.isNotBlank()) {
      Text(alt, color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.labelSmall, maxLines = 2, overflow = TextOverflow.Ellipsis)
    }
  }
}

private fun loadCodexBitmap(source: String): Bitmap? {
  val trimmed = source.trim()
  if (trimmed.isEmpty()) return null
  return runCatching {
    when {
      trimmed.startsWith("data:image/") -> {
        val payload = trimmed.substringAfter(",", "")
        if (payload.isBlank()) {
          null
        } else {
          val bytes = Base64.decode(payload, Base64.DEFAULT)
          BitmapFactory.decodeByteArray(bytes, 0, bytes.size)
        }
      }
      trimmed.startsWith("http://") || trimmed.startsWith("https://") ->
        URL(trimmed).openStream().use(BitmapFactory::decodeStream)
      trimmed.startsWith("file://") ->
        BitmapFactory.decodeFile(URL(trimmed).path)
      trimmed.startsWith("/") || trimmed.startsWith("~") ->
        BitmapFactory.decodeFile(trimmed.replaceFirst("^~".toRegex(), System.getProperty("user.home") ?: "~"))
      else -> null
    }
  }.getOrNull()
}

@Composable
private fun CodexTableBlock(block: CodexMarkdownBlock) {
  val columnCount =
    maxOf(
      block.tableHeaders.size,
      block.tableRows.maxOfOrNull { it.size } ?: 0,
      1,
    )
  Column(
    modifier =
      Modifier
        .fillMaxWidth()
        .horizontalScroll(rememberScrollState())
        .background(ShellowColors.TableBackground, RoundedCornerShape(8.dp)),
  ) {
    if (block.tableHeaders.isNotEmpty()) {
      CodexTableRow(block.tableHeaders, columnCount, isHeader = true)
    }
    block.tableRows.forEach { row ->
      CodexTableRow(row, columnCount, isHeader = false)
    }
  }
}

@Composable
private fun CodexTableRow(
  cells: List<CodexMarkdownTableCell>,
  columnCount: Int,
  isHeader: Boolean,
) {
  Row {
    for (index in 0 until columnCount) {
      val cell = cells.getOrNull(index) ?: CodexMarkdownTableCell("", emptyList())
      Box(
        modifier =
          Modifier
            .width(132.dp)
            .background(if (isHeader) ShellowColors.TableHeaderBackground else ShellowColors.TableBackground)
            .padding(horizontal = 9.dp, vertical = 8.dp),
      ) {
        Text(
          markdownAnnotatedString(cell.runs, cell.text),
          color = ShellowColors.TerminalText,
          style =
            if (isHeader) {
              MaterialTheme.typography.labelMedium.copy(fontWeight = FontWeight.SemiBold)
            } else {
              MaterialTheme.typography.bodySmall
            },
        )
      }
    }
  }
}

@Composable
private fun CodexCodeBlock(block: CodexMarkdownBlock) {
  val clipboard = LocalClipboardManager.current
  Column(
    modifier =
      Modifier
        .fillMaxWidth()
        .background(ShellowColors.CodeBackground, RoundedCornerShape(8.dp)),
  ) {
    Row(
      modifier = Modifier.fillMaxWidth().background(ShellowColors.CodeHeaderBackground).padding(horizontal = 10.dp, vertical = 7.dp),
      horizontalArrangement = Arrangement.spacedBy(8.dp),
      verticalAlignment = Alignment.CenterVertically,
    ) {
      Text(block.language ?: "code", color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.labelSmall, modifier = Modifier.weight(1f))
      if (block.incomplete) {
        Text("streaming", color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.labelSmall)
      }
      TextButton(onClick = { clipboard.setText(AnnotatedString(block.text)) }) { Text("Copy") }
    }
    Row(modifier = Modifier.horizontalScroll(rememberScrollState()).padding(10.dp)) {
      Text(
        block.text.ifBlank { " " },
        color = ShellowColors.TerminalText,
        style = MaterialTheme.typography.bodySmall.copy(fontFamily = FontFamily.Monospace),
      )
    }
  }
}

private fun markdownAnnotatedString(
  runs: List<CodexMarkdownInlineRun>,
  fallback: String,
): AnnotatedString =
  buildAnnotatedString {
    val usableRuns =
      if (runs.isEmpty()) {
        listOf(CodexMarkdownInlineRun(fallback, CodexMarkdownInlineStyle.Text, null))
      } else {
        runs
      }
    usableRuns.forEach { run ->
      val style =
        when (run.style) {
          CodexMarkdownInlineStyle.Text -> SpanStyle()
          CodexMarkdownInlineStyle.Bold -> SpanStyle(fontWeight = FontWeight.SemiBold)
          CodexMarkdownInlineStyle.Italic -> SpanStyle(fontStyle = FontStyle.Italic)
          CodexMarkdownInlineStyle.BoldItalic -> SpanStyle(fontWeight = FontWeight.SemiBold, fontStyle = FontStyle.Italic)
          CodexMarkdownInlineStyle.Code -> SpanStyle(fontFamily = FontFamily.Monospace)
          CodexMarkdownInlineStyle.Link -> SpanStyle(color = ShellowColors.Accent, textDecoration = TextDecoration.Underline)
        }
      pushStyle(style)
      append(run.text)
      pop()
    }
  }

@Composable
private fun CodexApprovalCard(
  approval: CodexApproval,
  onDecision: (String) -> Unit,
) {
  var selections by remember(approval.requestId) { mutableStateOf<Map<String, Set<String>>>(emptyMap()) }
  var customAnswers by remember(approval.requestId) { mutableStateOf<Map<String, String>>(emptyMap()) }
  var toolResult by remember(approval.requestId) { mutableStateOf("") }

  fun answerFor(question: xyz.zinglix.shellow.core.CodexUserInputQuestion): List<String>? {
    val custom = customAnswers[question.id].orEmpty().trim()
    if (custom.isNotEmpty()) return listOf(custom)
    val selected = selections[question.id].orEmpty()
    return question.options.map { it.label }.filter { it in selected }.takeIf { it.isNotEmpty() }
  }
  Row(
    modifier =
      Modifier
        .fillMaxWidth()
        .background(ShellowColors.PanelBackground, RoundedCornerShape(8.dp))
        .padding(12.dp),
    horizontalArrangement = Arrangement.spacedBy(10.dp),
    verticalAlignment = Alignment.Top,
  ) {
    Box(
      modifier =
        Modifier
          .padding(top = 2.dp)
          .width(3.dp)
          .height(44.dp)
          .background(ShellowColors.Warning, RoundedCornerShape(2.dp)),
    )
    Column(
      modifier = Modifier.weight(1f),
      verticalArrangement = Arrangement.spacedBy(7.dp),
    ) {
      Text(
        approval.title,
        color = ShellowColors.TerminalText,
        style = MaterialTheme.typography.labelLarge,
        fontWeight = FontWeight.SemiBold,
      )
      if (approval.questions.isEmpty()) {
        Text(approval.detail, color = ShellowColors.TerminalText, style = MaterialTheme.typography.bodySmall)
      } else {
        approval.questions.forEach { question ->
          Text(
            question.header.uppercase(),
            color = ShellowColors.TerminalMuted,
            style = MaterialTheme.typography.labelSmall,
            fontWeight = FontWeight.SemiBold,
          )
          Text(
            question.question,
            color = ShellowColors.TerminalText,
            style = MaterialTheme.typography.bodyMedium,
            fontWeight = FontWeight.Medium,
          )
          question.options.forEach { option ->
            val selected = option.label in selections[question.id].orEmpty()
            Row(
              modifier =
                Modifier
                  .fillMaxWidth()
                  .background(
                    if (selected) ShellowColors.Accent.copy(alpha = 0.12f) else ShellowColors.KeyBackground.copy(alpha = 0.35f),
                    RoundedCornerShape(7.dp),
                  )
                  .clickable {
                    val current = selections[question.id].orEmpty()
                    val next =
                      if (question.multiSelect) {
                        if (option.label in current) current - option.label else current + option.label
                      } else {
                        setOf(option.label)
                      }
                    selections = selections + (question.id to next)
                    customAnswers = customAnswers + (question.id to "")
                  }
                  .padding(horizontal = 10.dp, vertical = 8.dp),
              horizontalArrangement = Arrangement.spacedBy(9.dp),
              verticalAlignment = Alignment.Top,
            ) {
              Text(
                if (question.multiSelect) {
                  if (selected) "☑" else "☐"
                } else {
                  if (selected) "●" else "○"
                },
                color = if (selected) ShellowColors.Accent else ShellowColors.TerminalMuted,
              )
              Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                Text(option.label, color = ShellowColors.TerminalText, fontWeight = FontWeight.SemiBold)
                if (option.description.isNotEmpty()) {
                  Text(option.description, color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.bodySmall)
                }
              }
            }
            if (selected && !option.preview.isNullOrEmpty()) {
              Text(
                option.preview,
                modifier = Modifier.fillMaxWidth().background(ShellowColors.KeyBackground, RoundedCornerShape(6.dp)).padding(8.dp),
                color = ShellowColors.TerminalText,
                style = MaterialTheme.typography.bodySmall.copy(fontFamily = FontFamily.Monospace),
              )
            }
          }
          if (question.options.isEmpty() || question.isOther) {
            OutlinedTextField(
              value = customAnswers[question.id].orEmpty(),
              onValueChange = { value -> customAnswers = customAnswers + (question.id to value) },
              modifier = Modifier.fillMaxWidth(),
              label = { Text(if (question.isSecret) "Private answer" else "Other answer") },
              visualTransformation = if (question.isSecret) PasswordVisualTransformation() else androidx.compose.ui.text.input.VisualTransformation.None,
              minLines = 1,
              maxLines = 3,
            )
          }
        }
      }
      approval.cwd?.let {
        Text(
          it,
          color = ShellowColors.TerminalMuted,
          style = MaterialTheme.typography.labelSmall.copy(fontFamily = FontFamily.Monospace),
          maxLines = 1,
          overflow = TextOverflow.Ellipsis,
        )
      }
      when (approval.kind) {
        CodexApprovalKind.UserInput -> {
          Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
            TextButton(
              enabled = approval.questions.all { answerFor(it) != null },
              onClick = {
                val payload = JSONObject()
                approval.questions.forEach { question ->
                  val values = JSONArray()
                  answerFor(question).orEmpty().forEach(values::put)
                  payload.put(question.id, values)
                }
                onDecision(payload.toString())
              },
            ) { Text("Submit", fontWeight = FontWeight.SemiBold) }
          }
        }
        CodexApprovalKind.Permissions -> {
          approval.permissions?.let {
            Text(
              it,
              style = MaterialTheme.typography.labelSmall.copy(fontFamily = FontFamily.Monospace),
              color = ShellowColors.TerminalMuted,
            )
          }
          CodexApprovalActions(
            showSession = "acceptForSession" in approval.availableDecisions,
            allowLabel = "Allow for this turn",
            onDecision = onDecision,
          )
        }
        CodexApprovalKind.Elicitation -> {
          approval.permissions?.let {
            Text(
              it,
              style = MaterialTheme.typography.labelSmall.copy(fontFamily = FontFamily.Monospace),
              color = ShellowColors.TerminalMuted,
            )
          }
          OutlinedTextField(
            value = toolResult,
            onValueChange = { toolResult = it },
            modifier = Modifier.fillMaxWidth(),
            label = { Text("JSON response") },
          )
          Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
            TextButton(onClick = { onDecision("decline") }) { Text("Decline") }
            TextButton(enabled = toolResult.isNotBlank(), onClick = { onDecision(toolResult) }) {
              Text("Submit")
            }
          }
        }
        CodexApprovalKind.Tool -> {
          OutlinedTextField(
            value = toolResult,
            onValueChange = { toolResult = it },
            modifier = Modifier.fillMaxWidth(),
            label = { Text("Tool result") },
          )
          Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
            TextButton(onClick = { onDecision("decline") }) { Text("Decline") }
            TextButton(
              enabled = toolResult.isNotBlank(),
              onClick = {
                onDecision(
                  JSONObject()
                    .put("success", true)
                    .put("contentItems", JSONArray().put(JSONObject().put("type", "inputText").put("text", toolResult)))
                    .toString(),
                )
              },
            ) { Text("Submit") }
          }
        }
        CodexApprovalKind.Command, CodexApprovalKind.FileChange -> {
          CodexApprovalActions(
            showSession = "acceptForSession" in approval.availableDecisions,
            allowLabel = if (approval.kind == CodexApprovalKind.Command) "Run command" else "Apply changes",
            onDecision = onDecision,
          )
        }
      }
    }
  }
}

@Composable
private fun CodexApprovalActions(
  showSession: Boolean,
  allowLabel: String,
  onDecision: (String) -> Unit,
) {
  Row(
    modifier = Modifier.fillMaxWidth(),
    horizontalArrangement = Arrangement.spacedBy(2.dp),
    verticalAlignment = Alignment.CenterVertically,
  ) {
    TextButton(onClick = { onDecision("accept") }) {
      Text(allowLabel, fontWeight = FontWeight.SemiBold)
    }
    if (showSession) {
      TextButton(onClick = { onDecision("acceptForSession") }) {
        Text("Allow for session")
      }
    }
    Spacer(modifier = Modifier.weight(1f))
    TextButton(onClick = { onDecision("decline") }) {
      Text("Deny", color = MaterialTheme.colorScheme.error)
    }
  }
}

@Composable
private fun TerminalScreen(
  session: TerminalSession,
  displaySettings: AppDisplaySettings,
  profileName: String,
  persistentTerminal: PersistentTerminalConfiguration?,
  loadPersistentSessions: (suspend () -> RemoteTerminalSessionCatalog)?,
  onBackToHosts: () -> Unit,
  onInput: (String) -> Unit,
  onReconnect: (() -> Unit)?,
  onDisconnect: () -> Unit,
  onResize: (Int, Int) -> Unit,
  onAttachRendererSurface: (Surface, Int, Int) -> String,
  onSetRendererOverlay: (String) -> String,
  onRenderRendererSurface: (Int, Int, Int, Int) -> Boolean,
  onDetachRendererSurface: () -> String,
  onClearTerminal: () -> Unit,
  onResetTerminal: () -> Unit,
) {
  var ctrlArmed by remember { mutableStateOf(false) }
  var altArmed by remember { mutableStateOf(false) }
  var reportedCols by remember { mutableIntStateOf(session.terminalCols) }
  var reportedRows by remember { mutableIntStateOf(session.terminalRows) }
  var selection by remember { mutableStateOf<TerminalSelection?>(null) }
  var pendingPaste by remember { mutableStateOf<String?>(null) }
  var pendingRemoteClipboard by remember { mutableStateOf<RemoteClipboardRequest?>(null) }
  var transcriptSaveResult by remember { mutableStateOf<TranscriptSaveResult?>(null) }
  var handledClipboardSequence by remember { mutableLongStateOf(0L) }
  var searchVisible by remember { mutableStateOf(false) }
  var toolsExpanded by remember { mutableStateOf(false) }
  var persistentToolsVisible by remember { mutableStateOf(false) }
  var persistentSessionsVisible by remember { mutableStateOf(false) }
  var persistentSessionCatalog by remember { mutableStateOf(RemoteTerminalSessionCatalog()) }
  var activePersistentSessionName by remember(persistentTerminal?.name) {
    mutableStateOf(persistentTerminal?.name)
  }
  var refreshingPersistentSessions by remember { mutableStateOf(false) }
  var pendingDestructiveAction by remember { mutableStateOf<TerminalDestructiveAction?>(null) }
  var searchQuery by remember { mutableStateOf("") }
  var searchIndex by remember { mutableIntStateOf(0) }
  var viewportWidthPx by remember { mutableIntStateOf(0) }
  var viewportHeightPx by remember { mutableIntStateOf(0) }
  var rendererSurfaceReady by remember { mutableStateOf(false) }
  var terminalInputRevision by remember { mutableStateOf(0) }
  var directInputValue by remember {
    mutableStateOf(TextFieldValue(TerminalDirectInputSentinel, selection = TextRange(TerminalDirectInputSentinel.length)))
  }
  val terminalFocusRequester = remember { FocusRequester() }
  val clipboard = LocalClipboardManager.current
  val context = LocalContext.current
  val density = LocalDensity.current
  val keyboardController = LocalSoftwareKeyboardController.current
  val selectedText = session.selectedText(selection)
  val selectedLink = selectedText?.firstTerminalUrl()
  val search = session.searchPresentation(searchQuery, searchIndex)
  val terminalListState = rememberLazyListState()
  val terminalScope = rememberCoroutineScope()
  val keyboardOffsetPx = WindowInsets.ime.getBottom(density)
  var layoutKeyboardOffsetPx by remember { mutableIntStateOf(keyboardOffsetPx) }
  val keyboardVisualDeltaDp = with(density) { (keyboardOffsetPx - layoutKeyboardOffsetPx).toDp() }
  val keyboardLayoutOffsetDp = with(density) { layoutKeyboardOffsetPx.toDp() }
  val terminalLiftDp = with(density) { (keyboardOffsetPx - layoutKeyboardOffsetPx).coerceAtLeast(0).toDp() }
  val terminalHeaderInsetDp = 76.dp
  val terminalSearchBarTopDp = 64.dp
  val terminalSearchInsetDp = 130.dp
  val terminalTopInsetDp = if (searchVisible) terminalSearchInsetDp else terminalHeaderInsetDp
  val terminalBottomInsetDp = 10.dp
  val terminalTopInsetPx = with(density) { terminalTopInsetDp.toPx() }
  val terminalBottomInsetPx = with(density) { terminalBottomInsetDp.toPx() }
  val terminalTextSizePx = with(density) { displaySettings.fontSizeSp.sp.toPx() }
  val gridCellWidthPx = (terminalTextSizePx * 0.62f).coerceAtLeast(with(density) { 5.5.dp.toPx() })
  val terminalRowHeightPx =
    ((terminalTextSizePx * 1.45f).coerceAtLeast(with(density) { 13.dp.toPx() }) + with(density) { 4.dp.toPx() }) *
      displaySettings.lineHeightScale
  val viewportHorizontalPaddingPx = with(density) { 14.dp.toPx() }
  val grid = session.grid
  val visibleGrid = grid?.takeIf { it.hasVisibleContent || it.activeScreen == TerminalScreenKind.Alternate }
  val gridVisible = visibleGrid != null
  val rustSurfaceEnabled = gridVisible
  val viewportRowCount = visibleGrid?.rows?.toInt()?.coerceAtLeast(1) ?: 1
  val viewportFirstRow = visibleGrid?.viewportFirstRow(terminalListState.firstVisibleItemIndex) ?: 0
  val rendererOverlayJson =
    visibleGrid?.let { androidRendererOverlayJson(it, selection, search, viewportFirstRow, viewportRowCount) }

  fun resetDirectInputValue() {
    directInputValue =
      TextFieldValue(TerminalDirectInputSentinel, selection = TextRange(TerminalDirectInputSentinel.length))
  }

  fun focusTerminalInput() {
    terminalFocusRequester.requestFocus()
    keyboardController?.show()
  }

  fun sendTerminalInput(value: String) {
    selection = null
    terminalInputRevision += 1
    onInput(value)
  }

  fun sendTerminalScrollInput(value: String) {
    selection = null
    onInput(value)
  }

  suspend fun refreshPersistentSessions() {
    val loader = loadPersistentSessions ?: return
    if (refreshingPersistentSessions) return
    refreshingPersistentSessions = true
    persistentSessionCatalog = loader()
    refreshingPersistentSessions = false
  }

  fun switchPersistentSession(name: String) {
    val configuration = persistentTerminal ?: return
    val validatedName = PersistentTerminalConfiguration.validatedName(name) ?: return
    activePersistentSessionName = validatedName
    persistentSessionsVisible = false
    persistentToolsVisible = false
    terminalScope.launch {
      sendTerminalInput(configuration.backend.detachSequence)
      delay(400)
      sendTerminalInput("${configuration.backend.attachCommand(validatedName)}\r")
      delay(900)
      refreshPersistentSessions()
    }
  }

  LaunchedEffect(session.state, persistentTerminal?.name) {
    if (session.state == ConnectionState.Connected && persistentTerminal != null) {
      if (activePersistentSessionName == null) {
        activePersistentSessionName = persistentTerminal.name
      }
      refreshPersistentSessions()
    }
  }

  LaunchedEffect(session.clipboardSequence, session.pendingClipboardText) {
    val remoteText = session.pendingClipboardText
    if (session.clipboardSequence > handledClipboardSequence && !remoteText.isNullOrEmpty()) {
      handledClipboardSequence = session.clipboardSequence
      pendingRemoteClipboard = RemoteClipboardRequest(session.clipboardSequence, remoteText)
    }
  }

  LaunchedEffect(search.activeHit) {
    val row =
      when (val hit = search.activeHit) {
        is TerminalSearchHit.Grid -> hit.row
        is TerminalSearchHit.History -> hit.row
        null -> null
      }
    if (row != null) {
      terminalListState.animateScrollToItem(row)
    }
  }

  LaunchedEffect(gridVisible) {
    if (!gridVisible) {
      rendererSurfaceReady = false
    }
  }

  fun reportViewportSize(widthPx: Int, heightPx: Int) {
    if (widthPx <= 0 || heightPx <= 0) return

    val contentWidthPx = (widthPx.toFloat() - viewportHorizontalPaddingPx * 2f).coerceAtLeast(1f)
    val contentHeightPx = (heightPx.toFloat() - terminalTopInsetPx - terminalBottomInsetPx).coerceAtLeast(1f)
    val cols = (contentWidthPx / gridCellWidthPx).toInt().coerceIn(20, 300)
    val rows = (contentHeightPx / terminalRowHeightPx).toInt().coerceIn(8, 120)
    if (cols != reportedCols || rows != reportedRows) {
      reportedCols = cols
      reportedRows = rows
      onResize(cols, rows)
    }
  }

  LaunchedEffect(gridCellWidthPx, terminalRowHeightPx, viewportWidthPx, viewportHeightPx) {
    reportViewportSize(viewportWidthPx, viewportHeightPx)
  }

  LaunchedEffect(keyboardOffsetPx) {
    delay(TerminalKeyboardLayoutCommitDelayMs)
    layoutKeyboardOffsetPx = keyboardOffsetPx
  }

  LaunchedEffect(Unit) {
    terminalFocusRequester.requestFocus()
  }

  fun sendEnter() {
    sendTerminalInput("\r")
    ctrlArmed = false
    altArmed = false
  }

  fun sendPaste(value: String) {
    sendTerminalInput(
      if (session.isBracketedPasteActive()) {
        "\u001B[200~$value\u001B[201~"
      } else {
        value
      },
    )
    ctrlArmed = false
    altArmed = false
  }

  fun handlePaste(value: String) {
    if (ctrlArmed) {
      sendTerminalInput(translateTerminalInput(value, true))
      ctrlArmed = false
      altArmed = false
      return
    }

    if (altArmed) {
      sendTerminalInput(value.metaEncodedTerminalInput())
      altArmed = false
      return
    }

    if (displaySettings.confirmPaste && value.isRiskyTerminalPaste()) {
      pendingPaste = value
      return
    }

    if (session.isBracketedPasteActive()) {
      sendPaste(value)
      return
    }

    sendPaste(value)
  }

  fun sendToolbarInput(value: String) {
    if (altArmed) {
      sendTerminalInput("\u001B$value")
      altArmed = false
    } else {
      sendTerminalInput(value)
    }
  }

  fun sendDirectText(value: String) {
    val payload = value.replace("\r\n", "\r").replace("\n", "\r")
    if (payload.isEmpty()) return

    if (ctrlArmed) {
      translateTerminalInput(payload, true).takeIf { it.isNotEmpty() }?.let(::sendTerminalInput)
      ctrlArmed = false
      altArmed = false
      return
    }

    if (altArmed) {
      sendTerminalInput(payload.metaEncodedTerminalInput())
      ctrlArmed = false
      altArmed = false
      return
    }

    sendTerminalInput(payload)
  }

  fun sendDirectBackspace() {
    sendToolbarInput("\u007F")
    ctrlArmed = false
  }

  fun handleDirectInputChange(value: TextFieldValue) {
    val text = value.text
    if (text == TerminalDirectInputSentinel) {
      directInputValue = value.copy(selection = TextRange(text.length))
      return
    }

    if (!text.contains(TerminalDirectInputSentinel)) {
      if (text.isEmpty()) {
        sendDirectBackspace()
      } else {
        sendDirectText(text)
      }
      resetDirectInputValue()
      return
    }

    val inserted = text.replace(TerminalDirectInputSentinel, "")
    if (inserted.isNotEmpty()) {
      sendDirectText(inserted)
    }
    resetDirectInputValue()
  }

  BackHandler {
    when (terminalBackAction(searchVisible = searchVisible, hasSelection = selection != null)) {
      TerminalBackAction.CloseSearch -> {
        searchVisible = false
        searchQuery = ""
        searchIndex = 0
      }
      TerminalBackAction.ClearSelection -> selection = null
      TerminalBackAction.NavigateBack -> onBackToHosts()
    }
  }

  val alternateScreenScrollConnection =
    remember(
      visibleGrid?.activeScreen,
      visibleGrid?.mouseReporting,
      visibleGrid?.sgrMouse,
      visibleGrid?.cols,
      visibleGrid?.rows,
      persistentTerminal?.backend,
      terminalRowHeightPx,
      terminalInputRevision,
    ) {
      val scrollGrid = visibleGrid
      object : NestedScrollConnection {
        var remainderY = 0f
        var enteredMultiplexerScrollMode = false

        override fun onPreScroll(available: Offset, source: NestedScrollSource): Offset {
          if (scrollGrid?.activeScreen != TerminalScreenKind.Alternate || available.y == 0f) {
            return Offset.Zero
          }

          remainderY += available.y
          val threshold = terminalRowHeightPx.coerceAtLeast(1f)
          val tickCount = (abs(remainderY) / threshold).toInt().coerceAtMost(6)
          if (tickCount > 0) {
            val direction = if (remainderY > 0) TerminalScrollDirection.Up else TerminalScrollDirection.Down
            val translationSign = if (direction == TerminalScrollDirection.Up) 1f else -1f
            remainderY -= translationSign * tickCount * threshold
            val usesMouseWheel = scrollGrid.mouseReporting && scrollGrid.sgrMouse
            sendTerminalScrollInput(
              scrollGrid.scrollInputSequence(
                direction = direction,
                count = tickCount,
                backend = persistentTerminal?.backend,
                enterScrollMode = !enteredMultiplexerScrollMode,
              ),
            )
            if (!usesMouseWheel && persistentTerminal != null) {
              enteredMultiplexerScrollMode = true
            }
          }

          return Offset(x = 0f, y = available.y)
        }

        override suspend fun onPreFling(available: Velocity): Velocity {
          remainderY = 0f
          return Velocity(x = 0f, y = available.y)
        }
      }
    }

  Column(
    Modifier
      .fillMaxSize()
      .background(displaySettings.terminalTheme.background)
      .onPreviewKeyEvent { event ->
        if (event.type != KeyEventType.KeyDown) {
          return@onPreviewKeyEvent false
        }

        if (event.isCtrlPressed && event.isShiftPressed) {
          when (event.key) {
            Key.C -> {
              clipboard.setText(AnnotatedString(selectedText ?: session.copyableText()))
              return@onPreviewKeyEvent true
            }
            Key.V -> {
              clipboard.getText()?.text?.takeIf { it.isNotEmpty() }?.let(::handlePaste)
              return@onPreviewKeyEvent true
            }
            Key.F -> {
              searchVisible = true
              searchIndex = 0
              return@onPreviewKeyEvent true
            }
          }
        }

        terminalInputForHardwareKey(event, session.isApplicationCursorKeysActive())?.let { payload ->
          sendTerminalInput(payload)
          ctrlArmed = false
          altArmed = false
          return@onPreviewKeyEvent true
        }

        false
      },
  ) {
    val terminalItemCount =
      if (visibleGrid != null) {
        visibleGrid.lines.size
      } else {
        session.rows.size
      }
    val canJumpToBottom = terminalItemCount > 0 && visibleGrid?.activeScreen != TerminalScreenKind.Alternate
    LaunchedEffect(visibleGrid?.activeScreen) {
      if (visibleGrid?.activeScreen == TerminalScreenKind.Alternate && terminalItemCount > 0) {
        terminalListState.scrollToItem(0)
      }
    }
    LaunchedEffect(terminalItemCount, search.isEmpty, canJumpToBottom) {
      if (canJumpToBottom && search.isEmpty) {
        terminalListState.animateScrollToItem(terminalItemCount - 1)
      }
    }
    LaunchedEffect(keyboardOffsetPx, visibleGrid?.cursorRow, terminalItemCount, viewportHeightPx) {
      if (keyboardOffsetPx > 0 && visibleGrid != null && visibleGrid.activeScreen == TerminalScreenKind.Primary && terminalItemCount > 0) {
        val availableRows =
          ((viewportHeightPx.toFloat() - terminalTopInsetPx - terminalBottomInsetPx) / terminalRowHeightPx)
            .toInt()
            .coerceAtLeast(1)
        val cursorRow = visibleGrid.cursorRow.coerceIn(0, terminalItemCount - 1)
        val firstVisible = terminalListState.firstVisibleItemIndex
        val comfortableLast = firstVisible + availableRows - 3
        if (cursorRow < firstVisible || cursorRow > comfortableLast) {
          val target = (cursorRow - availableRows + 3).coerceIn(0, (terminalItemCount - 1).coerceAtLeast(0))
          terminalListState.animateScrollToItem(target)
        }
      }
    }
    Box(
      modifier =
        Modifier
          .weight(1f)
          .fillMaxWidth()
          .clickable { focusTerminalInput() }
          .offset(y = -terminalLiftDp),
    ) {
      if (visibleGrid != null && rustSurfaceEnabled) {
        val surfaceHeightDp = with(density) { (viewportRowCount * terminalRowHeightPx).coerceAtLeast(1f).toDp() }
        AndroidRendererSurfaceHost(
          grid = visibleGrid,
          viewportFirstRow = viewportFirstRow,
          viewportRowCount = viewportRowCount,
          overlayJson = rendererOverlayJson ?: "{\"ranges\":[]}",
          modifier =
            Modifier
              .padding(start = 14.dp, top = terminalTopInsetDp, end = 14.dp, bottom = terminalBottomInsetDp)
              .fillMaxWidth()
              .height(surfaceHeightDp),
          onAttachSurface = onAttachRendererSurface,
          onSetOverlay = onSetRendererOverlay,
          onRenderFrame = onRenderRendererSurface,
          onDetachSurface = onDetachRendererSurface,
          onPresentationChange = { rendererSurfaceReady = it },
        )
      }
      BasicTextField(
        value = directInputValue,
        onValueChange = ::handleDirectInputChange,
        modifier =
          Modifier
            .size(1.dp)
            .alpha(0f)
            .focusRequester(terminalFocusRequester),
        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Text),
      )
      LazyColumn(
        state = terminalListState,
        modifier =
          Modifier
            .fillMaxSize()
            .nestedScroll(alternateScreenScrollConnection)
            .onSizeChanged { size ->
              viewportWidthPx = size.width
              viewportHeightPx = size.height
              reportViewportSize(size.width, size.height)
            }
            .padding(start = 14.dp, top = terminalTopInsetDp, end = 14.dp, bottom = terminalBottomInsetDp),
        verticalArrangement = if (visibleGrid != null) Arrangement.spacedBy(0.dp) else Arrangement.spacedBy(6.dp),
      ) {
        if (visibleGrid != null) {
          itemsIndexed(visibleGrid.lines) { row, line ->
            TerminalGridRow(
              grid = visibleGrid,
              line = line,
              row = row,
              selected = selection.isFullGridRow(row, line),
              selectedCells = selection.gridCellRange(row, line),
              searchMatch = search.containsGrid(row),
              activeSearchMatch = search.activeGridRow == row,
              cellWidthPx = gridCellWidthPx,
              rowHeightPx = terminalRowHeightPx,
              textSizePx = terminalTextSizePx,
              preferRustSurface = rustSurfaceEnabled,
              onTap = {
                focusTerminalInput()
                val mouseInput = grid.mousePressSequence(row = row, column = 0)
                if (mouseInput != null) {
                  selection = null
                  sendTerminalInput(mouseInput)
                }
              },
              onLongPressSelect = { point ->
                focusTerminalInput()
                selection = TerminalSelection.gridRow(point.row)
              },
              onDragSelect = { anchor, focus ->
                selection = TerminalSelection.Grid(anchor, focus)
              },
            )
          }
        } else {
          itemsIndexed(session.rows) { rowIndex, row ->
            TerminalRowView(
              row = row,
              fontSizeSp = displaySettings.fontSizeSp,
              selected = selection.containsHistory(rowIndex),
              searchMatch = search.containsHistory(rowIndex),
              activeSearchMatch = search.activeHit == TerminalSearchHit.History(rowIndex),
              onTap = { focusTerminalInput() },
              onLongPressSelect = {
                focusTerminalInput()
                selection = selection.extendingHistory(rowIndex)
              },
            )
          }
        }
      }
      TerminalFloatingHeader(
        session = session,
        profileName = profileName,
        workspaceName = activePersistentSessionName ?: persistentTerminal?.name,
        workspaceCount = persistentSessionCatalog.sessions.size,
        refreshingWorkspaces = refreshingPersistentSessions,
        onOpenWorkspaceSwitcher =
          if (persistentTerminal == null) null else {
            { persistentSessionsVisible = true }
          },
        onBackToHosts = onBackToHosts,
        onReconnect = onReconnect,
        onDisconnect = onDisconnect,
        modifier =
          Modifier
            .align(Alignment.TopCenter)
            .padding(horizontal = 10.dp, vertical = 8.dp)
            .zIndex(3f),
      )
      if (searchVisible) {
        TerminalSearchBar(
          query = searchQuery,
          onQueryChange = {
            searchQuery = it
            searchIndex = 0
          },
          presentation = search,
          onPrevious = {
            val count = search.hits.size
            if (count > 0) searchIndex = (searchIndex - 1).floorMod(count)
          },
          onNext = {
            val count = search.hits.size
            if (count > 0) searchIndex = (searchIndex + 1).floorMod(count)
          },
          onClose = {
            searchVisible = false
            searchQuery = ""
            searchIndex = 0
          },
          modifier =
            Modifier
              .align(Alignment.TopCenter)
              .padding(start = 10.dp, top = terminalSearchBarTopDp, end = 10.dp)
              .zIndex(2f),
        )
      }
    }
    if (displaySettings.showKeyboardToolbar) {
    Row(
      modifier =
        Modifier
          .fillMaxWidth()
          .background(ShellowColors.PanelBackground)
          .padding(horizontal = 12.dp, vertical = 8.dp)
          .offset(y = -keyboardVisualDeltaDp),
      horizontalArrangement = Arrangement.spacedBy(8.dp),
      verticalAlignment = Alignment.CenterVertically,
    ) {
      Box {
        TerminalToolbarButton("...", accessibilityLabel = "Terminal Tools") { toolsExpanded = true }
        DropdownMenu(
          expanded = toolsExpanded,
          onDismissRequest = { toolsExpanded = false },
        ) {
          if (persistentTerminal != null) {
            DropdownMenuItem(
              text = { Text("${persistentTerminal.backend.displayTitle} Controls") },
              onClick = {
                toolsExpanded = false
                persistentToolsVisible = true
              },
            )
            PanelDivider()
          }
          DropdownMenuItem(
            text = { Text("Clear Terminal") },
            onClick = {
              toolsExpanded = false
              pendingDestructiveAction = TerminalDestructiveAction.Clear
            },
          )
          DropdownMenuItem(
            text = { Text("Reset Terminal") },
            onClick = {
              toolsExpanded = false
              pendingDestructiveAction = TerminalDestructiveAction.Reset
            },
          )
          PanelDivider()
          DropdownMenuItem(
            text = { Text("Save Transcript") },
            onClick = {
              toolsExpanded = false
              transcriptSaveResult =
                runCatching { saveTerminalTranscript(context, session) }
                  .fold(
                    onSuccess = { file -> TranscriptSaveResult("Transcript Saved", file.name) },
                    onFailure = { error -> TranscriptSaveResult("Save Failed", error.message ?: error.toString()) },
                  )
            },
          )
          DropdownMenuItem(
            text = { Text("Copy Terminal") },
            onClick = {
              toolsExpanded = false
              clipboard.setText(AnnotatedString(session.copyableText()))
            },
          )
          DropdownMenuItem(
            text = { Text(if (searchVisible) "Hide Search" else "Search") },
            onClick = {
              toolsExpanded = false
              searchVisible = !searchVisible
            },
          )
          if (canJumpToBottom) {
            DropdownMenuItem(
              text = { Text("Jump To Bottom") },
              onClick = {
                toolsExpanded = false
                terminalScope.launch {
                  terminalListState.animateScrollToItem(terminalItemCount - 1)
                }
              },
            )
          }
          DropdownMenuItem(
            text = { Text("Paste") },
            onClick = {
              toolsExpanded = false
              clipboard.getText()?.text?.takeIf { it.isNotEmpty() }?.let {
                handlePaste(it)
              }
            },
          )
          if (selectedText != null) {
            DropdownMenuItem(
              text = { Text("Copy Selection") },
              onClick = {
                toolsExpanded = false
                clipboard.setText(AnnotatedString(selectedText))
              },
            )
            if (selectedLink != null) {
              DropdownMenuItem(
                text = { Text("Copy Link") },
                onClick = {
                  toolsExpanded = false
                  clipboard.setText(AnnotatedString(selectedLink))
                },
              )
            }
            DropdownMenuItem(
              text = { Text("Clear Selection") },
              onClick = {
                toolsExpanded = false
                selection = null
              },
            )
          }
        }
      }
      TerminalDirectionKeyStrip(
        applicationCursorKeys = session.isApplicationCursorKeysActive(),
        sendInput = { sendToolbarInput(it) },
      )
      Spacer(Modifier.weight(1f))
      TerminalToolbarButton("Enter", accent = true) { sendEnter() }
    }
    Row(
      modifier =
        Modifier
          .fillMaxWidth()
          .background(ShellowColors.PanelBackground)
          .horizontalScroll(rememberScrollState())
          .padding(horizontal = 12.dp, vertical = 6.dp)
          .offset(y = -keyboardVisualDeltaDp),
      horizontalArrangement = Arrangement.spacedBy(8.dp),
      verticalAlignment = Alignment.CenterVertically,
    ) {
      TerminalKey("Esc") { sendToolbarInput("\u001B") }
      TerminalKey("Tab") { sendToolbarInput("\t") }
      TerminalKey("Ctrl", active = ctrlArmed) {
        ctrlArmed = !ctrlArmed
        if (ctrlArmed) altArmed = false
      }
      TerminalKey("Alt", active = altArmed) {
        altArmed = !altArmed
        if (altArmed) ctrlArmed = false
      }
      TerminalKey("^C") { sendToolbarInput("\u0003") }
      TerminalKey("^D") { sendToolbarInput("\u0004") }
      TerminalKey("^L") { sendToolbarInput("\u000C") }
      TerminalKey("^Z") { sendToolbarInput("\u001A") }
      TerminalKey("^A") { sendToolbarInput("\u0001") }
      TerminalKey("^B") { sendToolbarInput("\u0002") }
      TerminalKey("^E") { sendToolbarInput("\u0005") }
      TerminalKey("^K") { sendToolbarInput("\u000B") }
      TerminalKey("^O") { sendToolbarInput("\u000F") }
      TerminalKey("^U") { sendToolbarInput("\u0015") }
      TerminalKey("^W") { sendToolbarInput("\u0017") }
      TerminalKey("^R") { sendToolbarInput("\u0012") }
      TerminalKey("^X") { sendToolbarInput("\u0018") }
      TerminalKey("Up") { sendToolbarInput(TerminalArrowKey.Up.sequence(session.isApplicationCursorKeysActive())) }
      TerminalKey("Down") { sendToolbarInput(TerminalArrowKey.Down.sequence(session.isApplicationCursorKeysActive())) }
      TerminalKey("Left") { sendToolbarInput(TerminalArrowKey.Left.sequence(session.isApplicationCursorKeysActive())) }
      TerminalKey("Right") { sendToolbarInput(TerminalArrowKey.Right.sequence(session.isApplicationCursorKeysActive())) }
      TerminalKey("Del") { sendToolbarInput("\u007F") }
      TerminalKey("Home") { sendToolbarInput("\u001B[H") }
      TerminalKey("End") { sendToolbarInput("\u001B[F") }
      TerminalKey("PgUp") { sendToolbarInput("\u001B[5~") }
      TerminalKey("PgDn") { sendToolbarInput("\u001B[6~") }
      TerminalFunctionKey.entries.forEach { key ->
        TerminalKey(key.label) { sendToolbarInput(key.sequence) }
      }
    }
    }
    Spacer(Modifier.height(6.dp + keyboardLayoutOffsetDp).fillMaxWidth().background(ShellowColors.PanelBackground))
  }

  pendingPaste?.let { paste ->
    AlertDialog(
      onDismissRequest = { pendingPaste = null },
      containerColor = ShellowColors.PanelBackground,
      titleContentColor = ShellowColors.TerminalText,
      textContentColor = ShellowColors.TerminalText,
      title = { Text("Confirm Paste") },
      text = { Text("Send ${paste.terminalPasteLineCount()} lines and ${paste.length} characters to the terminal?") },
      confirmButton = {
        TextButton(
          onClick = {
            pendingPaste = null
            sendPaste(paste)
          },
        ) { Text("Paste") }
      },
      dismissButton = {
        TextButton(onClick = { pendingPaste = null }) { Text("Cancel") }
      },
    )
  }

  pendingDestructiveAction?.let { action ->
    val isClear = action == TerminalDestructiveAction.Clear
    AlertDialog(
      onDismissRequest = { pendingDestructiveAction = null },
      containerColor = ShellowColors.PanelBackground,
      titleContentColor = ShellowColors.TerminalText,
      textContentColor = ShellowColors.TerminalText,
      title = { Text(if (isClear) "Clear terminal?" else "Reset terminal?") },
      text = {
        Text(
          if (isClear) "The visible terminal history will be removed."
          else "The terminal display and input state will be reset.",
        )
      },
      confirmButton = {
        TextButton(
          onClick = {
            pendingDestructiveAction = null
            if (isClear) onClearTerminal() else onResetTerminal()
          },
        ) {
          Text(if (isClear) "Clear" else "Reset", color = MaterialTheme.colorScheme.error)
        }
      },
      dismissButton = {
        TextButton(onClick = { pendingDestructiveAction = null }) { Text("Cancel") }
      },
    )
  }

  pendingRemoteClipboard?.let { request ->
    AlertDialog(
      onDismissRequest = { pendingRemoteClipboard = null },
      containerColor = ShellowColors.PanelBackground,
      titleContentColor = ShellowColors.TerminalText,
      textContentColor = ShellowColors.TerminalText,
      title = { Text("Remote Clipboard") },
      text = {
        Text("Copy ${request.text.terminalPasteLineCount()} lines and ${request.text.length} characters from the remote terminal?")
      },
      confirmButton = {
        TextButton(
          onClick = {
            pendingRemoteClipboard = null
            clipboard.setText(AnnotatedString(request.text))
          },
        ) { Text("Copy") }
      },
      dismissButton = {
        TextButton(onClick = { pendingRemoteClipboard = null }) { Text("Cancel") }
      },
    )
  }

  transcriptSaveResult?.let { result ->
    AlertDialog(
      onDismissRequest = { transcriptSaveResult = null },
      containerColor = ShellowColors.PanelBackground,
      titleContentColor = ShellowColors.TerminalText,
      textContentColor = ShellowColors.TerminalText,
      title = { Text(result.title) },
      text = { Text(result.message) },
      confirmButton = {
        TextButton(onClick = { transcriptSaveResult = null }) { Text("OK") }
      },
    )
  }

  if (persistentSessionsVisible && persistentTerminal != null) {
    TerminalSessionSwitcherDialog(
      profileName = profileName,
      configuration = persistentTerminal,
      catalog = persistentSessionCatalog,
      activeSessionName = activePersistentSessionName ?: persistentTerminal.name,
      refreshing = refreshingPersistentSessions,
      onDismiss = { persistentSessionsVisible = false },
      onRefresh = { terminalScope.launch { refreshPersistentSessions() } },
      onSwitchSession = ::switchPersistentSession,
      onOpenControls = {
        persistentSessionsVisible = false
        persistentToolsVisible = true
      },
    )
  }

  if (persistentToolsVisible && persistentTerminal != null) {
    PersistentTerminalControlDialog(
      configuration = persistentTerminal,
      onDismiss = { persistentToolsVisible = false },
      onSend = { sequence ->
        persistentToolsVisible = false
        terminalScope.launch {
          delay(120)
          sendTerminalInput(sequence)
        }
      },
      onSwitchSession = { name ->
        switchPersistentSession(name)
      },
    )
  }
}

@Composable
private fun TerminalSessionSwitcherDialog(
  profileName: String,
  configuration: PersistentTerminalConfiguration,
  catalog: RemoteTerminalSessionCatalog,
  activeSessionName: String,
  refreshing: Boolean,
  onDismiss: () -> Unit,
  onRefresh: () -> Unit,
  onSwitchSession: (String) -> Unit,
  onOpenControls: () -> Unit,
) {
  var newSessionName by remember(configuration.name) {
    mutableStateOf(
      (configuration.name.take(PersistentTerminalConfiguration.MaximumNameLength - 2) + "-2"),
    )
  }
  val validatedNewSessionName = PersistentTerminalConfiguration.validatedName(newSessionName)

  AlertDialog(
    onDismissRequest = onDismiss,
    containerColor = ShellowColors.PanelBackground,
    titleContentColor = ShellowColors.TerminalText,
    textContentColor = ShellowColors.TerminalText,
    title = { Text("Sessions") },
    text = {
      Column(
        modifier = Modifier.heightIn(max = 520.dp).verticalScroll(rememberScrollState()),
        verticalArrangement = Arrangement.spacedBy(10.dp),
      ) {
        Text(
          "${configuration.backend.displayTitle} on $profileName",
          color = ShellowColors.TerminalMuted,
          style = MaterialTheme.typography.labelSmall,
        )

        when {
          refreshing && catalog.sessions.isEmpty() -> {
            Row(horizontalArrangement = Arrangement.spacedBy(10.dp), verticalAlignment = Alignment.CenterVertically) {
              CircularProgressIndicator(modifier = Modifier.size(18.dp), strokeWidth = 2.dp)
              Text("Loading sessions…", color = ShellowColors.TerminalMuted)
            }
          }
          catalog.sessions.isEmpty() -> {
            Text(
              catalog.errorMessage ?: "No remote sessions yet.",
              color = ShellowColors.TerminalMuted,
              style = MaterialTheme.typography.bodySmall,
            )
          }
          else -> {
            catalog.sessions.forEach { remoteSession ->
              Row(
                modifier =
                  Modifier
                    .fillMaxWidth()
                    .background(ShellowColors.KeyBackground.copy(alpha = 0.42f), RoundedCornerShape(10.dp))
                    .clickable(
                      enabled = remoteSession.name != activeSessionName,
                      onClick = { onSwitchSession(remoteSession.name) },
                    )
                    .padding(horizontal = 12.dp, vertical = 10.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(10.dp),
              ) {
                Text(
                  if (remoteSession.name == activeSessionName) "●" else "○",
                  color = if (remoteSession.name == activeSessionName) ShellowColors.Accent else ShellowColors.TerminalMuted,
                )
                Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                  Text(
                    remoteSession.name,
                    color = ShellowColors.TerminalText,
                    fontFamily = FontFamily.Monospace,
                    fontWeight = FontWeight.SemiBold,
                  )
                  Text(
                    remoteSession.windowCount?.let { count -> "$count ${if (count == 1) "window" else "windows"}" }
                      ?: if (remoteSession.isAttached) "Active remote workspace" else "Available remote workspace",
                    color = ShellowColors.TerminalMuted,
                    style = MaterialTheme.typography.labelSmall,
                  )
                }
                if (remoteSession.isAttached) {
                  Text("Attached", color = ShellowColors.Success, style = MaterialTheme.typography.labelSmall)
                }
              }
            }
          }
        }

        PanelDivider()
        Text("New session", color = ShellowColors.TerminalText, fontWeight = FontWeight.SemiBold)
        OutlinedTextField(
          value = newSessionName,
          onValueChange = { newSessionName = it },
          modifier = Modifier.fillMaxWidth(),
          label = { Text("Session name") },
          isError = validatedNewSessionName == null,
          singleLine = true,
        )
        TextButton(
          enabled = validatedNewSessionName != null,
          onClick = { validatedNewSessionName?.let(onSwitchSession) },
        ) {
          Text("Create and switch")
        }
      }
    },
    confirmButton = { TextButton(onClick = onDismiss) { Text("Done") } },
    dismissButton = {
      Row {
        TextButton(enabled = !refreshing, onClick = onRefresh) { Text("Refresh") }
        TextButton(onClick = onOpenControls) { Text("Controls") }
      }
    },
  )
}

private data class PersistentTerminalControl(
  val title: String,
  val sequence: String,
)

@Composable
private fun PersistentTerminalControlDialog(
  configuration: PersistentTerminalConfiguration,
  onDismiss: () -> Unit,
  onSend: (String) -> Unit,
  onSwitchSession: (String) -> Unit,
) {
  var targetSession by remember(configuration.name) { mutableStateOf(configuration.name) }
  val validatedTarget = PersistentTerminalConfiguration.validatedName(targetSession)
  val controls =
    when (configuration.backend) {
      PersistentTerminalBackend.Tmux ->
        listOf(
          PersistentTerminalControl("Choose session", "\u0002s"),
          PersistentTerminalControl("New window", "\u0002c"),
          PersistentTerminalControl("Previous window", "\u0002p"),
          PersistentTerminalControl("Next window", "\u0002n"),
          PersistentTerminalControl("Split left / right", "\u0002%"),
          PersistentTerminalControl("Split top / bottom", "\u0002\""),
          PersistentTerminalControl("Command prompt", "\u0002:"),
          PersistentTerminalControl("Detach", "\u0002d"),
        )
      PersistentTerminalBackend.Screen ->
        listOf(
          PersistentTerminalControl("Window list", "\u0001\""),
          PersistentTerminalControl("New window", "\u0001c"),
          PersistentTerminalControl("Previous window", "\u0001p"),
          PersistentTerminalControl("Next window", "\u0001n"),
          PersistentTerminalControl("Split top / bottom", "\u0001S"),
          PersistentTerminalControl("Next region", "\u0001\t"),
          PersistentTerminalControl("Command prompt", "\u0001:"),
          PersistentTerminalControl("Detach", "\u0001d"),
        )
      PersistentTerminalBackend.Zellij ->
        listOf(
          PersistentTerminalControl("Session manager", "\u000Fw"),
          PersistentTerminalControl("New tab", "\u0014n"),
          PersistentTerminalControl("Previous tab", "\u0014h"),
          PersistentTerminalControl("Next tab", "\u0014l"),
          PersistentTerminalControl("Rename tab", "\u0014r"),
          PersistentTerminalControl("Split right", "\u0010r"),
          PersistentTerminalControl("Split down", "\u0010d"),
          PersistentTerminalControl("Detach", "\u000Fd"),
        )
    }

  AlertDialog(
    onDismissRequest = onDismiss,
    containerColor = ShellowColors.PanelBackground,
    titleContentColor = ShellowColors.TerminalText,
    textContentColor = ShellowColors.TerminalText,
    title = { Text("${configuration.backend.displayTitle} · ${configuration.name}") },
    text = {
      Column(
        modifier = Modifier.heightIn(max = 520.dp).verticalScroll(rememberScrollState()),
        verticalArrangement = Arrangement.spacedBy(8.dp),
      ) {
        Text(
          "Commands use ${configuration.backend.controlPrefixLabel}. The active process remains on the target host after detach.",
          color = ShellowColors.TerminalMuted,
          style = MaterialTheme.typography.labelSmall,
        )
        controls.chunked(2).forEach { rowControls ->
          Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            rowControls.forEach { control ->
              TextButton(
                onClick = { onSend(control.sequence) },
                modifier =
                  Modifier
                    .weight(1f)
                    .background(ShellowColors.KeyBackground.copy(alpha = 0.42f), RoundedCornerShape(8.dp)),
              ) {
                Text(control.title, maxLines = 1, overflow = TextOverflow.Ellipsis)
              }
            }
            if (rowControls.size == 1) Spacer(Modifier.weight(1f))
          }
        }
        PanelDivider()
        Text("Switch or create session", color = ShellowColors.TerminalText, fontWeight = FontWeight.SemiBold)
        OutlinedTextField(
          value = targetSession,
          onValueChange = { targetSession = it },
          modifier = Modifier.fillMaxWidth(),
          label = { Text("Session name") },
          isError = validatedTarget == null,
          supportingText = { Text("Detaches the current session, then attaches or creates this name.") },
          singleLine = true,
        )
        TextButton(
          enabled = validatedTarget != null,
          onClick = { validatedTarget?.let(onSwitchSession) },
        ) {
          Text("Switch / Create")
        }
      }
    },
    confirmButton = { TextButton(onClick = onDismiss) { Text("Done") } },
  )
}

@Composable
private fun TerminalDirectionKeyStrip(
  applicationCursorKeys: Boolean,
  sendInput: (String) -> Unit,
) {
  Row(
    horizontalArrangement = Arrangement.spacedBy(6.dp),
    verticalAlignment = Alignment.CenterVertically,
  ) {
    TerminalCompactButton("↑", width = 34.dp, accessibilityLabel = "Arrow Up") {
      sendInput(TerminalArrowKey.Up.sequence(applicationCursorKeys))
    }
    TerminalCompactButton("↓", width = 34.dp, accessibilityLabel = "Arrow Down") {
      sendInput(TerminalArrowKey.Down.sequence(applicationCursorKeys))
    }
    TerminalCompactButton("←", width = 34.dp, accessibilityLabel = "Arrow Left") {
      sendInput(TerminalArrowKey.Left.sequence(applicationCursorKeys))
    }
    TerminalCompactButton("→", width = 34.dp, accessibilityLabel = "Arrow Right") {
      sendInput(TerminalArrowKey.Right.sequence(applicationCursorKeys))
    }
  }
}

@Composable
private fun TerminalToolbarButton(
  label: String,
  accent: Boolean = false,
  accessibilityLabel: String = label,
  onClick: () -> Unit,
) {
  TerminalCompactButton(
    label = label,
    active = accent,
    width = terminalToolbarButtonWidth(label),
    accessibilityLabel = accessibilityLabel,
    onClick = onClick,
  )
}

@Composable
private fun TerminalCompactButton(
  label: String,
  active: Boolean = false,
  width: androidx.compose.ui.unit.Dp,
  accessibilityLabel: String = label,
  onClick: () -> Unit,
) {
  Box(
    modifier =
      Modifier
        .width(width.coerceAtLeast(48.dp))
        .height(48.dp)
        .background(
          if (active) ShellowColors.Accent else ShellowColors.KeyBackground,
          RoundedCornerShape(8.dp),
        )
        .semantics { contentDescription = accessibilityLabel }
        .clickable(onClick = onClick),
    contentAlignment = Alignment.Center,
  ) {
    Text(
      label,
      color = ShellowColors.TerminalText,
      style = MaterialTheme.typography.labelMedium,
      maxLines = 1,
      overflow = TextOverflow.Ellipsis,
    )
  }
}

private fun terminalToolbarButtonWidth(label: String): androidx.compose.ui.unit.Dp =
  when {
    label.length >= 8 -> 74.dp
    label.length >= 6 -> 64.dp
    label.length >= 5 -> 56.dp
    else -> 48.dp
  }

private fun terminalKeyWidth(label: String): androidx.compose.ui.unit.Dp =
  when {
    label.length >= 5 -> 54.dp
    label.length >= 4 -> 48.dp
    else -> 42.dp
  }

@Composable
private fun TerminalKey(
  label: String,
  active: Boolean = false,
  action: () -> Unit,
) {
  TerminalCompactButton(
    label = label,
    active = active,
    width = terminalKeyWidth(label),
    onClick = action,
  )
}

@Composable
private fun TerminalFloatingHeader(
  session: TerminalSession,
  profileName: String,
  workspaceName: String?,
  workspaceCount: Int,
  refreshingWorkspaces: Boolean,
  onOpenWorkspaceSwitcher: (() -> Unit)?,
  onBackToHosts: () -> Unit,
  onReconnect: (() -> Unit)?,
  onDisconnect: () -> Unit,
  modifier: Modifier = Modifier,
) {
  Row(
    modifier =
      modifier
        .fillMaxWidth()
        .background(ShellowColors.PanelBackground.copy(alpha = 0.94f), RoundedCornerShape(12.dp))
        .padding(horizontal = 10.dp, vertical = 8.dp),
    verticalAlignment = Alignment.CenterVertically,
    horizontalArrangement = Arrangement.spacedBy(9.dp),
  ) {
    NavigationBackButton(onClick = onBackToHosts)
    Box(
      modifier =
        Modifier
          .size(9.dp)
          .background(statusColor(session.state), RoundedCornerShape(9.dp))
    )
    Column(
      modifier =
        Modifier
          .weight(1f)
          .clickable(enabled = onOpenWorkspaceSwitcher != null) { onOpenWorkspaceSwitcher?.invoke() },
      verticalArrangement = Arrangement.spacedBy(1.dp),
    ) {
      Text(
        workspaceName ?: session.title,
        color = ShellowColors.TerminalText,
        style = MaterialTheme.typography.titleSmall,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
      )
      Text(
        when {
          refreshingWorkspaces && workspaceCount == 0 -> "$profileName · Loading sessions"
          onOpenWorkspaceSwitcher != null -> "$profileName · $workspaceCount ${if (workspaceCount == 1) "session" else "sessions"} ▾"
          else -> profileName
        },
        color = ShellowColors.TerminalMuted,
        style = MaterialTheme.typography.labelSmall,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
      )
    }
    Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
      if (session.bellCount > 0) {
        Text(
          "Bell ${session.bellCount}",
          color = ShellowColors.Warning,
          style = MaterialTheme.typography.labelMedium,
          modifier =
            Modifier
              .background(ShellowColors.Warning.copy(alpha = 0.16f), RoundedCornerShape(12.dp))
              .padding(horizontal = 9.dp, vertical = 5.dp),
          )
      }
      if (session.state == ConnectionState.Disconnected && onReconnect != null) {
        TextButton(
          onClick = onReconnect,
          modifier = Modifier.background(ShellowColors.Accent.copy(alpha = 0.16f), RoundedCornerShape(8.dp)),
        ) {
          Text("Reconnect", color = ShellowColors.Accent, style = MaterialTheme.typography.labelMedium)
        }
      }
      if (session.state != ConnectionState.Disconnected) {
        TextButton(
          onClick = onDisconnect,
          modifier = Modifier.background(ShellowColors.Warning.copy(alpha = 0.16f), RoundedCornerShape(8.dp)),
        ) {
          Text("Stop", color = ShellowColors.Warning, style = MaterialTheme.typography.labelMedium)
        }
      }
    }
  }
}

@Composable
private fun TerminalSearchBar(
  query: String,
  onQueryChange: (String) -> Unit,
  presentation: TerminalSearchPresentation,
  onPrevious: () -> Unit,
  onNext: () -> Unit,
  onClose: () -> Unit,
  modifier: Modifier = Modifier,
) {
  Row(
    modifier =
      modifier
        .fillMaxWidth()
        .background(ShellowColors.PanelBackground.copy(alpha = 0.96f), RoundedCornerShape(12.dp))
        .padding(horizontal = 12.dp, vertical = 8.dp),
    verticalAlignment = Alignment.CenterVertically,
    horizontalArrangement = Arrangement.spacedBy(8.dp),
  ) {
    TextField(
      value = query,
      onValueChange = onQueryChange,
      modifier = Modifier.weight(1f),
      singleLine = true,
      placeholder = { Text("Search") },
      textStyle = MaterialTheme.typography.bodyMedium.copy(fontFamily = FontFamily.Monospace),
    )
    Text(
      text =
        when {
          presentation.isEmpty -> ""
          presentation.hits.isEmpty() -> "0/0"
          else -> "${presentation.activeOrdinal}/${presentation.hits.size}"
        },
      color = ShellowColors.TerminalMuted,
      style = MaterialTheme.typography.labelMedium,
    )
    TextButton(onClick = onPrevious) { Text("Prev") }
    TextButton(onClick = onNext) { Text("Next") }
    TextButton(onClick = onClose) { Text("Close") }
  }
}

@Composable
private fun AndroidRendererSurfaceHost(
  grid: TerminalGridSnapshot,
  viewportFirstRow: Int,
  viewportRowCount: Int,
  overlayJson: String,
  modifier: Modifier = Modifier,
  onAttachSurface: (Surface, Int, Int) -> String,
  onSetOverlay: (String) -> String,
  onRenderFrame: (Int, Int, Int, Int) -> Boolean,
  onDetachSurface: () -> String,
  onPresentationChange: (Boolean) -> Unit,
) {
  val surfaceViewState = remember { mutableStateOf<SurfaceView?>(null) }
  val lastAppliedOverlayJson = remember { arrayOf<String?>(null) }
  val renderSignature = androidRendererSurfaceSignature(grid, viewportFirstRow, viewportRowCount, overlayJson)

  fun renderIfReady(view: SurfaceView?): Boolean {
    val surface = view?.holder?.surface ?: return false
    val width = view.width
    val height = view.height
    if (!surface.isValid || width <= 0 || height <= 0) {
      return false
    }

    if (lastAppliedOverlayJson[0] != overlayJson) {
      onSetOverlay(overlayJson)
      lastAppliedOverlayJson[0] = overlayJson
    }
    val presented = onRenderFrame(width, height, viewportFirstRow, viewportRowCount)
    if (presented) {
      if (BuildConfig.DEBUG) Log.i(RendererLogTag, "Shellow renderer Android terminal surface frame ${width}x$height")
    }
    onPresentationChange(presented)
    return presented
  }

  DisposableEffect(Unit) {
    onDispose {
      onPresentationChange(false)
      if (BuildConfig.DEBUG) Log.i(RendererLogTag, "Shellow renderer Android surface detach ${onDetachSurface()}")
    }
  }

  LaunchedEffect(renderSignature, surfaceViewState.value) {
    surfaceViewState.value?.post {
      renderIfReady(surfaceViewState.value)
    }
  }

  AndroidView(
    modifier = modifier,
    factory = { context ->
      SurfaceView(context).apply {
        isClickable = false
        isFocusable = false
        isFocusableInTouchMode = false
        setZOrderOnTop(false)
        holder.addCallback(
          object : SurfaceHolder.Callback {
            override fun surfaceCreated(holder: SurfaceHolder) = Unit

            override fun surfaceChanged(
              holder: SurfaceHolder,
              format: Int,
              width: Int,
              height: Int,
            ) {
              if (width <= 0 || height <= 0 || !holder.surface.isValid) {
                onPresentationChange(false)
                return
              }

              val attachResponse = onAttachSurface(holder.surface, width, height)
              if (BuildConfig.DEBUG) Log.i(RendererLogTag, "Shellow renderer Android surface attach $attachResponse")
              renderIfReady(this@apply)
            }

            override fun surfaceDestroyed(holder: SurfaceHolder) {
              onPresentationChange(false)
              if (BuildConfig.DEBUG) Log.i(RendererLogTag, "Shellow renderer Android surface detach ${onDetachSurface()}")
            }
          },
        )
        surfaceViewState.value = this
      }
    },
    update = { view ->
      surfaceViewState.value = view
      view.post {
        renderIfReady(view)
      }
    },
  )
}

private fun androidRendererSurfaceSignature(
  grid: TerminalGridSnapshot,
  viewportFirstRow: Int,
  viewportRowCount: Int,
  overlayJson: String,
): String =
  listOf(
    grid.cols,
    grid.rows,
    viewportFirstRow,
    viewportRowCount,
    grid.activeScreen.wire,
    grid.scrollbackLen,
    grid.lines.hashCode(),
    grid.styledLines.hashCode(),
    grid.cursorColumn,
    grid.cursorRow,
    grid.cursorVisible,
    grid.cursorShape.wire,
    grid.dirtyRows.hashCode(),
    overlayJson.hashCode(),
  ).joinToString("|")

private fun androidRendererOverlayJson(
  grid: TerminalGridSnapshot,
  selection: TerminalSelection?,
  search: TerminalSearchPresentation,
  viewportFirstRow: Int,
  viewportRowCount: Int,
): String {
  val ranges = JSONArray()
  val visibleRows = viewportFirstRow until (viewportFirstRow + viewportRowCount).coerceAtMost(grid.lines.size)

  visibleRows.forEach { row ->
    val line = grid.lines.getOrNull(row) ?: return@forEach
    val selectedCells = selection.gridCellRange(row, line)
    if (selectedCells != null) {
      ranges.put(rendererOverlayRangeJson("selection", row - viewportFirstRow, selectedCells.start, selectedCells.endExclusive))
    }
  }

  visibleRows.forEach { row ->
    search.gridRanges(row).forEach { range ->
      ranges.put(rendererOverlayRangeJson("search", row - viewportFirstRow, range.start, range.endExclusive))
    }
  }

  search.activeGridRow?.let { row ->
    if (row in visibleRows) {
      search.activeGridRange?.let { range ->
        ranges.put(rendererOverlayRangeJson("active_search", row - viewportFirstRow, range.start, range.endExclusive))
      }
    }
  }

  return JSONObject().put("ranges", ranges).toString()
}

private fun rendererOverlayRangeJson(
  kind: String,
  row: Int,
  startCol: Int,
  endCol: Int,
): JSONObject =
  JSONObject()
    .put("kind", kind)
    .put("row", row.coerceAtLeast(0))
    .put("start_col", startCol.coerceAtLeast(0))
    .put("end_col", endCol.coerceAtLeast(0))

@Composable
private fun TerminalGridRow(
  grid: TerminalGridSnapshot,
  line: String,
  row: Int,
  selected: Boolean,
  selectedCells: TerminalCellRange?,
  searchMatch: Boolean,
  activeSearchMatch: Boolean,
  cellWidthPx: Float,
  rowHeightPx: Float,
  textSizePx: Float,
  preferRustSurface: Boolean,
  onTap: () -> Unit,
  onLongPressSelect: (TerminalSelectionPoint) -> Unit,
  onDragSelect: (TerminalSelectionPoint, TerminalSelectionPoint) -> Unit,
) {
  val density = LocalDensity.current
  val rowHeightDp = with(density) { rowHeightPx.toDp() }
  val rowModifier =
    Modifier
      .fillMaxWidth()
      .height(rowHeightDp)
      .pointerInput(row, line, cellWidthPx, rowHeightPx, grid.lines.size) {
        var anchor: TerminalSelectionPoint? = null
        detectDragGesturesAfterLongPress(
          onDragStart = { offset ->
            val start = terminalSelectionPointFromOffset(
              offset = offset,
              initialRow = row,
              lines = grid.lines,
              cellWidthPx = cellWidthPx,
              rowHeightPx = rowHeightPx,
            )
            anchor = start
            onLongPressSelect(start)
          },
          onDrag = { change, _ ->
            val start = anchor ?: return@detectDragGesturesAfterLongPress
            val focus = terminalSelectionPointFromOffset(
              offset = change.position,
              initialRow = row,
              lines = grid.lines,
              cellWidthPx = cellWidthPx,
              rowHeightPx = rowHeightPx,
            )
            onDragSelect(start, focus)
            change.consume()
          },
          onDragEnd = { anchor = null },
          onDragCancel = { anchor = null },
        )
      }
      .clickable(onClick = onTap)

  if (preferRustSurface) {
    Box(modifier = rowModifier)
    return
  }

  val xPaddingPx = with(density) { 3.dp.toPx() }
  val rowBackground = searchableRowBackground(selected, searchMatch, activeSearchMatch)
  val renderSignature = terminalGridCanvasRowSignature(grid, line, row)
  val dirtyGeneration = if (grid.dirtyRows.contains(row)) grid.dirtyRows.hashCode() else 0
  val renderPlan =
    remember(renderSignature, dirtyGeneration) {
      terminalGridCanvasRowPlan(grid, line, row)
    }

  Canvas(modifier = rowModifier) {
    drawRect(rowBackground, size = Size(size.width, size.height))
    selectedCells?.let { range ->
      drawRect(
        color = ShellowColors.Accent.copy(alpha = 0.34f),
        topLeft = Offset(xPaddingPx + range.start * cellWidthPx, 0f),
        size = Size((range.endExclusive - range.start).coerceAtLeast(0) * cellWidthPx, size.height),
      )
    }

    drawIntoCanvas { canvas ->
      val nativeCanvas = canvas.nativeCanvas
      val textPaint =
        Paint(Paint.ANTI_ALIAS_FLAG).apply {
          textSize = textSizePx
          typeface = Typeface.MONOSPACE
        }
      val backgroundPaint = Paint()
      val baseline = (size.height - (textPaint.descent() + textPaint.ascent())) / 2f
      renderPlan.runs.forEach { run ->
        nativeCanvas.drawTerminalRun(
          text = run.text,
          style = run.style,
          consumedCells = run.consumedCells,
          cellWidthPx = cellWidthPx,
          xPaddingPx = xPaddingPx,
          rowHeightPx = size.height,
          baseline = baseline,
          textPaint = textPaint,
          backgroundPaint = backgroundPaint,
        )
      }
    }
  }
}

@Composable
private fun TerminalRowView(
  row: TerminalRow,
  fontSizeSp: Float,
  selected: Boolean = false,
  searchMatch: Boolean = false,
  activeSearchMatch: Boolean = false,
  onTap: () -> Unit = {},
  onLongPressSelect: () -> Unit = {},
) {
  Row(
    modifier =
      Modifier
        .fillMaxWidth()
        .background(searchableRowBackground(selected, searchMatch, activeSearchMatch))
        .pointerInput(Unit) {
          detectTapGestures(
            onTap = { onTap() },
            onLongPress = { onLongPressSelect() },
          )
        }
        .padding(horizontal = 3.dp, vertical = 1.dp),
    verticalAlignment = Alignment.Top,
  ) {
    Text(
      text = row.prompt,
      modifier = Modifier.width(16.dp),
      color = ShellowColors.Success,
      fontFamily = FontFamily.Monospace,
      fontSize = fontSizeSp.sp,
    )
    Text(
      text = row.text.ifEmpty { " " },
      color =
        when (row.style) {
          TerminalRowStyle.Command, TerminalRowStyle.Prompt -> ShellowColors.TerminalText
          TerminalRowStyle.Muted -> ShellowColors.TerminalMuted
          TerminalRowStyle.Success -> ShellowColors.Success
          TerminalRowStyle.Warning -> ShellowColors.Warning
        },
      fontFamily = FontFamily.Monospace,
      style = MaterialTheme.typography.bodyMedium.copy(fontSize = fontSizeSp.sp),
    )
  }
}

private fun searchableRowBackground(
  selected: Boolean,
  searchMatch: Boolean,
  activeSearchMatch: Boolean,
): ComposeColor =
  when {
    selected -> ShellowColors.Accent.copy(alpha = 0.34f)
    activeSearchMatch -> ShellowColors.Warning.copy(alpha = 0.46f)
    searchMatch -> ShellowColors.Warning.copy(alpha = 0.24f)
    else -> ComposeColor.Transparent
  }

private fun terminalSelectionPointFromOffset(
  offset: Offset,
  initialRow: Int,
  lines: List<String>,
  cellWidthPx: Float,
  rowHeightPx: Float,
): TerminalSelectionPoint {
  val maxRow = (lines.size - 1).coerceAtLeast(0)
  val rowOffset = floor(offset.y / rowHeightPx).toInt()
  val row = (initialRow + rowOffset).coerceIn(0, maxRow)
  val lineEnd = lines.getOrNull(row)?.terminalCellWidth() ?: 0
  val column = floor((offset.x - 3f) / cellWidthPx).toInt().coerceIn(0, lineEnd)
  return TerminalSelectionPoint(row, column)
}

private enum class HostsHeaderActionIcon {
  Add,
  More,
}

@Composable
private fun HostsHeaderActionButton(
  icon: HostsHeaderActionIcon,
  contentDescription: String,
  onClick: () -> Unit,
) {
  IconButton(
    onClick = onClick,
    modifier =
      Modifier
        .size(40.dp)
        .semantics { this.contentDescription = contentDescription },
  ) {
    Canvas(Modifier.size(18.dp)) {
      val color = ShellowColors.Accent
      val strokeWidth = 1.8.dp.toPx()
      when (icon) {
        HostsHeaderActionIcon.Add -> {
          val arm = 5.dp.toPx()
          drawLine(
            color = color,
            start = Offset(center.x - arm, center.y),
            end = Offset(center.x + arm, center.y),
            strokeWidth = strokeWidth,
            cap = StrokeCap.Round,
          )
          drawLine(
            color = color,
            start = Offset(center.x, center.y - arm),
            end = Offset(center.x, center.y + arm),
            strokeWidth = strokeWidth,
            cap = StrokeCap.Round,
          )
        }
        HostsHeaderActionIcon.More -> {
          val spacing = 5.dp.toPx()
          val radius = 1.4.dp.toPx()
          drawCircle(color = color, radius = radius, center = Offset(center.x - spacing, center.y))
          drawCircle(color = color, radius = radius, center = center)
          drawCircle(color = color, radius = radius, center = Offset(center.x + spacing, center.y))
        }
      }
    }
  }
}

@Composable
private fun HostsScreen(
  profiles: List<HostProfile>,
  sshKeys: List<SSHKeyCredential>,
  secretStore: SSHSecretStore,
  onOpenSettings: () -> Unit,
  onAddProfile: (HostProfile) -> Unit,
  onUpdateProfile: (HostProfile) -> Unit,
  onDeleteProfile: (HostProfile) -> Unit,
  onProbeCapabilities: suspend (HostProfile) -> RemoteHostProbeOutcome,
  onAddKey: (SSHKeyCredential) -> Unit,
  onDeleteKey: (SSHKeyCredential) -> Unit,
  onConnectTerminal: (HostProfile) -> Unit,
  onConnectCodex: (HostProfile) -> Unit,
  onConnectClaude: (HostProfile) -> Unit,
) {
  var addingProfile by remember { mutableStateOf(false) }
  var managingKeys by remember { mutableStateOf(false) }
  var selectedProfile by remember { mutableStateOf<HostProfile?>(null) }
  var manageMenuExpanded by remember { mutableStateOf(false) }

  LazyColumn(
    modifier =
      Modifier
        .fillMaxSize()
        .background(ShellowColors.TerminalBackground)
        .padding(16.dp),
    verticalArrangement = Arrangement.spacedBy(10.dp),
  ) {
    item {
      Row(
        modifier = Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(8.dp),
      ) {
        Text(
          "Profiles",
          modifier = Modifier.weight(1f),
          color = ShellowColors.TerminalText,
          style = MaterialTheme.typography.titleLarge,
        )
        HostsHeaderActionButton(
          icon = HostsHeaderActionIcon.Add,
          contentDescription = "Add Profile",
          onClick = { addingProfile = true },
        )
        Box {
          HostsHeaderActionButton(
            icon = HostsHeaderActionIcon.More,
            contentDescription = "Manage",
            onClick = { manageMenuExpanded = true },
          )
          DropdownMenu(
            expanded = manageMenuExpanded,
            onDismissRequest = { manageMenuExpanded = false },
          ) {
            DropdownMenuItem(
              text = { Text("Settings") },
              onClick = {
                manageMenuExpanded = false
                onOpenSettings()
              },
            )
            DropdownMenuItem(
              text = { Text("SSH Keys") },
              onClick = {
                manageMenuExpanded = false
                managingKeys = true
              },
            )
          }
        }
      }
    }

    item {
      if (profiles.isEmpty()) {
        EmptyHostsPanel(onAddHost = { addingProfile = true })
      } else {
        Column(
          modifier =
            Modifier
              .fillMaxWidth()
              .background(ShellowColors.PanelBackground, RoundedCornerShape(8.dp)),
        ) {
          profiles.forEachIndexed { index, profile ->
            HostProfileRow(
              profile = profile,
              onOpen = {
                when (profile.launchKind) {
                  ProfileLaunchKind.Terminal -> onConnectTerminal(profile)
                  ProfileLaunchKind.Codex -> onConnectCodex(profile)
                  ProfileLaunchKind.Claude -> onConnectClaude(profile)
                }
              },
              onEdit = { selectedProfile = profile },
              onDuplicate = {
                onAddProfile(profile.duplicated(profiles.map { it.name }))
              },
            )
            if (index < profiles.lastIndex) {
              PanelDivider()
            }
          }
        }
      }
    }
  }

  if (addingProfile) {
    AddHostDialog(
      onDismiss = { addingProfile = false },
      onAdd = { profile ->
        addingProfile = false
        onAddProfile(profile)
      },
    )
  }

  if (managingKeys) {
    SSHKeyManagementDialog(
      credentials = sshKeys,
      secretStore = secretStore,
      onDismiss = { managingKeys = false },
      onAddKey = onAddKey,
      onDeleteKey = onDeleteKey,
    )
  }

  selectedProfile?.let { profile ->
    HostConnectionDialog(
      profile = profile,
      sshKeys = sshKeys,
      onSaveProfile = { updated ->
        onUpdateProfile(updated)
        selectedProfile = null
      },
      onDeleteProfile = {
        selectedProfile = null
        onDeleteProfile(profile)
      },
      onProbeCapabilities = onProbeCapabilities,
      onDismiss = { selectedProfile = null },
    )
  }
}

@Composable
private fun EmptyHostsPanel(onAddHost: () -> Unit) {
  Column(
    modifier =
      Modifier
        .fillMaxWidth()
        .background(ShellowColors.PanelBackground, RoundedCornerShape(8.dp))
        .padding(horizontal = 18.dp, vertical = 22.dp),
    verticalArrangement = Arrangement.spacedBy(8.dp),
  ) {
    Text(
      "No Profiles",
      color = ShellowColors.TerminalText,
      style = MaterialTheme.typography.bodyMedium,
      fontWeight = FontWeight.SemiBold,
    )
    Text(
      "Add a profile to open a host directly in Terminal or Codex.",
      color = ShellowColors.TerminalMuted,
      style = MaterialTheme.typography.bodySmall,
    )
    TextButton(onClick = onAddHost) {
      Text("Add Profile")
    }
  }
}

@Composable
private fun HostProfileRow(
  profile: HostProfile,
  onOpen: () -> Unit,
  onEdit: () -> Unit,
  onDuplicate: () -> Unit,
) {
  var actionsExpanded by remember(profile.id) { mutableStateOf(false) }

  Row(
    modifier =
      Modifier
        .fillMaxWidth()
        .padding(horizontal = 14.dp, vertical = 12.dp),
    verticalAlignment = Alignment.CenterVertically,
    horizontalArrangement = Arrangement.spacedBy(12.dp),
  ) {
    Row(
      modifier = Modifier.weight(1f).clickable(onClick = onOpen),
      verticalAlignment = Alignment.CenterVertically,
      horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
      Text(
        when (profile.launchKind) {
          ProfileLaunchKind.Terminal -> "T"
          ProfileLaunchKind.Codex -> "C"
          ProfileLaunchKind.Claude -> "A"
        },
        color = ShellowColors.Accent,
        fontWeight = FontWeight.Bold,
        modifier =
          Modifier
            .background(ShellowColors.Accent.copy(alpha = 0.12f), RoundedCornerShape(8.dp))
            .padding(horizontal = 9.dp, vertical = 7.dp),
      )
      Column(
        modifier = Modifier.weight(1f),
        verticalArrangement = Arrangement.spacedBy(2.dp),
      ) {
        Text(profile.name, color = ShellowColors.TerminalText, style = MaterialTheme.typography.titleSmall)
        Text("${profile.launchKind.title} · ${profile.endpoint}", color = ShellowColors.TerminalMuted)
        profile.persistentTerminal?.takeIf { profile.launchKind == ProfileLaunchKind.Terminal }?.let { configuration ->
          Text(
            "${configuration.backend.compactTitle} · ${configuration.name}",
            color = ShellowColors.Accent,
            style = MaterialTheme.typography.labelSmall,
          )
        }
      }
      Text("Open", color = ShellowColors.Accent, style = MaterialTheme.typography.labelMedium)
    }
    Box {
      TextButton(
        onClick = { actionsExpanded = true },
        modifier = Modifier.semantics { contentDescription = "Actions for ${profile.name}" },
      ) {
        Text("...", color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.labelMedium)
      }
      DropdownMenu(
        expanded = actionsExpanded,
        onDismissRequest = { actionsExpanded = false },
      ) {
        DropdownMenuItem(
          text = { Text("Edit") },
          onClick = {
            actionsExpanded = false
            onEdit()
          },
        )
        DropdownMenuItem(
          text = { Text("Duplicate") },
          onClick = {
            actionsExpanded = false
            onDuplicate()
          },
        )
      }
    }
  }
}

@Composable
private fun HostConnectionDialog(
  profile: HostProfile,
  sshKeys: List<SSHKeyCredential>,
  onSaveProfile: (HostProfile) -> Unit,
  onDeleteProfile: () -> Unit,
  onProbeCapabilities: suspend (HostProfile) -> RemoteHostProbeOutcome,
  onDismiss: () -> Unit,
) {
  var selectedTab by remember(profile.id) { mutableStateOf(ProfileEditorTab.Connection) }
  var name by remember(profile.id) { mutableStateOf(profile.name) }
  var host by remember(profile.id) { mutableStateOf(profile.host) }
  var port by remember(profile.id) { mutableStateOf(profile.port.toString()) }
  var username by remember(profile.id) { mutableStateOf(profile.username) }
  var launchKind by remember(profile.id) { mutableStateOf(profile.launchKind) }
  var authentication by remember(profile.id) { mutableStateOf(profile.authentication) }
  var preferredKeyId by remember(profile.id) { mutableStateOf(profile.preferredKeyId) }
  var persistentEnabled by remember(profile.id) { mutableStateOf(profile.persistentTerminal != null) }
  var persistentBackend by
    remember(profile.id) { mutableStateOf(profile.persistentTerminal?.backend ?: PersistentTerminalBackend.Tmux) }
  var persistentName by
    remember(profile.id) {
      mutableStateOf(
        profile.persistentTerminal?.name
          ?: PersistentTerminalConfiguration.suggestedName(profile.name, profile.host),
      )
    }
  val parsedPort = port.toIntOrNull()?.takeIf { it in 1..65535 }
  val normalizedHost = host.trim()
  val normalizedUser = username.trim().ifEmpty { "root" }
  val generatedName = "$normalizedUser@$normalizedHost:${parsedPort ?: 22}"
  val normalizedName = name.trim().ifEmpty { generatedName }
  var detectedReport by remember(profile.id) { mutableStateOf(profile.capabilityReport) }
  var probeInProgress by remember(profile.id) { mutableStateOf(false) }
  var probeError by remember(profile.id) { mutableStateOf<String?>(null) }
  var confirmingDelete by remember(profile.id) { mutableStateOf(false) }
  val scope = rememberCoroutineScope()
  val validatedPersistentName = PersistentTerminalConfiguration.validatedName(persistentName)
  val endpointChanged = normalizedHost != profile.host || parsedPort != profile.port || normalizedUser != profile.username
  val workingProfile =
    profile.copy(
      name = normalizedName,
      host = normalizedHost,
      port = parsedPort ?: 22,
      username = normalizedUser,
      launchKind = launchKind,
      authentication = authentication,
      preferredKeyId = if (authentication == AuthenticationKind.PrivateKey) preferredKeyId else null,
      trustedHostKeySha256 = if (endpointChanged) null else profile.trustedHostKeySha256,
      persistentTerminal =
        if (launchKind == ProfileLaunchKind.Terminal && persistentEnabled && validatedPersistentName != null) {
          PersistentTerminalConfiguration(validatedPersistentName, persistentBackend)
        } else {
          null
        },
      capabilityReport = if (endpointChanged) null else profile.capabilityReport,
    )
  val persistentConfigurationValid = !persistentEnabled || validatedPersistentName != null
  val serverRequirement =
    when {
      normalizedHost.isEmpty() -> "Enter a hostname or IP address."
      parsedPort == null -> "Port must be a number from 1 to 65535."
      else -> null
    }
  val configurationValid =
    serverRequirement == null &&
      (launchKind != ProfileLaunchKind.Terminal || persistentConfigurationValid)

  AlertDialog(
    onDismissRequest = onDismiss,
    containerColor = ShellowColors.PanelBackground,
    titleContentColor = ShellowColors.TerminalText,
    textContentColor = ShellowColors.TerminalText,
    title = { Text("Edit Profile") },
    text = {
      Column(
        modifier = Modifier.heightIn(max = 620.dp).verticalScroll(rememberScrollState()),
        verticalArrangement = Arrangement.spacedBy(14.dp),
      ) {
        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
          ProfileEditorTab.entries.forEach { tab ->
            AuthenticationChoice(
              title = tab.title,
              selected = selectedTab == tab,
              modifier = Modifier.weight(1f),
            ) { selectedTab = tab }
          }
        }

        if (selectedTab == ProfileEditorTab.Connection) {
          Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text("Open with", color = ShellowColors.TerminalText, fontWeight = FontWeight.SemiBold)
            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
              ProfileLaunchKind.entries.forEach { kind ->
                AuthenticationChoice(
                  title = if (kind == ProfileLaunchKind.Claude) "Claude" else kind.title,
                  selected = launchKind == kind,
                  modifier = Modifier.weight(1f),
                ) { launchKind = kind }
              }
            }
            Text(
              when (launchKind) {
                ProfileLaunchKind.Terminal -> "Open a remote shell and persistent workspaces."
                ProfileLaunchKind.Codex -> "Open remote Codex sessions."
                ProfileLaunchKind.Claude -> "Open durable Claude Code sessions over SSH."
              },
              color = ShellowColors.TerminalMuted,
              style = MaterialTheme.typography.labelSmall,
            )
          }

          Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text("Authentication", color = ShellowColors.TerminalText, fontWeight = FontWeight.SemiBold)
            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
              AuthenticationKind.entries.forEach { kind ->
                AuthenticationChoice(
                  title = kind.title,
                  selected = authentication == kind,
                  modifier = Modifier.weight(1f),
                ) { authentication = kind }
              }
            }
            Text(
              when (authentication) {
                AuthenticationKind.Automatic -> "Tries every saved key, then uses a saved password or asks for one."
                AuthenticationKind.Password -> "Uses password authentication only."
                AuthenticationKind.PrivateKey ->
                  if (preferredKeyId == null) "Tries every saved key and never falls back to a password."
                  else "Uses only the selected key and never falls back to a password."
              },
              color = ShellowColors.TerminalMuted,
              style = MaterialTheme.typography.labelSmall,
            )

            if (authentication == AuthenticationKind.PrivateKey) {
              AuthenticationChoice(
                title = "All saved keys",
                selected = preferredKeyId == null,
                modifier = Modifier.fillMaxWidth(),
              ) { preferredKeyId = null }
              sshKeys.forEach { key ->
                AuthenticationChoice(
                  title = key.name,
                  selected = preferredKeyId == key.id,
                  modifier = Modifier.fillMaxWidth(),
                ) { preferredKeyId = key.id }
              }
              if (sshKeys.isEmpty()) {
                Text(
                  "No SSH keys are saved yet. Add one from the SSH Keys menu before connecting.",
                  color = ShellowColors.Warning,
                  style = MaterialTheme.typography.labelSmall,
                )
              }
            }
          }

          if (launchKind == ProfileLaunchKind.Terminal) {
            Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
              Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                Text("Persistent terminal", color = ShellowColors.TerminalText, fontWeight = FontWeight.SemiBold)
                Text("Restore the same remote workspace after reconnecting", color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.labelSmall)
              }
              Checkbox(checked = persistentEnabled, onCheckedChange = { persistentEnabled = it })
            }
            if (persistentEnabled) {
              Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                PersistentTerminalBackend.entries.forEach { backend ->
                  AuthenticationChoice(
                    title = backend.compactTitle,
                    selected = persistentBackend == backend,
                    modifier = Modifier.weight(1f),
                  ) { persistentBackend = backend }
                }
              }
              OutlinedTextField(
                value = persistentName,
                onValueChange = { persistentName = it },
                modifier = Modifier.fillMaxWidth(),
                label = { Text("Session name") },
                isError = validatedPersistentName == null,
                singleLine = true,
              )
              if (validatedPersistentName == null) {
                Text("Use 1–48 ASCII letters, numbers, - or _.", color = ShellowColors.Warning, style = MaterialTheme.typography.labelSmall)
              } else {
                val capability = profile.capabilityReport?.capability(persistentBackend)
                if (capability != null && capability.supportLevel != RemoteComponentSupportLevel.Supported) {
                  Text(
                    "This host was last detected without full ${persistentBackend.displayTitle} support; you can still save and try it.",
                    color = ShellowColors.Warning,
                    style = MaterialTheme.typography.labelSmall,
                  )
                }
              }
            }
          }
        } else {
          OutlinedTextField(
            value = name,
            onValueChange = { name = it },
            modifier = Modifier.fillMaxWidth(),
            label = { Text("Name") },
            singleLine = true,
          )
          OutlinedTextField(
            value = host,
            onValueChange = { host = it },
            modifier = Modifier.fillMaxWidth(),
            label = { Text("Host") },
            singleLine = true,
          )
          OutlinedTextField(
            value = port,
            onValueChange = { port = it },
            modifier = Modifier.fillMaxWidth(),
            label = { Text("Port") },
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
            singleLine = true,
          )
          OutlinedTextField(
            value = username,
            onValueChange = { username = it },
            modifier = Modifier.fillMaxWidth(),
            label = { Text("User") },
            singleLine = true,
          )
          Text(
            serverRequirement ?: "Leave Name blank to use $generatedName. An empty User is saved as root.",
            color = if (serverRequirement == null) ShellowColors.TerminalMuted else ShellowColors.Warning,
            style = MaterialTheme.typography.labelSmall,
          )
        }
        RemoteCapabilityCard(
          report = detectedReport,
          inProgress = probeInProgress,
          errorMessage = probeError,
          onRefresh = {
            scope.launch {
              probeInProgress = true
              probeError = null
              val outcome = onProbeCapabilities(workingProfile)
              detectedReport = outcome.report
              probeError = outcome.errorMessage
              probeInProgress = false
            }
          },
        )
        TextButton(onClick = { confirmingDelete = true }) {
          Text("Delete profile", color = MaterialTheme.colorScheme.error)
        }
      }
    },
    confirmButton = {
      TextButton(enabled = configurationValid, onClick = { onSaveProfile(workingProfile) }) { Text("Save") }
    },
    dismissButton = { TextButton(onClick = onDismiss) { Text("Cancel") } },
  )
  if (confirmingDelete) {
    AlertDialog(
      onDismissRequest = { confirmingDelete = false },
      title = { Text("Delete this profile?") },
      text = { Text("The saved profile and its profile-scoped credentials will be removed from this device.") },
      confirmButton = {
        TextButton(onClick = onDeleteProfile) {
          Text("Delete", color = MaterialTheme.colorScheme.error)
        }
      },
      dismissButton = {
        TextButton(onClick = { confirmingDelete = false }) { Text("Cancel") }
      },
    )
  }
}

private enum class ProfileEditorTab(val title: String) {
  Connection("Connection"),
  Server("Server"),
}

@Composable
private fun ConnectionStatusRow(
  title: String,
  detail: String,
  healthy: Boolean,
) {
  Row(horizontalArrangement = Arrangement.spacedBy(10.dp), verticalAlignment = Alignment.Top) {
    Box(
      modifier =
        Modifier
          .padding(top = 5.dp)
          .size(8.dp)
          .background(if (healthy) ShellowColors.Success else ShellowColors.Warning, RoundedCornerShape(4.dp)),
    )
    Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
      Text(title, color = ShellowColors.TerminalText, style = MaterialTheme.typography.bodySmall, fontWeight = FontWeight.SemiBold)
      Text(detail, color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.labelSmall)
    }
  }
}

@Composable
private fun ConnectionModeOption(
  title: String,
  subtitle: String,
  detail: String,
  enabled: Boolean = true,
  onClick: () -> Unit,
) {
  Row(
    modifier =
      Modifier
        .fillMaxWidth()
        .background(ShellowColors.KeyBackground.copy(alpha = 0.38f), RoundedCornerShape(12.dp))
        .clickable(enabled = enabled, onClick = onClick)
        .padding(horizontal = 14.dp, vertical = 13.dp),
    verticalAlignment = Alignment.CenterVertically,
  ) {
    Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
      Text(title, color = if (enabled) ShellowColors.TerminalText else ShellowColors.TerminalMuted, style = MaterialTheme.typography.bodyMedium, fontWeight = FontWeight.SemiBold)
      Text(subtitle, color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.labelSmall)
      Text(detail, color = ShellowColors.TerminalMuted.copy(alpha = 0.72f), style = MaterialTheme.typography.labelSmall)
    }
    Text("Open", color = if (enabled) ShellowColors.Accent else ShellowColors.TerminalMuted, style = MaterialTheme.typography.labelMedium)
  }
}

@Composable
private fun RemoteCapabilityCard(
  report: xyz.zinglix.shellow.core.RemoteHostCapabilityReport?,
  inProgress: Boolean,
  errorMessage: String?,
  onRefresh: suspend () -> Unit,
) {
  val scope = rememberCoroutineScope()
  Column(
    modifier =
      Modifier
        .fillMaxWidth()
        .background(ShellowColors.KeyBackground.copy(alpha = 0.38f), RoundedCornerShape(8.dp))
        .padding(12.dp),
    verticalArrangement = Arrangement.spacedBy(8.dp),
  ) {
    Row(verticalAlignment = Alignment.CenterVertically) {
      Text("Target capabilities", modifier = Modifier.weight(1f), color = ShellowColors.TerminalText, fontWeight = FontWeight.SemiBold)
      if (inProgress) {
        CircularProgressIndicator(modifier = Modifier.size(18.dp), strokeWidth = 2.dp)
      } else {
        TextButton(onClick = { scope.launch { onRefresh() } }) { Text("Refresh") }
      }
    }
    if (report != null) {
      Text(
        "${report.system.displayTitle} · ${report.system.architecture} · ${report.system.shellName}",
        color = ShellowColors.TerminalMuted,
        style = MaterialTheme.typography.labelSmall,
      )
      report.components.forEach { component ->
        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
          Text(component.backend.compactTitle, modifier = Modifier.weight(1f), color = ShellowColors.TerminalText, style = MaterialTheme.typography.bodySmall)
          Text(
            listOf(component.supportLevel.title, component.version).filter { it.isNotBlank() }.joinToString(" · "),
            color =
              if (component.supportLevel == RemoteComponentSupportLevel.Supported) ShellowColors.Success
              else ShellowColors.Warning,
            style = MaterialTheme.typography.labelSmall,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
          )
        }
      }
      Text("Kernel ${report.system.kernelRelease}", color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.labelSmall)
    } else if (inProgress) {
      Text("Detecting the target system and terminal components…", color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.labelSmall)
    } else {
      Text(errorMessage ?: "No capability report yet.", color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.labelSmall)
    }
  }
}

@Composable
private fun AddHostDialog(
  onDismiss: () -> Unit,
  onAdd: (HostProfile) -> Unit,
) {
  var name by remember { mutableStateOf("") }
  var host by remember { mutableStateOf("") }
  var port by remember { mutableStateOf("22") }
  var username by remember { mutableStateOf("root") }
  val parsedPort = port.toIntOrNull()
  val normalizedHost = host.trim()
  val normalizedUser = username.trim().ifEmpty { "root" }
  val generatedName = "$normalizedUser@$normalizedHost:${parsedPort ?: 22}"
  val addHostRequirement =
    when {
      normalizedHost.isEmpty() -> "Enter a hostname or IP address."
      parsedPort == null || parsedPort !in 1..65535 -> "Port must be a number from 1 to 65535."
      else -> null
    }
  val canAdd = addHostRequirement == null

  AlertDialog(
    onDismissRequest = onDismiss,
    containerColor = ShellowColors.PanelBackground,
    titleContentColor = ShellowColors.TerminalText,
    textContentColor = ShellowColors.TerminalText,
    title = { Text("Add Host") },
    text = {
      Column(
        modifier =
          Modifier
            .fillMaxWidth()
            .heightIn(max = 420.dp)
            .verticalScroll(rememberScrollState()),
        verticalArrangement = Arrangement.spacedBy(10.dp),
      ) {
        OutlinedTextField(
          value = name,
          onValueChange = { name = it },
          modifier = Modifier.fillMaxWidth(),
          label = { Text("Name") },
          singleLine = true,
        )
        OutlinedTextField(
          value = host,
          onValueChange = { host = it },
          modifier = Modifier.fillMaxWidth(),
          label = { Text("Host") },
          singleLine = true,
        )
        OutlinedTextField(
          value = port,
          onValueChange = { port = it },
          modifier = Modifier.fillMaxWidth(),
          label = { Text("Port") },
          singleLine = true,
          keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
        )
        OutlinedTextField(
          value = username,
          onValueChange = { username = it },
          modifier = Modifier.fillMaxWidth(),
          label = { Text("User") },
          singleLine = true,
        )
        addHostRequirement?.let {
          Text(it, color = ShellowColors.Warning, style = MaterialTheme.typography.labelSmall)
        } ?: Text(
          if (name.isBlank() && normalizedHost.isNotEmpty()) "Generated name: $generatedName"
          else "Only Host is required. New profiles use Terminal and Auto authentication.",
          color = ShellowColors.TerminalMuted,
          style = MaterialTheme.typography.labelSmall,
        )
      }
    },
    confirmButton = {
      TextButton(
        enabled = canAdd,
        onClick = {
          onAdd(
            HostProfile(
              name = name.trim().ifEmpty { generatedName },
              host = normalizedHost,
              port = parsedPort ?: 22,
              username = normalizedUser,
              authentication = AuthenticationKind.Automatic,
              launchKind = ProfileLaunchKind.Terminal,
              trustedHostKeySha256 = null,
            ),
          )
        },
      ) { Text("Add") }
    },
    dismissButton = { TextButton(onClick = onDismiss) { Text("Cancel") } },
  )
}

@Composable
private fun AuthenticationChoice(
  title: String,
  selected: Boolean,
  modifier: Modifier = Modifier,
  onClick: () -> Unit,
) {
  TextButton(
    onClick = onClick,
    modifier =
      modifier.background(
        if (selected) ShellowColors.Accent.copy(alpha = 0.14f) else ShellowColors.KeyBackground.copy(alpha = 0.38f),
        RoundedCornerShape(8.dp),
      ),
  ) {
    Text(title, color = if (selected) ShellowColors.Accent else ShellowColors.TerminalMuted)
  }
}

@Composable
private fun SSHKeyManagementDialog(
  credentials: List<SSHKeyCredential>,
  secretStore: SSHSecretStore,
  onDismiss: () -> Unit,
  onAddKey: (SSHKeyCredential) -> Unit,
  onDeleteKey: (SSHKeyCredential) -> Unit,
) {
  var addingKey by remember { mutableStateOf(false) }
  var name by remember { mutableStateOf("") }
  var privateKeyPem by remember { mutableStateOf("") }
  var passphrase by remember { mutableStateOf("") }
  var status by remember { mutableStateOf<String?>(null) }
  val clipboard = LocalClipboardManager.current

  val beginAddingKey = {
    name = ""
    privateKeyPem = ""
    passphrase = ""
    status = null
    addingKey = true
  }
  val keyRequirement =
    when {
      name.isBlank() -> "Enter a name for this key."
      !privateKeyLooksUsable(privateKeyPem) -> "Paste a valid OpenSSH private key."
      else -> null
    }
  val canAdd = keyRequirement == null

  AlertDialog(
    onDismissRequest = {
      if (addingKey) {
        addingKey = false
      } else {
        onDismiss()
      }
    },
    containerColor = ShellowColors.PanelBackground,
    titleContentColor = ShellowColors.TerminalText,
    textContentColor = ShellowColors.TerminalText,
    title = { Text(if (addingKey) "New Key" else "SSH Keys") },
    text = {
      Column(
        modifier =
          Modifier
            .fillMaxWidth()
            .heightIn(max = 420.dp)
            .verticalScroll(rememberScrollState()),
        verticalArrangement = Arrangement.spacedBy(10.dp),
      ) {
        if (addingKey) {
          OutlinedTextField(
            value = name,
            onValueChange = { name = it },
            modifier = Modifier.fillMaxWidth(),
            label = { Text("Name") },
            singleLine = true,
          )
          Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
            Text("OpenSSH private key", modifier = Modifier.weight(1f), color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.labelMedium)
            TextButton(
              onClick = {
                clipboard.getText()?.text?.let { privateKeyPem = it }
              },
            ) { Text("Paste") }
          }
          OutlinedTextField(
            value = privateKeyPem,
            onValueChange = { privateKeyPem = it },
            modifier = Modifier.fillMaxWidth(),
            minLines = 7,
            textStyle = MaterialTheme.typography.bodySmall.copy(fontFamily = FontFamily.Monospace),
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Ascii),
          )
          Text("Paste an OpenSSH private key.", color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.labelSmall)
          OutlinedTextField(
            value = passphrase,
            onValueChange = { passphrase = it },
            modifier = Modifier.fillMaxWidth(),
            label = { Text("Optional passphrase") },
            singleLine = true,
            visualTransformation = PasswordVisualTransformation(),
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password),
          )
          keyRequirement?.let {
            Text(it, color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.labelSmall)
          }
          status?.let { Text(it, color = ShellowColors.TerminalMuted) }
        } else if (credentials.isEmpty()) {
          Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
            Text("No SSH Keys", color = ShellowColors.TerminalText, style = MaterialTheme.typography.bodyMedium, fontWeight = FontWeight.SemiBold)
            Text(
              "Add a private key for key-based authentication.",
              color = ShellowColors.TerminalMuted,
              style = MaterialTheme.typography.bodySmall,
            )
            TextButton(onClick = beginAddingKey) { Text("Add Key") }
          }
        } else {
          credentials.forEach { credential ->
            Row(
              modifier = Modifier.fillMaxWidth(),
              verticalAlignment = Alignment.CenterVertically,
              horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
              Column(Modifier.weight(1f)) {
                Text(credential.name, color = ShellowColors.TerminalText, style = MaterialTheme.typography.bodyMedium)
                Text(credential.id, color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.labelSmall, maxLines = 1)
              }
              TextButton(onClick = { onDeleteKey(credential) }) { Text("Delete") }
            }
          }
        }
      }
    },
    confirmButton = {
      Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        if (addingKey) {
          TextButton(
            enabled = canAdd,
            onClick = {
              val credential = SSHKeyCredential(name = name.trim())
              runCatching {
                secretStore.saveKeySecret(privateKeyPem, credential.id, SSHSecretKind.PrivateKey)
                if (passphrase.isNotBlank()) {
                  secretStore.saveKeySecret(passphrase, credential.id, SSHSecretKind.Passphrase)
                }
              }.onSuccess {
                addingKey = false
                onAddKey(credential)
              }.onFailure {
                secretStore.deleteKeySecret(credential.id, SSHSecretKind.PrivateKey)
                secretStore.deleteKeySecret(credential.id, SSHSecretKind.Passphrase)
                status = "Key could not be saved"
              }
            },
          ) { Text("Add") }
        } else {
          if (credentials.isNotEmpty()) {
            TextButton(onClick = beginAddingKey) { Text("Add") }
          }
          TextButton(onClick = onDismiss) { Text("Done") }
        }
      }
    },
    dismissButton = {
      if (addingKey) {
        TextButton(onClick = { addingKey = false }) { Text("Cancel") }
      }
    },
  )
}

@Composable
private fun PasswordPromptDialog(
  request: PasswordPromptRequest,
  secretStore: SSHSecretStore,
  onDismiss: () -> Unit,
  onConnect: (String) -> Unit,
) {
  var password by remember(request.profile.id, request.mode) { mutableStateOf("") }
  var rememberPassword by remember(request.profile.id, request.mode) { mutableStateOf(false) }
  var status by remember(request.profile.id, request.mode) { mutableStateOf<String?>(null) }

  AlertDialog(
    onDismissRequest = onDismiss,
    containerColor = ShellowColors.PanelBackground,
    titleContentColor = ShellowColors.TerminalText,
    textContentColor = ShellowColors.TerminalText,
    title = { Text(request.mode.passwordTitle) },
    text = {
      Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
        HostConnectionSummary(profile = request.profile, reason = request.reason)
        OutlinedTextField(
          value = password,
          onValueChange = { password = it },
          modifier = Modifier.fillMaxWidth(),
          label = { Text("Password") },
          singleLine = true,
          visualTransformation = PasswordVisualTransformation(),
          keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password),
        )
        Row(verticalAlignment = Alignment.CenterVertically) {
          Checkbox(checked = rememberPassword, onCheckedChange = { rememberPassword = it })
          Text("Save in Keystore", color = ShellowColors.TerminalText, modifier = Modifier.weight(1f))
        }
        status?.let { Text(it, color = ShellowColors.TerminalMuted) }
        if (password.isBlank()) {
          Text("Enter a password to connect.", color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.labelSmall)
        }
      }
    },
    confirmButton = {
      TextButton(
        enabled = password.isNotBlank(),
        onClick = {
          if (rememberPassword) {
            val saved =
              runCatching {
                secretStore.saveSecret(password, request.profile, SSHSecretKind.Password)
              }.isSuccess
            if (!saved) {
              status = "Password could not be saved"
              return@TextButton
            }
          }
          onConnect(password)
        },
      ) { Text("Connect") }
    },
    dismissButton = { TextButton(onClick = onDismiss) { Text("Cancel") } },
  )
}

@Composable
private fun HostConnectionSummary(
  profile: HostProfile,
  reason: String? = null,
) {
  Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
    Text(
      profile.endpoint,
      color = ShellowColors.TerminalText,
      style = MaterialTheme.typography.bodyMedium,
      fontWeight = FontWeight.SemiBold,
    )
    Text(profile.hostKeyTrustTitle, color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.labelSmall)
    reason?.let {
      Text(it, color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.labelSmall)
    }
  }
}

@Composable
private fun SettingsScreen(
  report: IntegrationReport,
  displaySettings: AppDisplaySettings,
  onBack: () -> Unit,
  onDisplaySettingsChange: (AppDisplaySettings) -> Unit,
) {
  BackHandler(onBack = onBack)

  Column(
    Modifier
      .fillMaxSize()
      .background(ShellowColors.TerminalBackground)
      .verticalScroll(rememberScrollState())
      .padding(16.dp),
    verticalArrangement = Arrangement.spacedBy(12.dp),
  ) {
    Row(
      modifier = Modifier.fillMaxWidth(),
      verticalAlignment = Alignment.CenterVertically,
      horizontalArrangement = Arrangement.spacedBy(8.dp),
    ) {
      NavigationBackButton(onClick = onBack)
      Text("Settings", color = ShellowColors.TerminalText, style = MaterialTheme.typography.titleLarge)
    }
    SettingsSectionLabel("Display")
    SettingsGroup {
      TerminalThemeSelector(
        value = displaySettings.terminalTheme,
        onValueChange = { onDisplaySettingsChange(displaySettings.copy(terminalTheme = it)) },
      )
      PanelDivider()
      ThemeSelector(
        value = displaySettings.colorScheme,
        onValueChange = { onDisplaySettingsChange(displaySettings.copy(colorScheme = it)) },
      )
      PanelDivider()
      DisplaySlider(
        title = "Font Size",
        valueLabel = "${displaySettings.fontSizeSp.roundToInt()} sp",
        value = displaySettings.fontSizeSp,
        valueRange = DisplayFontSizeMin..DisplayFontSizeMax,
        onValueChange = { onDisplaySettingsChange(displaySettings.copy(fontSizeSp = it.roundToInt().toFloat())) },
      )
      PanelDivider()
      DisplaySlider(
        title = "Line Height",
        valueLabel = "${(displaySettings.lineHeightScale * 100).roundToInt()}%",
        value = displaySettings.lineHeightScale,
        valueRange = 0.9f..1.35f,
        onValueChange = { onDisplaySettingsChange(displaySettings.copy(lineHeightScale = (it * 20).roundToInt() / 20f)) },
      )
    }
    SettingsSectionLabel("Runtime")
    SettingsGroup {
      SettingsRow("VT", report.terminalBackend)
      PanelDivider()
      SettingsRow("SSH", report.sshBackend)
      PanelDivider()
      SettingsRow("GPU", report.rendererBackend)
    }
    SettingsSectionLabel("Input")
    SettingsGroup {
      SettingsToggleRow(
        title = "Keyboard Toolbar",
        checked = displaySettings.showKeyboardToolbar,
        onCheckedChange = { onDisplaySettingsChange(displaySettings.copy(showKeyboardToolbar = it)) },
      )
      PanelDivider()
      SettingsToggleRow(
        title = "Confirm Risky Paste",
        checked = displaySettings.confirmPaste,
        onCheckedChange = { onDisplaySettingsChange(displaySettings.copy(confirmPaste = it)) },
      )
    }
    SettingsSectionLabel("Transport")
    SettingsGroup {
      DisplaySlider(
        title = "Keep Alive",
        valueLabel = "${displaySettings.keepAliveSeconds.roundToInt()} s",
        value = displaySettings.keepAliveSeconds,
        valueRange = 10f..120f,
        onValueChange = {
          onDisplaySettingsChange(displaySettings.copy(keepAliveSeconds = (it / 5).roundToInt() * 5f))
        },
      )
      PanelDivider()
      SettingsToggleRow(
        title = "Detect Remote Ports",
        subtitle = "Opens a second SSH channel and checks listening TCP ports every two seconds.",
        checked = displaySettings.detectRemotePorts,
        onCheckedChange = { onDisplaySettingsChange(displaySettings.copy(detectRemotePorts = it)) },
      )
    }
    SettingsSectionLabel("Build")
    SettingsGroup {
      SettingsRow("Version", "${BuildConfig.VERSION_NAME} (${BuildConfig.VERSION_CODE})")
      PanelDivider()
      SettingsRow("Commit", BuildConfig.GIT_COMMIT)
    }
    TextButton(onClick = { onDisplaySettingsChange(AppDisplaySettings()) }) {
      Text("Restore defaults", color = MaterialTheme.colorScheme.error)
    }
  }
}

@Composable
private fun SettingsSectionLabel(title: String) {
  Text(
    title,
    color = ShellowColors.TerminalMuted,
    style = MaterialTheme.typography.labelSmall,
    modifier = Modifier.padding(horizontal = 4.dp),
  )
}

@Composable
private fun SettingsGroup(content: @Composable () -> Unit) {
  Column(
    modifier =
      Modifier
        .fillMaxWidth()
        .background(ShellowColors.PanelBackground, RoundedCornerShape(8.dp)),
  ) {
    content()
  }
}

@Composable
private fun PanelDivider() {
  Spacer(
    Modifier
      .fillMaxWidth()
      .height(1.dp)
      .background(ShellowColors.KeyBackground.copy(alpha = 0.5f)),
  )
}

@Composable
private fun ThemeSelector(
  value: ShellowColorScheme,
  onValueChange: (ShellowColorScheme) -> Unit,
) {
  var expanded by remember { mutableStateOf(false) }
  Box {
    SettingsValueRow(
      title = "App Appearance",
      value = value.title,
      onClick = { expanded = true },
    )
    DropdownMenu(
      expanded = expanded,
      onDismissRequest = { expanded = false },
    ) {
      ShellowColorScheme.entries.forEach { scheme ->
        DropdownMenuItem(
          text = {
            Text(
              scheme.title,
              color = if (value == scheme) ShellowColors.Accent else ShellowColors.TerminalText,
            )
          },
          onClick = {
            expanded = false
            onValueChange(scheme)
          },
        )
      }
    }
  }
}

@Composable
private fun TerminalThemeSelector(
  value: TerminalThemeSelection,
  onValueChange: (TerminalThemeSelection) -> Unit,
) {
  var expanded by remember { mutableStateOf(false) }
  Box {
    SettingsValueRow(
      title = "Terminal Theme",
      value = value.title,
      onClick = { expanded = true },
    )
    DropdownMenu(
      expanded = expanded,
      onDismissRequest = { expanded = false },
    ) {
      TerminalThemeSelection.entries.forEach { theme ->
        DropdownMenuItem(
          text = {
            Text(
              theme.title,
              color = if (value == theme) ShellowColors.Accent else ShellowColors.TerminalText,
            )
          },
          onClick = {
            expanded = false
            onValueChange(theme)
          },
        )
      }
    }
  }
}

@Composable
private fun DisplaySlider(
  title: String,
  valueLabel: String,
  value: Float,
  valueRange: ClosedFloatingPointRange<Float>,
  onValueChange: (Float) -> Unit,
) {
  Column(
    modifier =
      Modifier
        .fillMaxWidth()
        .padding(horizontal = 14.dp, vertical = 12.dp),
    verticalArrangement = Arrangement.spacedBy(8.dp),
  ) {
    Row(verticalAlignment = Alignment.CenterVertically) {
      Text(title, modifier = Modifier.weight(1f), color = ShellowColors.TerminalText)
      Text(valueLabel, color = ShellowColors.TerminalMuted)
    }
    Slider(
      value = value,
      onValueChange = onValueChange,
      valueRange = valueRange,
      colors =
        SliderDefaults.colors(
          thumbColor = ShellowColors.Accent,
          activeTrackColor = ShellowColors.Accent,
          inactiveTrackColor = ShellowColors.KeyBackground,
        ),
    )
  }
}

@Composable
private fun SettingsRow(label: String, value: String) {
  SettingsValueRow(title = label, value = value)
}

@Composable
private fun SettingsToggleRow(
  title: String,
  checked: Boolean,
  onCheckedChange: (Boolean) -> Unit,
  subtitle: String? = null,
) {
  Row(
    modifier =
      Modifier
        .fillMaxWidth()
        .clickable { onCheckedChange(!checked) }
        .padding(horizontal = 14.dp, vertical = 8.dp),
    verticalAlignment = Alignment.CenterVertically,
  ) {
    Column(Modifier.weight(1f)) {
      Text(title, color = ShellowColors.TerminalText)
      subtitle?.let {
        Text(it, color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.bodySmall)
      }
    }
    Checkbox(checked = checked, onCheckedChange = onCheckedChange)
  }
}

@Composable
private fun SettingsValueRow(
  title: String,
  value: String,
  onClick: (() -> Unit)? = null,
) {
  var rowModifier = Modifier.fillMaxWidth()
  if (onClick != null) {
    rowModifier = rowModifier.clickable { onClick() }
  }
  Row(
    modifier = rowModifier.padding(horizontal = 14.dp, vertical = 12.dp),
    verticalAlignment = Alignment.CenterVertically,
  ) {
    Text(title, modifier = Modifier.weight(1f), color = ShellowColors.TerminalText)
    Text(value, color = ShellowColors.TerminalMuted, maxLines = 1, overflow = TextOverflow.Ellipsis)
  }
}

private fun statusColor(state: ConnectionState) =
  when (state) {
    ConnectionState.Connected -> ShellowColors.Success
    ConnectionState.Connecting -> ShellowColors.Warning
    ConnectionState.Disconnected -> ShellowColors.TerminalMuted
  }

private const val DisplaySettingsPrefs = "shellow.displaySettings"
private const val DisplayFontSizeKey = "fontSizeSp.v1"
private const val DisplayFontSizeMin = 9f
private const val DisplayFontSizeMax = 22f
private const val DisplayLineHeightKey = "lineHeightScale.v1"
private const val DisplayColorSchemeKey = "colorScheme.v1"
private const val DisplayTerminalThemeKey = "terminalTheme.v1"
private const val DisplayConfirmPasteKey = "confirmPaste.v1"
private const val DisplayKeyboardToolbarKey = "keyboardToolbar.v1"
private const val DisplayKeepAliveKey = "keepAliveSeconds.v1"
private const val DisplayDetectRemotePortsKey = "detectRemotePorts.v1"

private fun loadDisplaySettings(context: Context): AppDisplaySettings {
  val preferences = context.getSharedPreferences(DisplaySettingsPrefs, Context.MODE_PRIVATE)
  return AppDisplaySettings(
    fontSizeSp = preferences.getFloat(DisplayFontSizeKey, 14f).coerceIn(DisplayFontSizeMin, DisplayFontSizeMax),
    lineHeightScale = preferences.getFloat(DisplayLineHeightKey, 1f).coerceIn(0.9f, 1.35f),
    colorScheme = ShellowColorScheme.fromWire(preferences.getString(DisplayColorSchemeKey, ShellowColorScheme.System.wire)),
    terminalTheme = TerminalThemeSelection.fromWire(preferences.getString(DisplayTerminalThemeKey, TerminalThemeSelection.ShellowDark.wire)),
    confirmPaste = preferences.getBoolean(DisplayConfirmPasteKey, true),
    showKeyboardToolbar = preferences.getBoolean(DisplayKeyboardToolbarKey, true),
    keepAliveSeconds = preferences.getFloat(DisplayKeepAliveKey, 30f).coerceIn(10f, 120f),
    detectRemotePorts = preferences.getBoolean(DisplayDetectRemotePortsKey, false),
  )
}

private fun saveDisplaySettings(
  context: Context,
  settings: AppDisplaySettings,
) {
  context.getSharedPreferences(DisplaySettingsPrefs, Context.MODE_PRIVATE).edit {
    putFloat(DisplayFontSizeKey, settings.fontSizeSp.coerceIn(DisplayFontSizeMin, DisplayFontSizeMax))
    putFloat(DisplayLineHeightKey, settings.lineHeightScale.coerceIn(0.9f, 1.35f))
    putString(DisplayColorSchemeKey, settings.colorScheme.wire)
    putString(DisplayTerminalThemeKey, settings.terminalTheme.wire)
    putBoolean(DisplayConfirmPasteKey, settings.confirmPaste)
    putBoolean(DisplayKeyboardToolbarKey, settings.showKeyboardToolbar)
    putFloat(DisplayKeepAliveKey, settings.keepAliveSeconds.coerceIn(10f, 120f))
    putBoolean(DisplayDetectRemotePortsKey, settings.detectRemotePorts)
  }
}

private const val HostProfilesPrefs = "shellow.hostProfiles"
private const val HostProfilesKey = "profiles.v1"
private const val SSHKeysPrefs = "shellow.sshKeys"
private const val SSHKeysKey = "keys.v1"

private fun defaultHostProfiles(): List<HostProfile> = emptyList()

private fun loadHostProfiles(context: Context): List<HostProfile> {
  val stored =
    context
      .getSharedPreferences(HostProfilesPrefs, Context.MODE_PRIVATE)
      .getString(HostProfilesKey, null)
      ?: return defaultHostProfiles()

  return runCatching {
    val values = JSONArray(stored)
    List(values.length()) { index -> HostProfile.fromJson(values.getJSONObject(index)) }
      .filter { it.name.isNotBlank() && it.host.isNotBlank() && it.username.isNotBlank() }
  }.getOrElse {
    defaultHostProfiles()
  }
}

private fun saveHostProfiles(
  context: Context,
  profiles: List<HostProfile>,
) {
  val json = JSONArray()
  profiles.forEach { profile -> json.put(profile.toJson()) }
  context.getSharedPreferences(HostProfilesPrefs, Context.MODE_PRIVATE).edit {
    putString(HostProfilesKey, json.toString())
  }
}

private fun loadSSHKeyCredentials(context: Context): List<SSHKeyCredential> {
  val stored =
    context
      .getSharedPreferences(SSHKeysPrefs, Context.MODE_PRIVATE)
      .getString(SSHKeysKey, null)
      ?: return emptyList()

  return runCatching {
    val values = JSONArray(stored)
    List(values.length()) { index ->
      val value = values.getJSONObject(index)
      SSHKeyCredential(
        id = value.optString("id").takeIf { it.isNotBlank() } ?: UUID.randomUUID().toString(),
        name = value.optString("name").takeIf { it.isNotBlank() } ?: "SSH Key",
      )
    }.filter { it.name.isNotBlank() }
  }.getOrElse {
    emptyList()
  }
}

private fun saveSSHKeyCredentials(
  context: Context,
  credentials: List<SSHKeyCredential>,
) {
  val json = JSONArray()
  credentials.forEach { credential ->
    json.put(
      JSONObject()
        .put("id", credential.id)
        .put("name", credential.name),
    )
  }
  context.getSharedPreferences(SSHKeysPrefs, Context.MODE_PRIVATE).edit {
    putString(SSHKeysKey, json.toString())
  }
}

private fun translateTerminalInput(value: String, ctrlArmed: Boolean): String =
  if (!ctrlArmed) {
    value
  } else {
    value
      .mapNotNull { char ->
        val lower = char.lowercaseChar()
        if (lower in 'a'..'z') {
          (lower.code - 'a'.code + 1).toChar()
        } else {
          null
        }
      }
      .joinToString("")
  }

private fun String.metaEncodedTerminalInput(): String =
  buildString {
    this@metaEncodedTerminalInput.forEach { char ->
      append('\u001B')
      append(char)
    }
  }

private fun String.isRiskyTerminalPaste(): Boolean =
  length > 120 || any { it == '\n' || it == '\r' }

private fun String.terminalPasteLineCount(): Int =
  if (isEmpty()) 0 else count { it == '\n' } + 1

private fun String.hostKeyFingerprintOrNull(): String? {
  val markerIndex = indexOf(HostKeyConfirmationPrefix)
  if (markerIndex < 0) return null
  return substring(markerIndex + HostKeyConfirmationPrefix.length)
    .trim()
    .substringBefore(' ')
    .substringBefore('\n')
    .takeIf { it.startsWith("SHA256:") && it.length > "SHA256:".length }
}

private fun saveTerminalTranscript(
  context: Context,
  session: TerminalSession,
): File {
  val documentsDir = context.getExternalFilesDir(Environment.DIRECTORY_DOCUMENTS) ?: context.filesDir
  val transcriptDir = File(documentsDir, "Shellow-Transcripts").apply { mkdirs() }
  val file = File(transcriptDir, transcriptFileName(session.host))
  file.writeText(session.copyableText(), Charsets.UTF_8)
  return file
}

private fun transcriptFileName(host: String): String {
  val timestamp = SimpleDateFormat("yyyyMMdd-HHmmss", Locale.US).format(Date())
  return "shellow-${host.safeTranscriptFileComponent()}-$timestamp.txt"
}

private fun String.safeTranscriptFileComponent(): String {
  val value =
    map { char ->
      if (char.isLetterOrDigit() || char == '-' || char == '_') char else '-'
    }
      .joinToString("")
      .split('-')
      .filter { it.isNotBlank() }
      .joinToString("-")
  return value.ifBlank { "terminal" }
}

private fun String.firstTerminalUrl(): String? {
  val match =
    Regex("""https?://[^\s<>()\[\]{}"'`]+""")
      .find(this)
      ?.value
      ?: return null
  return match.trimEnd('.', ',', ';', ':', '!', '?')
}

private fun privateKeyLooksUsable(value: String): Boolean =
  value.contains("BEGIN") && value.contains("PRIVATE KEY")

private fun TerminalSession.searchPresentation(
  query: String,
  focusedIndex: Int,
): TerminalSearchPresentation {
  val normalized = query.trim()
  if (normalized.isEmpty()) {
    return TerminalSearchPresentation("", emptyList(), null)
  }

  val hits =
    if (grid != null && (grid.hasVisibleContent || grid.activeScreen == TerminalScreenKind.Alternate)) {
      grid.lines.flatMapIndexed { index, line ->
        line.terminalSearchCellRanges(normalized).map { range ->
          TerminalSearchHit.Grid(index, range)
        }
      }
    } else {
      rows.mapIndexedNotNull { index, row ->
        if (row.searchableText().contains(normalized, ignoreCase = true)) TerminalSearchHit.History(index) else null
      }
    }

  val active = hits.getOrNull(focusedIndex.floorMod(hits.size))
  return TerminalSearchPresentation(normalized, hits, active)
}

private fun TerminalRow.searchableText(): String {
  val prefix = if (prompt.isEmpty()) "" else "$prompt "
  return "$prefix$text"
}

private fun Int.floorMod(divisor: Int): Int =
  if (divisor <= 0) 0 else ((this % divisor) + divisor) % divisor

private fun TerminalSession.copyableText(): String =
  if (grid != null && (grid.hasVisibleContent || grid.activeScreen == TerminalScreenKind.Alternate)) {
    grid.lines.joinToString("\n")
  } else {
    rows.joinToString("\n") { row ->
      val prompt = if (row.prompt.isEmpty()) "" else "${row.prompt} "
      "$prompt${row.text}"
    }
  }

private fun TerminalSession.terminalDescriptor(): String {
  val size = "${terminalCols}x${terminalRows}"
  val currentGrid = grid
  if (currentGrid == null) {
    return "$host  $size"
  }
  val parts = mutableListOf(host, size)
  if (currentGrid.activeScreen == TerminalScreenKind.Primary && currentGrid.scrollbackLen > 0) {
    parts += "sb ${currentGrid.scrollbackLen}"
  }
  parts += "dirty ${currentGrid.dirtyRows.size}"
  return parts.joinToString("  ")
}

private fun TerminalSession.promptInputText(): String {
  if (isAlternateScreenActive()) return ""
  val row = rows.lastOrNull() ?: return ""
  return if (row.style == TerminalRowStyle.Prompt) row.text else ""
}

private fun TerminalGridSnapshot.viewportFirstRow(firstVisibleItemIndex: Int): Int {
  if (activeScreen != TerminalScreenKind.Primary || lines.size <= rows) return 0
  val visibleRows = rows.toInt().coerceAtLeast(1)
  val maxFirstRow = (lines.size - visibleRows).coerceAtLeast(0)
  return firstVisibleItemIndex.coerceIn(0, maxFirstRow)
}

private fun TerminalSelection?.containsGrid(row: Int): Boolean =
  this is TerminalSelection.Grid && row in rowRange(anchor.row, focus.row)

private fun TerminalSelection?.containsHistory(row: Int): Boolean =
  this is TerminalSelection.History && row in rowRange(anchor, focus)

private fun TerminalSelection?.extendingGridRow(row: Int): TerminalSelection =
  if (this is TerminalSelection.Grid) {
    TerminalSelection.Grid(
      TerminalSelectionPoint(anchor.row, 0),
      TerminalSelectionPoint(row, TerminalSelectionPoint.LineEndColumn),
    )
  } else {
    TerminalSelection.gridRow(row)
  }

private fun TerminalSelection?.extendingHistory(row: Int): TerminalSelection =
  if (this is TerminalSelection.History) {
    TerminalSelection.History(anchor, row)
  } else {
    TerminalSelection.History(row, row)
  }

private fun rowRange(anchor: Int, focus: Int): IntRange =
  if (anchor <= focus) anchor..focus else focus..anchor

private fun TerminalSelection.Companion.gridRow(row: Int): TerminalSelection =
  TerminalSelection.Grid(
    TerminalSelectionPoint(row, 0),
    TerminalSelectionPoint(row, TerminalSelectionPoint.LineEndColumn),
  )

private fun TerminalSelection?.gridCellRange(
  row: Int,
  line: String,
): TerminalCellRange? {
  if (this !is TerminalSelection.Grid) return null
  val lineEnd = line.terminalCellWidth().coerceAtLeast(1)
  val (start, end) = orderedSelectionPoints(anchor, focus)
  if (row !in start.row..end.row) return null

  val lower = if (row == start.row) start.column.coerceAtMost(lineEnd) else 0
  val upper = if (row == end.row) end.column.coerceAtMost(lineEnd) else lineEnd
  if (lower == upper && row == start.row && row == end.row) return null
  return TerminalCellRange(lower.coerceAtMost(upper), lower.coerceAtLeast(upper))
}

private fun TerminalSelection?.isFullGridRow(
  row: Int,
  line: String,
): Boolean {
  val range = gridCellRange(row, line) ?: return false
  val lineEnd = line.terminalCellWidth().coerceAtLeast(1)
  return range.start <= 0 && range.endExclusive >= lineEnd
}

private fun orderedSelectionPoints(
  first: TerminalSelectionPoint,
  second: TerminalSelectionPoint,
): Pair<TerminalSelectionPoint, TerminalSelectionPoint> =
  if (first.row < second.row || (first.row == second.row && first.column <= second.column)) {
    first to second
  } else {
    second to first
  }

private fun TerminalSession.selectedText(selection: TerminalSelection?): String? =
  when (selection) {
    is TerminalSelection.Grid -> {
      val text =
        rowRange(selection.anchor.row, selection.focus.row)
          .mapNotNull { row ->
            val line = grid?.lines?.getOrNull(row) ?: return@mapNotNull null
            val range = selection.gridCellRange(row, line) ?: return@mapNotNull null
            line.terminalSubstring(range).trim()
          }
          .joinToString("\n")
          .trim()
      text.takeIf { it.isNotEmpty() }
    }
    is TerminalSelection.History -> {
      val text =
        rowRange(selection.anchor, selection.focus)
          .mapNotNull { rowIndex ->
            rows.getOrNull(rowIndex)?.let { row ->
              val prompt = if (row.prompt.isEmpty()) "" else "${row.prompt} "
              "$prompt${row.text}".trim()
            }
          }
          .joinToString("\n")
          .trim()
      text.takeIf { it.isNotEmpty() }
    }
    null -> null
  }

private fun TerminalSession.isAlternateScreenActive(): Boolean =
  grid?.activeScreen == TerminalScreenKind.Alternate

private fun TerminalSession.isBracketedPasteActive(): Boolean =
  grid?.bracketedPaste == true

private fun TerminalSession.isApplicationCursorKeysActive(): Boolean =
  grid?.applicationCursorKeys == true

private fun TerminalGridSnapshot.mousePressSequence(row: Int, column: Int): String? {
  return mouseEventSequence(row, column, TerminalMouseEvent.Press)
}

private enum class TerminalMouseEvent(
  val buttonCode: Int,
  val terminator: Char,
) {
  Press(0, 'M'),
  Drag(32, 'M'),
  Release(0, 'm'),
}

private fun TerminalGridSnapshot.mouseEventSequence(
  row: Int,
  column: Int,
  event: TerminalMouseEvent,
): String? {
  if (!mouseReporting || !sgrMouse) return null
  if (event == TerminalMouseEvent.Drag && !mouseDragReporting) return null

  val terminalRow =
    if (activeScreen == TerminalScreenKind.Primary) {
      row - scrollbackLen + 1
    } else {
      row + 1
    }

  if (terminalRow !in 1..rows) return null
  val terminalColumn = (column + 1).coerceIn(1, cols)
  return "\u001B[<${event.buttonCode};$terminalColumn;$terminalRow${event.terminator}"
}

private enum class TerminalArrowKey(
  private val csi: String,
  private val ss3: String,
) {
  Up("\u001B[A", "\u001BOA"),
  Down("\u001B[B", "\u001BOB"),
  Left("\u001B[D", "\u001BOD"),
  Right("\u001B[C", "\u001BOC");

  fun sequence(applicationCursorKeys: Boolean): String =
    if (applicationCursorKeys) ss3 else csi
}

private fun terminalInputForHardwareKey(
  event: KeyEvent,
  applicationCursorKeys: Boolean,
): String? {
  val metaPressed = event.isAltPressed || event.isMetaPressed
  terminalSpecialInput(event.key, applicationCursorKeys)?.let { special ->
    return if (metaPressed) "\u001B$special" else special
  }

  if (event.isCtrlPressed) {
    hardwareControlInput(event.key)?.let { return it }
  }

  val unicode = event.utf16CodePoint
  if (unicode <= 0) {
    return null
  }

  val text = String(Character.toChars(unicode))
  return if (metaPressed) "\u001B$text" else text
}

private fun terminalSpecialInput(
  key: Key,
  applicationCursorKeys: Boolean,
): String? =
  when (key) {
    Key.Enter -> "\r"
    Key.Tab -> "\t"
    Key.Escape -> "\u001B"
    Key.Backspace -> "\u007F"
    Key.Delete -> "\u001B[3~"
    Key.DirectionUp -> TerminalArrowKey.Up.sequence(applicationCursorKeys)
    Key.DirectionDown -> TerminalArrowKey.Down.sequence(applicationCursorKeys)
    Key.DirectionLeft -> TerminalArrowKey.Left.sequence(applicationCursorKeys)
    Key.DirectionRight -> TerminalArrowKey.Right.sequence(applicationCursorKeys)
    Key.MoveHome -> "\u001B[H"
    Key.MoveEnd -> "\u001B[F"
    Key.PageUp -> "\u001B[5~"
    Key.PageDown -> "\u001B[6~"
    Key.F1 -> TerminalFunctionKey.F1.sequence
    Key.F2 -> TerminalFunctionKey.F2.sequence
    Key.F3 -> TerminalFunctionKey.F3.sequence
    Key.F4 -> TerminalFunctionKey.F4.sequence
    Key.F5 -> TerminalFunctionKey.F5.sequence
    Key.F6 -> TerminalFunctionKey.F6.sequence
    Key.F7 -> TerminalFunctionKey.F7.sequence
    Key.F8 -> TerminalFunctionKey.F8.sequence
    Key.F9 -> TerminalFunctionKey.F9.sequence
    Key.F10 -> TerminalFunctionKey.F10.sequence
    Key.F11 -> TerminalFunctionKey.F11.sequence
    Key.F12 -> TerminalFunctionKey.F12.sequence
    else -> null
  }

private fun hardwareControlInput(key: Key): String? =
  when (key) {
    Key.A -> "\u0001"
    Key.B -> "\u0002"
    Key.C -> "\u0003"
    Key.D -> "\u0004"
    Key.E -> "\u0005"
    Key.F -> "\u0006"
    Key.G -> "\u0007"
    Key.H -> "\u0008"
    Key.I -> "\t"
    Key.J -> "\n"
    Key.K -> "\u000B"
    Key.L -> "\u000C"
    Key.M -> "\r"
    Key.N -> "\u000E"
    Key.O -> "\u000F"
    Key.P -> "\u0010"
    Key.Q -> "\u0011"
    Key.R -> "\u0012"
    Key.S -> "\u0013"
    Key.T -> "\u0014"
    Key.U -> "\u0015"
    Key.V -> "\u0016"
    Key.W -> "\u0017"
    Key.X -> "\u0018"
    Key.Y -> "\u0019"
    Key.Z -> "\u001A"
    Key.LeftBracket -> "\u001B"
    Key.Backslash -> "\u001C"
    Key.RightBracket -> "\u001D"
    Key.Six -> "\u001E"
    Key.Minus -> "\u001F"
    Key.Slash -> "\u007F"
    else -> null
  }

private enum class TerminalFunctionKey(val label: String, val sequence: String) {
  F1("F1", "\u001BOP"),
  F2("F2", "\u001BOQ"),
  F3("F3", "\u001BOR"),
  F4("F4", "\u001BOS"),
  F5("F5", "\u001B[15~"),
  F6("F6", "\u001B[17~"),
  F7("F7", "\u001B[18~"),
  F8("F8", "\u001B[19~"),
  F9("F9", "\u001B[20~"),
  F10("F10", "\u001B[21~"),
  F11("F11", "\u001B[23~"),
  F12("F12", "\u001B[24~"),
}

private data class TerminalGridCanvasRowPlan(
  val runs: List<TerminalGridCanvasRun>,
)

private data class TerminalGridCanvasRun(
  val text: String,
  val style: TerminalGridStyle,
  val consumedCells: Int,
)

private fun terminalGridCanvasRowPlan(
  grid: TerminalGridSnapshot,
  line: String,
  row: Int,
): TerminalGridCanvasRowPlan {
  val sourceRuns =
    grid.styledLines
      .getOrNull(row)
      ?.runs
      ?.takeIf { it.isNotEmpty() }
      ?: listOf(TerminalGridRun(line.ifEmpty { " " }, TerminalGridStyle.Plain))
  val cursorOffset =
    if (grid.cursorVisible && row == grid.cursorRow) {
      grid.cursorColumn.coerceIn(0, (grid.cols - 1).coerceAtLeast(0))
    } else {
      null
    }
  val cursorGlyph = grid.cursorShape.glyph()
  val planned = mutableListOf<TerminalGridCanvasRun>()
  var consumed = 0
  var cursorWritten = false

  sourceRuns.forEach { run ->
    val insertion =
      run.text.withTerminalCursor(
        targetColumn = cursorOffset.takeUnless { cursorWritten },
        consumedCells = consumed,
        glyph = cursorGlyph,
      )
    planned += TerminalGridCanvasRun(insertion.text, run.style, consumed)
    consumed = insertion.nextCellColumn
    cursorWritten = cursorWritten || insertion.didWrite
  }

  if (cursorOffset != null && !cursorWritten) {
    if (cursorOffset > consumed) {
      planned += TerminalGridCanvasRun(" ".repeat(cursorOffset - consumed), TerminalGridStyle.Plain, consumed)
      consumed = cursorOffset
    }
    planned += TerminalGridCanvasRun(cursorGlyph.toString(), TerminalGridStyle.Plain, consumed)
  }

  if (planned.isEmpty()) {
    planned += TerminalGridCanvasRun(" ", TerminalGridStyle.Plain, 0)
  }

  return TerminalGridCanvasRowPlan(planned)
}

private fun terminalGridCanvasRowSignature(
  grid: TerminalGridSnapshot,
  line: String,
  row: Int,
): String {
  val styleSignature =
    grid.styledLines
      .getOrNull(row)
      ?.runs
      ?.joinToString("|") { "${it.text}\u001E${it.style.canvasSignature()}" }
      .orEmpty()
  val cursor =
    if (grid.cursorVisible && row == grid.cursorRow) {
      "cursor:${grid.cursorColumn}:${grid.cursorShape}"
    } else {
      "-"
    }
  return listOf(grid.cols.toString(), line, styleSignature, cursor).joinToString("\u001F")
}

private fun android.graphics.Canvas.drawTerminalRun(
  text: String,
  style: TerminalGridStyle,
  consumedCells: Int,
  cellWidthPx: Float,
  xPaddingPx: Float,
  rowHeightPx: Float,
  baseline: Float,
  textPaint: Paint,
  backgroundPaint: Paint,
) {
  if (text.isEmpty()) return

  val x = xPaddingPx + consumedCells * cellWidthPx
  val runWidth = text.terminalCellWidth() * cellWidthPx
  style.resolvedBackground()?.let { background ->
    backgroundPaint.color = background.toArgb()
    drawRect(x, 0f, x + runWidth, rowHeightPx, backgroundPaint)
  }

  textPaint.color = style.resolvedForeground().toArgb()
  textPaint.isFakeBoldText = style.bold
  textPaint.isUnderlineText = style.underline
  textPaint.isStrikeThruText = style.strikethrough
  drawText(text, x, baseline, textPaint)
}

private fun terminalGridAnnotatedLine(
  grid: TerminalGridSnapshot,
  line: String,
  row: Int,
  selectedCells: TerminalCellRange?,
): AnnotatedString =
  buildAnnotatedString {
    val sourceRuns =
      grid.styledLines
        .getOrNull(row)
        ?.runs
        ?.takeIf { it.isNotEmpty() }
        ?: listOf(TerminalGridRun(line.ifEmpty { " " }, TerminalGridStyle.Plain))
    val cursorOffset =
      if (grid.cursorVisible && row == grid.cursorRow) {
        grid.cursorColumn.coerceIn(0, (grid.cols - 1).coerceAtLeast(0))
      } else {
        null
      }
    var consumed = 0
    var cursorWritten = false
    val cursorGlyph = grid.cursorShape.glyph()

    sourceRuns.forEach { run ->
      val insertion =
        run.text.withTerminalCursor(
          targetColumn = cursorOffset.takeUnless { cursorWritten },
          consumedCells = consumed,
          glyph = cursorGlyph,
        )
      appendStyledRunWithSelection(insertion.text, run.style, selectedCells, consumed)
      consumed = insertion.nextCellColumn
      cursorWritten = cursorWritten || insertion.didWrite
    }

    if (cursorOffset != null && !cursorWritten) {
      if (cursorOffset > consumed) {
        appendStyledRunWithSelection(" ".repeat(cursorOffset - consumed), TerminalGridStyle.Plain, selectedCells, consumed)
        consumed = cursorOffset
      }
      appendStyledRunWithSelection(cursorGlyph.toString(), TerminalGridStyle.Plain, selectedCells, consumed)
    }

    if (length == 0) {
      appendStyledRun(" ", TerminalGridStyle.Plain)
    }
  }

private fun TerminalCursorShape.glyph(): Char =
  when (this) {
    TerminalCursorShape.Block -> '\u2588'
    TerminalCursorShape.Underline -> '\u2581'
    TerminalCursorShape.Bar -> '\u258F'
  }

private data class TerminalCursorInsertion(
  val text: String,
  val nextCellColumn: Int,
  val didWrite: Boolean,
)

private fun String.withTerminalCursor(
  targetColumn: Int?,
  consumedCells: Int,
  glyph: Char,
): TerminalCursorInsertion {
  if (targetColumn == null) {
    return TerminalCursorInsertion(this, consumedCells + terminalCellWidth(), didWrite = false)
  }

  val output = StringBuilder()
  var index = 0
  var cell = consumedCells
  var didWrite = false

  while (index < length) {
    val start = index
    var next = index + Character.charCount(codePointAt(index))
    while (next < length && codePointAt(next).isZeroWidthTerminalCodePoint()) {
      next += Character.charCount(codePointAt(next))
    }

    val segment = substring(start, next)
    val width = segment.terminalCellWidth()
    if (!didWrite && width > 0 && targetColumn >= cell && targetColumn < cell + width) {
      output.append(glyph)
      repeat(width - 1) { output.append(' ') }
      didWrite = true
    } else {
      output.append(segment)
    }
    cell += width
    index = next
  }

  return TerminalCursorInsertion(output.toString(), cell, didWrite)
}

private inline fun String.forEachTerminalSegment(block: (String, Int) -> Unit) {
  var index = 0

  while (index < length) {
    val start = index
    var next = index + Character.charCount(codePointAt(index))
    while (next < length && codePointAt(next).isZeroWidthTerminalCodePoint()) {
      next += Character.charCount(codePointAt(next))
    }

    val segment = substring(start, next)
    block(segment, segment.terminalSegmentCellWidth())
    index = next
  }
}

private fun String.terminalCellWidth(): Int {
  var total = 0
  forEachTerminalSegment { _, width -> total += width }
  return total
}

private fun String.terminalSearchCellRanges(query: String): List<TerminalCellRange> {
  if (query.isEmpty()) return emptyList()

  val ranges = mutableListOf<TerminalCellRange>()
  var startIndex = 0
  while (startIndex < length) {
    val matchStart = indexOf(query, startIndex = startIndex, ignoreCase = true)
    if (matchStart < 0) break

    val matchEnd = (matchStart + query.length).coerceAtMost(length)
    terminalCellRangeForUtf16Range(matchStart, matchEnd)?.let(ranges::add)
    startIndex = (matchStart + query.length).coerceAtLeast(matchStart + 1)
  }

  return ranges
}

private fun String.terminalCellRangeForUtf16Range(
  rangeStart: Int,
  rangeEndExclusive: Int,
): TerminalCellRange? {
  var index = 0
  var cell = 0
  var firstCell: Int? = null
  var lastCell = 0

  while (index < length) {
    val segmentStart = index
    var next = index + Character.charCount(codePointAt(index))
    while (next < length && codePointAt(next).isZeroWidthTerminalCodePoint()) {
      next += Character.charCount(codePointAt(next))
    }

    val width = substring(segmentStart, next).terminalSegmentCellWidth()
    if (width > 0 && segmentStart < rangeEndExclusive && rangeStart < next) {
      if (firstCell == null) firstCell = cell
      lastCell = cell + width
    }
    cell += width
    index = next
  }

  val start = firstCell ?: return null
  return TerminalCellRange(start, lastCell.coerceAtLeast(start + 1))
}

private fun String.terminalSubstring(range: TerminalCellRange): String {
  val output = StringBuilder()
  var cell = 0

  forEachTerminalSegment { segment, width ->
    if (width > 0 && range.overlaps(cell, cell + width)) {
      output.append(segment)
    }
    cell += width
  }

  return output.toString()
}

private fun String.terminalSegmentCellWidth(): Int {
  var index = 0
  var sawWide = false
  var sawNonZero = false

  while (index < length) {
    val codePoint = codePointAt(index)
    if (!codePoint.isZeroWidthTerminalCodePoint()) {
      sawNonZero = true
    }
    if (codePoint.isWideTerminalCodePoint()) {
      sawWide = true
    }
    index += Character.charCount(codePoint)
  }

  return when {
    !sawNonZero -> 0
    sawWide -> 2
    else -> 1
  }
}

private fun Int.isZeroWidthTerminalCodePoint(): Boolean =
  this in 0x0300..0x036F ||
    this in 0x1AB0..0x1AFF ||
    this in 0x1DC0..0x1DFF ||
    this in 0x20D0..0x20FF ||
    this in 0xFE00..0xFE0F ||
    this in 0xE0100..0xE01EF ||
    this == 0x200D

private fun Int.isWideTerminalCodePoint(): Boolean =
  this in 0x1100..0x115F ||
    this in 0x2329..0x232A ||
    this in 0x2E80..0xA4CF ||
    this in 0xAC00..0xD7A3 ||
    this in 0xF900..0xFAFF ||
    this in 0xFE10..0xFE19 ||
    this in 0xFE30..0xFE6F ||
    this in 0xFF00..0xFF60 ||
    this in 0xFFE0..0xFFE6 ||
    this in 0x1F000..0x1FAFF ||
    this in 0x20000..0x3FFFD

private fun AnnotatedString.Builder.appendStyledRun(
  text: String,
  style: TerminalGridStyle,
  isSelected: Boolean = false,
) {
  if (text.isEmpty()) return
  pushStyle(
    SpanStyle(
      color = style.resolvedForeground(),
      background =
        if (isSelected) {
          ShellowColors.Accent.copy(alpha = 0.34f)
        } else {
          style.resolvedBackground() ?: ComposeColor.Transparent
        },
      fontWeight = if (style.bold) FontWeight.SemiBold else FontWeight.Normal,
      textDecoration =
        when {
          style.underline && style.strikethrough -> TextDecoration.combine(listOf(TextDecoration.Underline, TextDecoration.LineThrough))
          style.underline -> TextDecoration.Underline
          style.strikethrough -> TextDecoration.LineThrough
          else -> null
        },
    ),
  )
  append(text)
  pop()
}

private fun AnnotatedString.Builder.appendStyledRunWithSelection(
  text: String,
  style: TerminalGridStyle,
  selectedCells: TerminalCellRange?,
  consumedCells: Int,
) {
  if (text.isEmpty()) return
  if (selectedCells == null) {
    appendStyledRun(text, style)
    return
  }

  var cell = consumedCells
  text.forEachTerminalSegment { segment, width ->
    appendStyledRun(
      text = segment,
      style = style,
      isSelected = width > 0 && selectedCells.overlaps(cell, cell + width),
    )
    cell += width
  }
}

private fun TerminalGridStyle.resolvedForeground(): ComposeColor =
  if (inverse) {
    bg?.toComposeColor() ?: ShellowColors.TerminalBackground
  } else {
    fg?.toComposeColor() ?: ShellowColors.TerminalText
  }

private fun TerminalGridStyle.resolvedBackground(): ComposeColor? =
  if (inverse) {
    fg?.toComposeColor() ?: ShellowColors.TerminalText
  } else {
    bg?.toComposeColor()
  }

private fun TerminalGridStyle.canvasSignature(): String =
  listOf(
    if (bold) "b" else "-",
    if (faint) "f" else "-",
    if (italic) "i" else "-",
    if (underline) "u" else "-",
    if (blink) "blink" else "-",
    if (inverse) "inv" else "-",
    if (strikethrough) "s" else "-",
    fg?.canvasSignature() ?: "fg:-",
    bg?.canvasSignature() ?: "bg:-",
  ).joinToString(",")

private fun TerminalGridColor.canvasSignature(): String = "$r,$g,$b"

private fun TerminalGridColor.toComposeColor(): ComposeColor =
  ComposeColor(
    red = r / 255f,
    green = g / 255f,
    blue = b / 255f,
  )
