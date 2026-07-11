package xyz.zinglix.shellow

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import xyz.zinglix.shellow.core.AuthenticationKind
import xyz.zinglix.shellow.core.HostProfile
import xyz.zinglix.shellow.core.PersistentTerminalBackend
import xyz.zinglix.shellow.core.PersistentTerminalConfiguration
import xyz.zinglix.shellow.core.ProfileLaunchKind
import xyz.zinglix.shellow.core.RemoteComponentSupportLevel
import xyz.zinglix.shellow.core.RemoteHostCapabilityProbe
import xyz.zinglix.shellow.core.RemoteTerminalSessionProbe

class PersistentTerminalTest {
  @Test
  fun sessionNameValidation_rejectsShellSyntax() {
    assertEquals("shellow-main_2", PersistentTerminalConfiguration.validatedName(" shellow-main_2 "))
    assertNull(PersistentTerminalConfiguration.validatedName(""))
    assertNull(PersistentTerminalConfiguration.validatedName("-starts-with-dash"))
    assertNull(PersistentTerminalConfiguration.validatedName("main; reboot"))
    assertNull(PersistentTerminalConfiguration.validatedName("会话"))
  }

  @Test
  fun startupCommands_attachOrCreateEachBackend() {
    val tmux = profile(PersistentTerminalBackend.Tmux).terminalStartupCommand
    val screen = profile(PersistentTerminalBackend.Screen).terminalStartupCommand
    val zellij = profile(PersistentTerminalBackend.Zellij).terminalStartupCommand

    assertTrue(tmux.contains("tmux new-session -A -s shellow-main"))
    assertTrue(screen.contains("screen -ls"))
    assertTrue(screen.contains("screen -D -R"))
    assertTrue(screen.contains("screen -S shellow-main"))
    assertTrue(zellij.contains("zellij attach --create shellow-main"))
  }

  @Test
  fun capabilityParser_reportsSystemAndCompletesMissingComponents() {
    val report =
      RemoteHostCapabilityProbe.parse(
        """
        ignored banner
        __SHELLOW_CAPABILITIES_V1__
        system|Linux|Ubuntu|24.04|6.8.0|aarch64|/bin/zsh
        component|tmux|supported|tmux 3.4
        component|screen|limited|Screen version 4.09.01
        """.trimIndent(),
        detectedAtEpochMillis = 42L,
      )

    assertNotNull(report)
    assertEquals("Ubuntu 24.04", report?.system?.displayTitle)
    assertEquals("zsh", report?.system?.shellName)
    assertEquals(RemoteComponentSupportLevel.Supported, report?.capability(PersistentTerminalBackend.Tmux)?.supportLevel)
    assertEquals(RemoteComponentSupportLevel.Limited, report?.capability(PersistentTerminalBackend.Screen)?.supportLevel)
    assertEquals(RemoteComponentSupportLevel.Unavailable, report?.capability(PersistentTerminalBackend.Zellij)?.supportLevel)
    assertFalse(report?.components.orEmpty().isEmpty())
  }

  @Test
  fun capabilityParser_requiresMarkerAndSystemLine() {
    assertNull(RemoteHostCapabilityProbe.parse("system|Linux|Linux||||"))
    assertNull(RemoteHostCapabilityProbe.parse("__SHELLOW_CAPABILITIES_V1__\ncomponent|tmux|supported|tmux 3.4"))
  }

  @Test
  fun sessionCatalogParser_readsAndSortsRemoteWorkspaces() {
    val catalog =
      RemoteTerminalSessionProbe.parse(
        """
        login banner
        __SHELLOW_SESSIONS_V1__
        session|zeta|0|1
        session|main|1|3
        session|main|1|3
        """.trimIndent(),
      )

    assertNotNull(catalog)
    assertEquals(listOf("main", "zeta"), catalog?.sessions?.map { it.name })
    assertTrue(catalog?.sessions?.first()?.isAttached == true)
    assertEquals(3, catalog?.sessions?.first()?.windowCount)
  }

  @Test
  fun missingLaunchKindDefaultsToTerminal() {
    assertEquals(ProfileLaunchKind.Terminal, ProfileLaunchKind.fromWire(null))
    assertEquals(ProfileLaunchKind.Terminal, ProfileLaunchKind.fromWire(""))
  }

  @Test
  fun authenticationKind_decodesSavedAndUnknownValues() {
    assertEquals(AuthenticationKind.Password, AuthenticationKind.fromWire(0))
    assertEquals(AuthenticationKind.PrivateKey, AuthenticationKind.fromWire(1))
    assertEquals(AuthenticationKind.Automatic, AuthenticationKind.fromWire(2))
    assertEquals(AuthenticationKind.Automatic, AuthenticationKind.fromWire(99))
  }

  private fun profile(backend: PersistentTerminalBackend) =
    HostProfile(
      name = "Lab",
      host = "lab.example.com",
      port = 22,
      username = "operator",
      authentication = AuthenticationKind.Password,
      persistentTerminal = PersistentTerminalConfiguration("shellow-main", backend),
    )
}
