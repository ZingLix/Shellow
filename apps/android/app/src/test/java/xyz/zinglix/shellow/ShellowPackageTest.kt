package xyz.zinglix.shellow

import org.junit.Assert.assertEquals
import org.junit.Test

class ShellowPackageTest {
  @Test
  fun packageName_isStable() {
    assertEquals("xyz.zinglix.shellow", javaClass.packageName)
  }
}
