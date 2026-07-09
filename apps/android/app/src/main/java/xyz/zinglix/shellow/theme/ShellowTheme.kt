package xyz.zinglix.shellow.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.graphics.Color

enum class ShellowColorScheme(val wire: String, val title: String) {
  System("system", "System"),
  Light("light", "Light"),
  Dark("dark", "Dark");

  companion object {
    fun fromWire(value: String?) = entries.firstOrNull { it.wire == value } ?: System
  }
}

data class ShellowPalette(
  val accent: Color,
  val success: Color,
  val warning: Color,
  val terminalText: Color,
  val terminalMuted: Color,
  val terminalBackground: Color,
  val panelBackground: Color,
  val inputBackground: Color,
  val keyBackground: Color,
  val userMessageBackground: Color,
  val assistantMessageBackground: Color,
  val statusMessageBackground: Color,
  val toolMessageBackground: Color,
  val codeBackground: Color,
  val codeHeaderBackground: Color,
  val inlineCodeBackground: Color,
  val tableBackground: Color,
  val tableHeaderBackground: Color,
  val approvalBackground: Color,
  val warningBackground: Color,
  val successBackground: Color,
)

private val DarkShellowPalette =
  ShellowPalette(
    accent = Color(0xFF1C9F70),
    success = Color(0xFF45D18C),
    warning = Color(0xFFEDAD38),
    terminalText = Color(0xFFE0E8DE),
    terminalMuted = Color(0xFF95A39B),
    terminalBackground = Color(0xFF0D0F0E),
    panelBackground = Color(0xFF151817),
    inputBackground = Color(0xFF202422),
    keyBackground = Color(0xFF272C29),
    userMessageBackground = Color(0xFF151817),
    assistantMessageBackground = Color(0xFF18231D),
    statusMessageBackground = Color(0xFF17191D),
    toolMessageBackground = Color(0xFF1E1B16),
    codeBackground = Color(0xFF111317),
    codeHeaderBackground = Color(0xFF1A1D22),
    inlineCodeBackground = Color(0xFF252A31),
    tableBackground = Color(0xFF14171B),
    tableHeaderBackground = Color(0xFF1D2228),
    approvalBackground = Color(0xFF2A2116),
    warningBackground = Color(0xFF2A2116),
    successBackground = Color(0xFF162217),
  )

private val LightShellowPalette =
  ShellowPalette(
    accent = Color(0xFF147A56),
    success = Color(0xFF1E8F59),
    warning = Color(0xFFA56608),
    terminalText = Color(0xFF18221D),
    terminalMuted = Color(0xFF66736C),
    terminalBackground = Color(0xFFF7F9F6),
    panelBackground = Color(0xFFFFFFFF),
    inputBackground = Color(0xFFEFF3EE),
    keyBackground = Color(0xFFE0E7DF),
    userMessageBackground = Color(0xFFEAF5EE),
    assistantMessageBackground = Color(0xFFFFFFFF),
    statusMessageBackground = Color(0xFFF0F3EF),
    toolMessageBackground = Color(0xFFFFF3E2),
    codeBackground = Color(0xFFF7F9F6),
    codeHeaderBackground = Color(0xFFE9EFE8),
    inlineCodeBackground = Color(0xFFE8EEE7),
    tableBackground = Color(0xFFFFFFFF),
    tableHeaderBackground = Color(0xFFE9EFE8),
    approvalBackground = Color(0xFFFFF3E2),
    warningBackground = Color(0xFFFFF3E2),
    successBackground = Color(0xFFE8F6ED),
  )

object ShellowColors {
  private var palette by mutableStateOf(DarkShellowPalette)

  internal fun use(palette: ShellowPalette) {
    this.palette = palette
  }

  val Accent: Color get() = palette.accent
  val Success: Color get() = palette.success
  val Warning: Color get() = palette.warning
  val TerminalText: Color get() = palette.terminalText
  val TerminalMuted: Color get() = palette.terminalMuted
  val TerminalBackground: Color get() = palette.terminalBackground
  val PanelBackground: Color get() = palette.panelBackground
  val InputBackground: Color get() = palette.inputBackground
  val KeyBackground: Color get() = palette.keyBackground
  val UserMessageBackground: Color get() = palette.userMessageBackground
  val AssistantMessageBackground: Color get() = palette.assistantMessageBackground
  val StatusMessageBackground: Color get() = palette.statusMessageBackground
  val ToolMessageBackground: Color get() = palette.toolMessageBackground
  val CodeBackground: Color get() = palette.codeBackground
  val CodeHeaderBackground: Color get() = palette.codeHeaderBackground
  val InlineCodeBackground: Color get() = palette.inlineCodeBackground
  val TableBackground: Color get() = palette.tableBackground
  val TableHeaderBackground: Color get() = palette.tableHeaderBackground
  val ApprovalBackground: Color get() = palette.approvalBackground
  val WarningBackground: Color get() = palette.warningBackground
  val SuccessBackground: Color get() = palette.successBackground
}

private val DarkShellowScheme =
  darkColorScheme(
    primary = DarkShellowPalette.accent,
    primaryContainer = DarkShellowPalette.keyBackground,
    onPrimaryContainer = DarkShellowPalette.terminalText,
    secondary = DarkShellowPalette.success,
    secondaryContainer = DarkShellowPalette.inputBackground,
    onSecondaryContainer = DarkShellowPalette.terminalText,
    tertiary = DarkShellowPalette.warning,
    tertiaryContainer = DarkShellowPalette.warningBackground,
    onTertiaryContainer = DarkShellowPalette.terminalText,
    background = DarkShellowPalette.terminalBackground,
    surface = DarkShellowPalette.panelBackground,
    surfaceVariant = DarkShellowPalette.keyBackground,
    onSurfaceVariant = DarkShellowPalette.terminalMuted,
    inverseSurface = DarkShellowPalette.terminalText,
    inverseOnSurface = DarkShellowPalette.terminalBackground,
    outline = DarkShellowPalette.terminalMuted.copy(alpha = 0.68f),
    outlineVariant = DarkShellowPalette.keyBackground,
    onPrimary = Color.White,
    onSecondary = Color.Black,
    onTertiary = Color.Black,
    onBackground = DarkShellowPalette.terminalText,
    onSurface = DarkShellowPalette.terminalText,
  )

private val LightShellowScheme =
  lightColorScheme(
    primary = LightShellowPalette.accent,
    primaryContainer = LightShellowPalette.keyBackground,
    onPrimaryContainer = LightShellowPalette.terminalText,
    secondary = LightShellowPalette.success,
    secondaryContainer = LightShellowPalette.inputBackground,
    onSecondaryContainer = LightShellowPalette.terminalText,
    tertiary = LightShellowPalette.warning,
    tertiaryContainer = LightShellowPalette.warningBackground,
    onTertiaryContainer = LightShellowPalette.terminalText,
    background = LightShellowPalette.terminalBackground,
    surface = LightShellowPalette.panelBackground,
    surfaceVariant = LightShellowPalette.keyBackground,
    onSurfaceVariant = LightShellowPalette.terminalMuted,
    inverseSurface = LightShellowPalette.terminalText,
    inverseOnSurface = LightShellowPalette.terminalBackground,
    outline = LightShellowPalette.terminalMuted.copy(alpha = 0.68f),
    outlineVariant = LightShellowPalette.keyBackground,
    onPrimary = Color.White,
    onSecondary = Color.White,
    onTertiary = Color.White,
    onBackground = LightShellowPalette.terminalText,
    onSurface = LightShellowPalette.terminalText,
  )

@Composable
fun ShellowTheme(
  colorScheme: ShellowColorScheme = ShellowColorScheme.System,
  content: @Composable () -> Unit,
) {
  val darkTheme =
    when (colorScheme) {
      ShellowColorScheme.System -> isSystemInDarkTheme()
      ShellowColorScheme.Dark -> true
      ShellowColorScheme.Light -> false
    }
  val palette = if (darkTheme) DarkShellowPalette else LightShellowPalette
  SideEffect {
    ShellowColors.use(palette)
  }
  MaterialTheme(colorScheme = if (darkTheme) DarkShellowScheme else LightShellowScheme, content = content)
}
