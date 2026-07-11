package xyz.zinglix.shellow

import org.junit.Assert.assertEquals
import org.junit.Test
import xyz.zinglix.shellow.core.PersistentTerminalBackend
import xyz.zinglix.shellow.core.TerminalCursorShape
import xyz.zinglix.shellow.core.TerminalGridSnapshot
import xyz.zinglix.shellow.core.TerminalScreenKind
import xyz.zinglix.shellow.core.TerminalScrollDirection
import xyz.zinglix.shellow.core.scrollInputSequence

class TerminalScrollInputTest {
  @Test
  fun sgrMouseScroll_encodesWheelAtViewportCenter() {
    val grid = grid(mouseReporting = true, sgrMouse = true)

    assertEquals(
      "\u001B[<64;40;12M\u001B[<64;40;12M",
      grid.scrollInputSequence(
        direction = TerminalScrollDirection.Up,
        count = 2,
        backend = PersistentTerminalBackend.Tmux,
        enterScrollMode = true,
      ),
    )
  }

  @Test
  fun fallbackScroll_entersBackendModeOnceThenSendsArrowKeys() {
    val grid = grid(mouseReporting = false, sgrMouse = false)

    assertEquals(
      "\u0001[\u001B[B\u001B[B",
      grid.scrollInputSequence(
        direction = TerminalScrollDirection.Down,
        count = 2,
        backend = PersistentTerminalBackend.Screen,
        enterScrollMode = true,
      ),
    )
    assertEquals(
      "\u001B[A",
      grid.scrollInputSequence(
        direction = TerminalScrollDirection.Up,
        count = 1,
        backend = PersistentTerminalBackend.Screen,
        enterScrollMode = false,
      ),
    )
  }

  private fun grid(mouseReporting: Boolean, sgrMouse: Boolean) =
    TerminalGridSnapshot(
      cols = 80,
      rows = 24,
      cursorColumn = 0,
      cursorRow = 0,
      cursorVisible = true,
      cursorShape = TerminalCursorShape.Block,
      activeScreen = TerminalScreenKind.Alternate,
      scrollbackLen = 0,
      bracketedPaste = false,
      applicationCursorKeys = false,
      mouseReporting = mouseReporting,
      mouseDragReporting = false,
      sgrMouse = sgrMouse,
      lines = emptyList(),
      styledLines = emptyList(),
      dirtyRows = emptyList(),
    )
}
