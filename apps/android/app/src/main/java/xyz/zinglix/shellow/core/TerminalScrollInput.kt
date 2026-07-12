package xyz.zinglix.shellow.core

enum class TerminalScrollDirection(
  val wheelButtonCode: Int,
  val pageScrollSequence: String,
) {
  Up(64, "\u0002"),
  Down(65, "\u0006"),
}

fun TerminalGridSnapshot.scrollInputSequence(
  direction: TerminalScrollDirection,
  count: Int,
  backend: PersistentTerminalBackend?,
  enterScrollMode: Boolean,
): String {
  val eventCount = count.coerceAtLeast(1)
  if (mouseReporting && sgrMouse) {
    val column = (cols / 2).coerceAtLeast(1)
    val row = (rows / 2).coerceAtLeast(1)
    return "\u001B[<${direction.wheelButtonCode};$column;${row}M".repeat(eventCount)
  }

  val modePrefix = if (enterScrollMode) backend?.scrollModeSequence.orEmpty() else ""
  // Alternate-screen programs own their scrollback. Ctrl-B/Ctrl-F move the
  // viewport in tmux, screen, and Zellij without moving a copy-mode cursor.
  return modePrefix + direction.pageScrollSequence.repeat(eventCount)
}
