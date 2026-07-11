package xyz.zinglix.shellow.ui

internal enum class AppScreen {
  Hosts,
  Terminal,
  Codex,
  Claude,
  Settings,
}

internal data class AppNavigationState(
  private val backStack: List<AppScreen> = listOf(AppScreen.Hosts),
) {
  init {
    require(backStack.isNotEmpty()) { "The app navigation stack cannot be empty." }
    require(backStack.first() == AppScreen.Hosts) { "Hosts must remain the root destination." }
  }

  val currentScreen: AppScreen
    get() = backStack.last()

  val canNavigateBack: Boolean
    get() = backStack.size > 1

  fun navigateTo(destination: AppScreen): AppNavigationState =
    when {
      destination == AppScreen.Hosts -> AppNavigationState()
      destination == currentScreen -> this
      else -> AppNavigationState(listOf(AppScreen.Hosts, destination))
    }

  fun navigateBack(): AppNavigationState =
    if (canNavigateBack) {
      AppNavigationState(backStack.dropLast(1))
    } else {
      this
    }
}

internal enum class TerminalBackAction {
  CloseSearch,
  ClearSelection,
  NavigateBack,
}

internal fun terminalBackAction(
  searchVisible: Boolean,
  hasSelection: Boolean,
): TerminalBackAction =
  when {
    searchVisible -> TerminalBackAction.CloseSearch
    hasSelection -> TerminalBackAction.ClearSelection
    else -> TerminalBackAction.NavigateBack
  }
