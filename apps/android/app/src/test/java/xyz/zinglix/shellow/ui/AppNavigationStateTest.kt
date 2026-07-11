package xyz.zinglix.shellow.ui

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertSame
import org.junit.Assert.assertTrue
import org.junit.Test

class AppNavigationStateTest {
  @Test
  fun hostsIsTheOnlyExitDestination() {
    val navigation = AppNavigationState()

    assertEquals(AppScreen.Hosts, navigation.currentScreen)
    assertFalse(navigation.canNavigateBack)
    assertSame(navigation, navigation.navigateBack())
  }

  @Test
  fun everyTopLevelDestinationReturnsToHosts() {
    val destinations =
      listOf(
        AppScreen.Terminal,
        AppScreen.Codex,
        AppScreen.Claude,
        AppScreen.Settings,
      )

    destinations.forEach { destination ->
      val opened = AppNavigationState().navigateTo(destination)

      assertEquals(destination, opened.currentScreen)
      assertTrue(opened.canNavigateBack)
      assertEquals(AppScreen.Hosts, opened.navigateBack().currentScreen)
    }
  }

  @Test
  fun repeatedNavigationDoesNotCreateDuplicateDestinations() {
    val terminal = AppNavigationState().navigateTo(AppScreen.Terminal)

    assertSame(terminal, terminal.navigateTo(AppScreen.Terminal))
    assertEquals(AppScreen.Hosts, terminal.navigateBack().currentScreen)
  }

  @Test
  fun navigatingBetweenTopLevelDestinationsKeepsHostsAsTheirParent() {
    val settings =
      AppNavigationState()
        .navigateTo(AppScreen.Terminal)
        .navigateTo(AppScreen.Settings)

    assertEquals(AppScreen.Settings, settings.currentScreen)
    assertEquals(AppScreen.Hosts, settings.navigateBack().currentScreen)
  }

  @Test
  fun terminalBackClosesTransientUiBeforeLeavingTheScreen() {
    assertEquals(TerminalBackAction.CloseSearch, terminalBackAction(searchVisible = true, hasSelection = true))
    assertEquals(TerminalBackAction.ClearSelection, terminalBackAction(searchVisible = false, hasSelection = true))
    assertEquals(TerminalBackAction.NavigateBack, terminalBackAction(searchVisible = false, hasSelection = false))
  }
}
