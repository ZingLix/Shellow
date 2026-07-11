package xyz.zinglix.shellow.core

enum class TerminalScrollDirection(
  val wheelButtonCode: Int,
  val arrowSequence: String,
) {
  Up(64, "\u001B[A"),
  Down(65, "\u001B[B"),
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
  return modePrefix + direction.arrowSequence.repeat(eventCount)
}
