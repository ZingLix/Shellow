package xyz.zinglix.shellow

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Test
import xyz.zinglix.shellow.core.AuthenticationKind
import xyz.zinglix.shellow.core.HostProfile
import xyz.zinglix.shellow.core.ProfileLaunchKind
import xyz.zinglix.shellow.core.duplicateProfileName
import xyz.zinglix.shellow.core.duplicated

class ProfileDuplicationTest {
  @Test
  fun duplicateNameUsesTheFirstAvailableSuffix() {
    assertEquals(
      "Production Copy 3",
      duplicateProfileName(
        "Production",
        listOf("Production", "Production Copy", "production copy 2"),
      ),
    )
  }

  @Test
  fun duplicatingAnExistingCopyContinuesTheSequence() {
    assertEquals(
      "Production Copy 2",
      duplicateProfileName("Production Copy", listOf("Production", "Production Copy")),
    )
  }

  @Test
  fun duplicatedProfileKeepsConfigurationButGetsANewIdentity() {
    val original =
      HostProfile(
        name = "Production",
        host = "prod.example.com",
        port = 2222,
        username = "deploy",
        authentication = AuthenticationKind.PrivateKey,
        preferredKeyId = "key-1",
        launchKind = ProfileLaunchKind.Codex,
        trustedHostKeySha256 = "SHA256:test",
        id = "profile-1",
      )

    val duplicate = original.duplicated(listOf(original.name))

    assertNotEquals(original.id, duplicate.id)
    assertEquals("Production Copy", duplicate.name)
    assertEquals(original.host, duplicate.host)
    assertEquals(original.port, duplicate.port)
    assertEquals(original.username, duplicate.username)
    assertEquals(original.authentication, duplicate.authentication)
    assertEquals(original.preferredKeyId, duplicate.preferredKeyId)
    assertEquals(original.launchKind, duplicate.launchKind)
    assertEquals(original.trustedHostKeySha256, duplicate.trustedHostKeySha256)
  }
}
