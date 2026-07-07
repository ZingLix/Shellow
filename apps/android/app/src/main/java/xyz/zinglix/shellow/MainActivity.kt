package xyz.zinglix.shellow

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import xyz.zinglix.shellow.theme.ShellowTheme
import xyz.zinglix.shellow.ui.ShellowApp

class MainActivity : ComponentActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)

    enableEdgeToEdge()
    setContent {
      ShellowTheme {
        ShellowApp()
      }
    }
  }
}
