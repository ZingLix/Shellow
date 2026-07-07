package xyz.zinglix.shellow

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class ShellowApplicationIdTest {
  @Test
  fun applicationId_isStable() {
    val context = InstrumentationRegistry.getInstrumentation().targetContext
    assertEquals("xyz.zinglix.shellow", context.packageName)
  }
}
