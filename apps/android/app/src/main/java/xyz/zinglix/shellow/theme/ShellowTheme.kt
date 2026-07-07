package xyz.zinglix.shellow.theme

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

object ShellowColors {
  val Accent = Color(0xFF1C9F70)
  val Success = Color(0xFF45D18C)
  val Warning = Color(0xFFEDAD38)
  val TerminalText = Color(0xFFE0E8DE)
  val TerminalMuted = Color(0xFF95A39B)
  val TerminalBackground = Color(0xFF0D0F0E)
  val PanelBackground = Color(0xFF151817)
  val InputBackground = Color(0xFF202422)
  val KeyBackground = Color(0xFF272C29)
}

private val ShellowScheme =
  darkColorScheme(
    primary = ShellowColors.Accent,
    secondary = ShellowColors.Success,
    tertiary = ShellowColors.Warning,
    background = ShellowColors.TerminalBackground,
    surface = ShellowColors.PanelBackground,
    onPrimary = Color.White,
    onSecondary = Color.Black,
    onTertiary = Color.Black,
    onBackground = ShellowColors.TerminalText,
    onSurface = ShellowColors.TerminalText,
  )

@Composable
fun ShellowTheme(content: @Composable () -> Unit) {
  MaterialTheme(colorScheme = ShellowScheme, content = content)
}
