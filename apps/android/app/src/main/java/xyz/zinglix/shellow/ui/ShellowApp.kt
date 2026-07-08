package xyz.zinglix.shellow.ui

import android.graphics.Paint
import android.graphics.Typeface
import android.content.Context
import android.os.Environment
import android.util.Log
import android.view.Surface
import android.view.SurfaceHolder
import android.view.SurfaceView
import androidx.compose.ui.viewinterop.AndroidView
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectDragGesturesAfterLongPress
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.ime
import androidx.compose.foundation.layout.navigationBarsPadding
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
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Checkbox
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Slider
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
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
import androidx.compose.ui.graphics.drawscope.drawIntoCanvas
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
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.zIndex
import xyz.zinglix.shellow.core.AuthenticationKind
import xyz.zinglix.shellow.core.ConnectionState
import xyz.zinglix.shellow.core.HostProfile
import xyz.zinglix.shellow.core.IntegrationReport
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
import xyz.zinglix.shellow.core.TerminalSession
import xyz.zinglix.shellow.theme.ShellowColors
import java.io.File
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.json.JSONArray
import org.json.JSONObject
import kotlin.math.floor
import kotlin.math.roundToInt

private enum class AppScreen {
  Hosts,
  Terminal,
  Settings,
}

private const val TerminalDirectInputSentinel = "\u2060"
private const val RendererLogTag = "ShellowRenderer"

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
)

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

private fun HostProfile.matchesProfileIdentity(other: HostProfile): Boolean =
  name == other.name &&
    host == other.host &&
    port == other.port &&
    username == other.username &&
    authentication == other.authentication

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun ShellowApp() {
  val core = remember { ShellowCoreSession() }
  val context = LocalContext.current
  val secretStore = remember { SSHSecretStore(context) }
  val scope = rememberCoroutineScope()
  var displaySettings by remember { mutableStateOf(loadDisplaySettings(context)) }
  val profiles =
    remember {
      mutableStateListOf<HostProfile>().also { profiles ->
        profiles.addAll(loadHostProfiles(context))
      }
    }
  var screen by remember { mutableStateOf(AppScreen.Hosts) }
  var session by remember { mutableStateOf(core.snapshot()) }
  var reconnectTarget by remember { mutableStateOf<ReconnectTarget?>(null) }

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

  fun updateSession(next: TerminalSession) {
    session = next
    captureObservedHostKeyIfNeeded(next)
  }

  LaunchedEffect(core) {
    var lastLiveRevision = withContext(Dispatchers.IO) { core.liveShellEventRevision() }
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
    }
  }

  DisposableEffect(core) {
    onDispose { core.close() }
  }

  LaunchedEffect(displaySettings) {
    saveDisplaySettings(context, displaySettings)
  }

  fun connectPasswordShell(profile: HostProfile, password: String, startupCommand: String) {
    session = TerminalSession.connecting(profile)
    screen = AppScreen.Terminal
    scope.launch {
      updateSession(withContext(Dispatchers.IO) { core.startPasswordShell(profile, password) })
      val command = startupCommand.trim()
      if (command.isNotEmpty() && session.state != ConnectionState.Disconnected) {
        updateSession(withContext(Dispatchers.IO) { core.sendTerminalInput("$command\r") })
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
    screen = AppScreen.Terminal
    scope.launch {
      updateSession(withContext(Dispatchers.IO) { core.startPrivateKeyShell(profile, privateKeyPem, passphrase) })
      val command = startupCommand.trim()
      if (command.isNotEmpty() && session.state != ConnectionState.Disconnected) {
        updateSession(withContext(Dispatchers.IO) { core.sendTerminalInput("$command\r") })
      }
    }
  }

  fun reconnect() {
    when (val target = reconnectTarget) {
      is ReconnectTarget.Preview -> {
        updateSession(core.connectPreview(target.profile))
        screen = AppScreen.Terminal
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

  Box(
    modifier =
      Modifier
        .fillMaxSize()
        .background(ShellowColors.TerminalBackground)
        .statusBarsPadding()
        .navigationBarsPadding()
  ) {
    when (screen) {
      AppScreen.Terminal ->
        TerminalScreen(
          session = session,
          displaySettings = displaySettings,
          onBackToHosts = { screen = AppScreen.Hosts },
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
      AppScreen.Hosts ->
        HostsScreen(
          profiles = profiles,
          secretStore = secretStore,
          onOpenSettings = { screen = AppScreen.Settings },
          onAddProfile = { profile ->
            profiles.add(profile)
            saveHostProfiles(context, profiles)
          },
          onPreview = { profile ->
            reconnectTarget = ReconnectTarget.Preview(profile)
            updateSession(core.connectPreview(profile))
            screen = AppScreen.Terminal
          },
          onConnectPassword = { profile, password, startup ->
            reconnectTarget = ReconnectTarget.Password(profile, password, startup)
            connectPasswordShell(profile, password, startup)
          },
          onConnectPrivateKey = { profile, privateKeyPem, passphrase, startup ->
            reconnectTarget = ReconnectTarget.PrivateKey(profile, privateKeyPem, passphrase, startup)
            connectPrivateKeyShell(profile, privateKeyPem, passphrase, startup)
          },
        )
      AppScreen.Settings ->
        SettingsScreen(
          report = session.integration,
          displaySettings = displaySettings,
          onBack = { screen = AppScreen.Hosts },
          onDisplaySettingsChange = { displaySettings = it },
        )
    }
  }
}

@Composable
@OptIn(ExperimentalLayoutApi::class)
private fun TerminalScreen(
  session: TerminalSession,
  displaySettings: AppDisplaySettings,
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
  var reportedCols by remember { mutableStateOf(session.terminalCols) }
  var reportedRows by remember { mutableStateOf(session.terminalRows) }
  var selection by remember { mutableStateOf<TerminalSelection?>(null) }
  var pendingPaste by remember { mutableStateOf<String?>(null) }
  var pendingRemoteClipboard by remember { mutableStateOf<RemoteClipboardRequest?>(null) }
  var transcriptSaveResult by remember { mutableStateOf<TranscriptSaveResult?>(null) }
  var handledClipboardSequence by remember { mutableStateOf(0L) }
  var searchVisible by remember { mutableStateOf(false) }
  var searchQuery by remember { mutableStateOf("") }
  var searchIndex by remember { mutableStateOf(0) }
  var viewportWidthPx by remember { mutableStateOf(0) }
  var viewportHeightPx by remember { mutableStateOf(0) }
  var rendererSurfaceReady by remember { mutableStateOf(false) }
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
  val keyboardOffsetDp = with(density) { keyboardOffsetPx.toDp() }
  val terminalHeaderInsetDp = 76.dp
  val terminalSearchBarTopDp = 64.dp
  val terminalSearchInsetDp = 130.dp
  val terminalTopInsetDp = if (searchVisible) terminalSearchInsetDp else terminalHeaderInsetDp
  val terminalBottomInsetDp = 10.dp
  val terminalTopInsetPx = with(density) { terminalTopInsetDp.toPx() }
  val terminalBottomInsetPx = with(density) { terminalBottomInsetDp.toPx() }
  val terminalTextSizePx = with(density) { displaySettings.fontSizeSp.sp.toPx() }
  val gridCellWidthPx = (terminalTextSizePx * 0.62f).coerceAtLeast(with(density) { 7.dp.toPx() })
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
    onInput(value)
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

    if (value.isRiskyTerminalPaste()) {
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

  Column(
    Modifier
      .fillMaxSize()
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
          .clickable { focusTerminalInput() },
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
    Row(
      modifier =
        Modifier
          .fillMaxWidth()
          .background(ShellowColors.PanelBackground)
          .padding(horizontal = 12.dp, vertical = 8.dp),
      horizontalArrangement = Arrangement.spacedBy(8.dp),
      verticalAlignment = Alignment.CenterVertically,
    ) {
      Row(
        modifier =
          Modifier
            .weight(1f)
            .horizontalScroll(rememberScrollState()),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalAlignment = Alignment.CenterVertically,
      ) {
        TerminalToolbarButton("Clear", onClick = onClearTerminal)
        TerminalToolbarButton("Reset", onClick = onResetTerminal)
        TerminalToolbarButton(
          "Save",
          onClick = {
            transcriptSaveResult =
              runCatching { saveTerminalTranscript(context, session) }
                .fold(
                  onSuccess = { file -> TranscriptSaveResult("Transcript Saved", file.name) },
                  onFailure = { error -> TranscriptSaveResult("Save Failed", error.message ?: error.toString()) },
                )
          },
        )
        TerminalToolbarButton("Copy") { clipboard.setText(AnnotatedString(session.copyableText())) }
        TerminalToolbarButton("Find") { searchVisible = !searchVisible }
        if (canJumpToBottom) {
          TerminalToolbarButton("Bottom") {
            terminalScope.launch {
              terminalListState.animateScrollToItem(terminalItemCount - 1)
            }
          }
        }
        if (selectedText != null) {
          TerminalToolbarButton("Copy Sel") { clipboard.setText(AnnotatedString(selectedText)) }
          if (selectedLink != null) {
            TerminalToolbarButton("Copy Link") { clipboard.setText(AnnotatedString(selectedLink)) }
          }
          TerminalToolbarButton("Clear Sel") { selection = null }
        }
        TerminalToolbarButton("Paste") {
          clipboard.getText()?.text?.takeIf { it.isNotEmpty() }?.let {
            handlePaste(it)
          }
        }
      }
      TerminalToolbarButton("Enter", accent = true) { sendEnter() }
    }
    Row(
      modifier =
        Modifier
          .fillMaxWidth()
          .background(ShellowColors.PanelBackground)
          .horizontalScroll(rememberScrollState())
          .padding(horizontal = 12.dp, vertical = 6.dp),
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
    Spacer(Modifier.height(6.dp + keyboardOffsetDp).fillMaxWidth().background(ShellowColors.PanelBackground))
  }

  pendingPaste?.let { paste ->
    AlertDialog(
      onDismissRequest = { pendingPaste = null },
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

  pendingRemoteClipboard?.let { request ->
    AlertDialog(
      onDismissRequest = { pendingRemoteClipboard = null },
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
      title = { Text(result.title) },
      text = { Text(result.message) },
      confirmButton = {
        TextButton(onClick = { transcriptSaveResult = null }) { Text("OK") }
      },
    )
  }
}

@Composable
private fun TerminalToolbarButton(
  label: String,
  accent: Boolean = false,
  onClick: () -> Unit,
) {
  TerminalCompactButton(
    label = label,
    active = accent,
    width = terminalToolbarButtonWidth(label),
    onClick = onClick,
  )
}

@Composable
private fun TerminalCompactButton(
  label: String,
  active: Boolean = false,
  width: androidx.compose.ui.unit.Dp,
  onClick: () -> Unit,
) {
  Box(
    modifier =
      Modifier
        .width(width)
        .height(34.dp)
        .background(
          if (active) ShellowColors.Accent else ShellowColors.KeyBackground,
          RoundedCornerShape(8.dp),
        )
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
    TerminalCompactButton("Back", width = 48.dp, onClick = onBackToHosts)
    Box(
      modifier =
        Modifier
          .size(9.dp)
          .background(statusColor(session.state), RoundedCornerShape(9.dp))
    )
    Text(
      session.title,
      modifier = Modifier.weight(1f),
      color = ShellowColors.TerminalText,
      style = MaterialTheme.typography.titleSmall,
      maxLines = 1,
      overflow = TextOverflow.Ellipsis,
    )
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
      Log.i(RendererLogTag, "Shellow renderer Android terminal surface frame ${width}x$height")
    }
    onPresentationChange(presented)
    return presented
  }

  DisposableEffect(Unit) {
    onDispose {
      onPresentationChange(false)
      Log.i(RendererLogTag, "Shellow renderer Android surface detach ${onDetachSurface()}")
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
              Log.i(RendererLogTag, "Shellow renderer Android surface attach $attachResponse")
              renderIfReady(this@apply)
            }

            override fun surfaceDestroyed(holder: SurfaceHolder) {
              onPresentationChange(false)
              Log.i(RendererLogTag, "Shellow renderer Android surface detach ${onDetachSurface()}")
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

@Composable
private fun HostsScreen(
  profiles: List<HostProfile>,
  secretStore: SSHSecretStore,
  onOpenSettings: () -> Unit,
  onAddProfile: (HostProfile) -> Unit,
  onPreview: (HostProfile) -> Unit,
  onConnectPassword: (HostProfile, String, String) -> Unit,
  onConnectPrivateKey: (HostProfile, String, String, String) -> Unit,
) {
  var passwordProfile by remember { mutableStateOf<HostProfile?>(null) }
  var privateKeyProfile by remember { mutableStateOf<HostProfile?>(null) }
  var addingProfile by remember { mutableStateOf(false) }

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
          "Hosts",
          modifier = Modifier.weight(1f),
          color = ShellowColors.TerminalText,
          style = MaterialTheme.typography.titleLarge,
        )
        TextButton(onClick = onOpenSettings) { Text("Settings") }
        Button(onClick = { addingProfile = true }) {
          Text("Add")
        }
      }
    }

    items(profiles, key = { it.id }) { profile ->
      Card(
        onClick = {
          if (profile.authentication == AuthenticationKind.Password) {
            passwordProfile = profile
          } else {
            privateKeyProfile = profile
          }
        },
        colors = CardDefaults.cardColors(containerColor = ShellowColors.PanelBackground),
      ) {
        Row(Modifier.fillMaxWidth().padding(14.dp), verticalAlignment = Alignment.CenterVertically) {
          Column(Modifier.weight(1f)) {
            Text(profile.name, color = ShellowColors.TerminalText, style = MaterialTheme.typography.titleSmall)
            Text(profile.endpoint, color = ShellowColors.TerminalMuted)
            Text(profile.hostKeyTrustTitle, color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.labelSmall)
          }
          Text(profile.authentication.title, color = ShellowColors.TerminalMuted, style = MaterialTheme.typography.labelMedium)
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

  passwordProfile?.let { profile ->
    PasswordDialog(
      profile = profile,
      secretStore = secretStore,
      onDismiss = { passwordProfile = null },
      onConnect = { password, startup ->
        passwordProfile = null
        onConnectPassword(profile, password, startup)
      },
    )
  }

  privateKeyProfile?.let { profile ->
    PrivateKeyDialog(
      profile = profile,
      secretStore = secretStore,
      onPreview = {
        privateKeyProfile = null
        onPreview(profile)
      },
      onDismiss = { privateKeyProfile = null },
      onConnect = { privateKeyPem, passphrase, startup ->
        privateKeyProfile = null
        onConnectPrivateKey(profile, privateKeyPem, passphrase, startup)
      },
    )
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
  var username by remember { mutableStateOf("") }
  var trustedHostKey by remember { mutableStateOf("") }
  var auth by remember { mutableStateOf(AuthenticationKind.PrivateKey) }
  val parsedPort = port.toIntOrNull()
  val canAdd =
    name.isNotBlank() &&
      host.isNotBlank() &&
      username.isNotBlank() &&
      parsedPort != null &&
      parsedPort in 1..65535

  AlertDialog(
    onDismissRequest = onDismiss,
    title = { Text("Add Host") },
    text = {
      Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
        OutlinedTextField(value = name, onValueChange = { name = it }, label = { Text("Name") }, singleLine = true)
        OutlinedTextField(value = host, onValueChange = { host = it }, label = { Text("Host") }, singleLine = true)
        OutlinedTextField(
          value = port,
          onValueChange = { port = it },
          label = { Text("Port") },
          singleLine = true,
          keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
        )
        OutlinedTextField(value = username, onValueChange = { username = it }, label = { Text("User") }, singleLine = true)
        OutlinedTextField(
          value = trustedHostKey,
          onValueChange = { trustedHostKey = it },
          label = { Text("Host key SHA256") },
          singleLine = true,
          textStyle = MaterialTheme.typography.bodySmall.copy(fontFamily = FontFamily.Monospace),
        )
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
          FilterChip(
            selected = auth == AuthenticationKind.PrivateKey,
            onClick = { auth = AuthenticationKind.PrivateKey },
            label = { Text(AuthenticationKind.PrivateKey.title) },
          )
          FilterChip(
            selected = auth == AuthenticationKind.Password,
            onClick = { auth = AuthenticationKind.Password },
            label = { Text(AuthenticationKind.Password.title) },
          )
        }
      }
    },
    confirmButton = {
      TextButton(
        enabled = canAdd,
        onClick = {
          onAdd(
            HostProfile(
              name = name.trim(),
              host = host.trim(),
              port = parsedPort ?: 22,
              username = username.trim(),
              authentication = auth,
              trustedHostKeySha256 = trustedHostKey.trim().takeIf { it.isNotBlank() },
            ),
          )
        },
      ) { Text("Add") }
    },
    dismissButton = { TextButton(onClick = onDismiss) { Text("Cancel") } },
  )
}

@Composable
private fun PasswordDialog(
  profile: HostProfile,
  secretStore: SSHSecretStore,
  onDismiss: () -> Unit,
  onConnect: (String, String) -> Unit,
) {
  var password by remember { mutableStateOf("") }
  var startup by remember { mutableStateOf("") }
  var rememberPassword by remember { mutableStateOf(false) }
  var hasSavedPassword by remember { mutableStateOf(false) }
  var keychainStatus by remember { mutableStateOf<String?>(null) }

  LaunchedEffect(profile.id) {
    hasSavedPassword = secretStore.hasSecret(profile, SSHSecretKind.Password)
    rememberPassword = !hasSavedPassword
  }

  AlertDialog(
    onDismissRequest = onDismiss,
    title = { Text("Live SSH") },
    text = {
      Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
        Text(profile.endpoint, color = ShellowColors.TerminalMuted)
        Text(profile.hostKeyTrustTitle, color = ShellowColors.TerminalMuted)
        if (hasSavedPassword) {
          Text("Saved password available", color = ShellowColors.TerminalMuted)
        }
        OutlinedTextField(
          value = password,
          onValueChange = { password = it },
          label = { Text("Password") },
          singleLine = true,
          visualTransformation = PasswordVisualTransformation(),
          keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password),
        )
        Row(verticalAlignment = Alignment.CenterVertically) {
          Checkbox(checked = rememberPassword, onCheckedChange = { rememberPassword = it })
          Text("Save password in Android Keystore")
        }
        OutlinedTextField(
          value = startup,
          onValueChange = { startup = it },
          label = { Text("Startup command") },
          singleLine = true,
        )
        keychainStatus?.let { Text(it, color = ShellowColors.TerminalMuted) }
      }
    },
    confirmButton = {
      TextButton(
        onClick = {
          val resolvedPassword =
            if (password.isBlank()) {
              secretStore.loadSecret(profile, SSHSecretKind.Password)
            } else {
              password
            }
          if (resolvedPassword == null) {
            keychainStatus = "Saved password could not be loaded"
            hasSavedPassword = false
            return@TextButton
          }

          if (rememberPassword && password.isNotBlank()) {
            runCatching {
              secretStore.saveSecret(password, profile, SSHSecretKind.Password)
            }.onSuccess {
              hasSavedPassword = true
              keychainStatus = "Password saved"
            }.onFailure {
              keychainStatus = "Password save failed"
            }
          }

          onConnect(resolvedPassword, startup)
        },
        enabled = password.isNotBlank() || hasSavedPassword,
      ) { Text("Connect") }
    },
    dismissButton = { TextButton(onClick = onDismiss) { Text("Cancel") } },
  )
}

@Composable
private fun PrivateKeyDialog(
  profile: HostProfile,
  secretStore: SSHSecretStore,
  onPreview: () -> Unit,
  onDismiss: () -> Unit,
  onConnect: (String, String, String) -> Unit,
) {
  var privateKeyPem by remember { mutableStateOf("") }
  var passphrase by remember { mutableStateOf("") }
  var startup by remember { mutableStateOf("") }
  var rememberPrivateKey by remember { mutableStateOf(false) }
  var rememberPassphrase by remember { mutableStateOf(false) }
  var hasSavedPrivateKey by remember { mutableStateOf(false) }
  var hasSavedPassphrase by remember { mutableStateOf(false) }
  var keychainStatus by remember { mutableStateOf<String?>(null) }
  val canConnect = privateKeyLooksUsable(privateKeyPem) || hasSavedPrivateKey

  LaunchedEffect(profile.id) {
    hasSavedPrivateKey = secretStore.hasSecret(profile, SSHSecretKind.PrivateKey)
    hasSavedPassphrase = secretStore.hasSecret(profile, SSHSecretKind.Passphrase)
    rememberPrivateKey = !hasSavedPrivateKey
    rememberPassphrase = false
  }

  AlertDialog(
    onDismissRequest = onDismiss,
    title = { Text("Live SSH Key") },
    text = {
      Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
        Text(profile.endpoint, color = ShellowColors.TerminalMuted)
        Text(profile.hostKeyTrustTitle, color = ShellowColors.TerminalMuted)
        TextButton(onClick = onPreview) { Text("Preview Connection Metadata") }
        if (hasSavedPrivateKey) {
          Text("Saved private key available", color = ShellowColors.TerminalMuted)
        }
        OutlinedTextField(
          value = privateKeyPem,
          onValueChange = { privateKeyPem = it },
          label = { Text("OpenSSH private key") },
          minLines = 7,
          textStyle = MaterialTheme.typography.bodySmall.copy(fontFamily = FontFamily.Monospace),
          keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Ascii),
        )
        Row(verticalAlignment = Alignment.CenterVertically) {
          Checkbox(checked = rememberPrivateKey, onCheckedChange = { rememberPrivateKey = it })
          Text("Save private key in Android Keystore")
        }
        if (hasSavedPassphrase) {
          Text("Saved passphrase available", color = ShellowColors.TerminalMuted)
        }
        OutlinedTextField(
          value = passphrase,
          onValueChange = { passphrase = it },
          label = { Text("Passphrase") },
          singleLine = true,
          visualTransformation = PasswordVisualTransformation(),
          keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password),
        )
        Row(verticalAlignment = Alignment.CenterVertically) {
          Checkbox(checked = rememberPassphrase, onCheckedChange = { rememberPassphrase = it })
          Text("Save passphrase in Android Keystore")
        }
        OutlinedTextField(
          value = startup,
          onValueChange = { startup = it },
          label = { Text("Startup command") },
          singleLine = true,
        )
        keychainStatus?.let { Text(it, color = ShellowColors.TerminalMuted) }
      }
    },
    confirmButton = {
      TextButton(
        onClick = {
          val resolvedPrivateKey =
            if (privateKeyPem.isBlank()) {
              secretStore.loadSecret(profile, SSHSecretKind.PrivateKey)
            } else {
              privateKeyPem
            }
          if (resolvedPrivateKey == null) {
            keychainStatus = "Saved private key could not be loaded"
            hasSavedPrivateKey = false
            return@TextButton
          }
          if (!privateKeyLooksUsable(resolvedPrivateKey)) {
            keychainStatus = "Private key is not an OpenSSH key"
            return@TextButton
          }

          if (rememberPrivateKey && privateKeyLooksUsable(privateKeyPem)) {
            runCatching {
              secretStore.saveSecret(privateKeyPem, profile, SSHSecretKind.PrivateKey)
            }.onSuccess {
              hasSavedPrivateKey = true
              keychainStatus = "Private key saved"
            }.onFailure {
              keychainStatus = "Private key save failed"
            }
          }

          if (rememberPassphrase && passphrase.isNotBlank()) {
            runCatching {
              secretStore.saveSecret(passphrase, profile, SSHSecretKind.Passphrase)
            }.onSuccess {
              hasSavedPassphrase = true
            }.onFailure {
              keychainStatus = "Passphrase save failed"
            }
          }

          val resolvedPassphrase =
            if (passphrase.isBlank()) {
              secretStore.loadSecret(profile, SSHSecretKind.Passphrase).orEmpty()
            } else {
              passphrase
            }
          onConnect(resolvedPrivateKey, resolvedPassphrase, startup)
        },
        enabled = canConnect,
      ) { Text("Connect") }
    },
    dismissButton = { TextButton(onClick = onDismiss) { Text("Cancel") } },
  )
}

@Composable
private fun SettingsScreen(
  report: IntegrationReport,
  displaySettings: AppDisplaySettings,
  onBack: () -> Unit,
  onDisplaySettingsChange: (AppDisplaySettings) -> Unit,
) {
  Column(Modifier.fillMaxSize().padding(16.dp), verticalArrangement = Arrangement.spacedBy(14.dp)) {
    Row(
      modifier = Modifier.fillMaxWidth(),
      verticalAlignment = Alignment.CenterVertically,
      horizontalArrangement = Arrangement.spacedBy(8.dp),
    ) {
      TextButton(onClick = onBack) { Text("Back") }
      Text("Settings", color = ShellowColors.TerminalText, style = MaterialTheme.typography.titleLarge)
    }
    DisplaySlider(
      title = "Font Size",
      valueLabel = "${displaySettings.fontSizeSp.roundToInt()} sp",
      value = displaySettings.fontSizeSp,
      valueRange = 11f..22f,
      onValueChange = { onDisplaySettingsChange(displaySettings.copy(fontSizeSp = it.roundToInt().toFloat())) },
    )
    DisplaySlider(
      title = "Line Height",
      valueLabel = "${(displaySettings.lineHeightScale * 100).roundToInt()}%",
      value = displaySettings.lineHeightScale,
      valueRange = 0.9f..1.35f,
      onValueChange = { onDisplaySettingsChange(displaySettings.copy(lineHeightScale = (it * 20).roundToInt() / 20f)) },
    )
    SettingsRow("VT", report.terminalBackend)
    SettingsRow("SSH", report.sshBackend)
    SettingsRow("GPU", report.rendererBackend)
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
        .background(ShellowColors.PanelBackground, RoundedCornerShape(8.dp))
        .padding(14.dp),
    verticalArrangement = Arrangement.spacedBy(8.dp),
  ) {
    Row(verticalAlignment = Alignment.CenterVertically) {
      Text(title, modifier = Modifier.weight(1f), color = ShellowColors.TerminalText)
      Text(valueLabel, color = ShellowColors.TerminalMuted)
    }
    Slider(value = value, onValueChange = onValueChange, valueRange = valueRange)
  }
}

@Composable
private fun SettingsRow(label: String, value: String) {
  Row(
    modifier =
      Modifier
        .fillMaxWidth()
        .background(ShellowColors.PanelBackground, RoundedCornerShape(8.dp))
        .padding(14.dp),
    verticalAlignment = Alignment.CenterVertically,
  ) {
    Text(label, modifier = Modifier.width(56.dp), color = ShellowColors.TerminalMuted)
    Text(value, color = ShellowColors.TerminalText, maxLines = 1, overflow = TextOverflow.Ellipsis)
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
private const val DisplayLineHeightKey = "lineHeightScale.v1"

private fun loadDisplaySettings(context: Context): AppDisplaySettings {
  val preferences = context.getSharedPreferences(DisplaySettingsPrefs, Context.MODE_PRIVATE)
  return AppDisplaySettings(
    fontSizeSp = preferences.getFloat(DisplayFontSizeKey, 14f).coerceIn(11f, 22f),
    lineHeightScale = preferences.getFloat(DisplayLineHeightKey, 1f).coerceIn(0.9f, 1.35f),
  )
}

private fun saveDisplaySettings(
  context: Context,
  settings: AppDisplaySettings,
) {
  context
    .getSharedPreferences(DisplaySettingsPrefs, Context.MODE_PRIVATE)
    .edit()
    .putFloat(DisplayFontSizeKey, settings.fontSizeSp.coerceIn(11f, 22f))
    .putFloat(DisplayLineHeightKey, settings.lineHeightScale.coerceIn(0.9f, 1.35f))
    .apply()
}

private const val HostProfilesPrefs = "shellow.hostProfiles"
private const val HostProfilesKey = "profiles.v1"

private fun defaultHostProfiles(): List<HostProfile> =
  listOf(
    HostProfile(
      "Staging",
      "10.0.0.18",
      22,
      "deploy",
      AuthenticationKind.PrivateKey,
      trustedHostKeySha256 = "SHA256:sample-staging-host-key",
      id = "sample-staging",
    ),
    HostProfile("10.248.1.102", "10.248.1.102", 22, "zinglix", AuthenticationKind.Password, id = "lab-10-248-1-102"),
    HostProfile("Home Lab", "192.168.1.42", 22, "zinglix", AuthenticationKind.Password, id = "sample-home-lab"),
  )

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
      .ifEmpty { defaultHostProfiles() }
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
  context
    .getSharedPreferences(HostProfilesPrefs, Context.MODE_PRIVATE)
    .edit()
    .putString(HostProfilesKey, json.toString())
    .apply()
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
