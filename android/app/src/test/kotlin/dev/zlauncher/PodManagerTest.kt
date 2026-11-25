package dev.zlauncher

import dev.zlauncher.pods.PodManagementState
import dev.zlauncher.pods.PodTab
import dev.zlauncher.pods.formatFileSize
import dev.zlauncher.pods.formatSignerFingerprint
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Test

/** Tests for PodManager state formatting and business logic. */
class PodManagerTest {

    @Test
    fun signerFingerprintFormattedInUppercaseChunksOfFour() {
        val rawSigner = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        val formatted = formatSignerFingerprint(rawSigner)
        assertEquals("E3B0 C442 98FC 1C14 9AFB F4C8 996F B924", formatted)
    }

    @Test
    fun fileSizesFormattedCorrectly() {
        assertEquals("500 KB", formatFileSize(500 * 1024L))
        assertEquals("1.5 MB", formatFileSize((1.5 * 1024 * 1024).toLong()))
        assertEquals("10.0 MB", formatFileSize(10 * 1024 * 1024L))
    }

    @Test
    fun defaultStateHasExternalPodsDisabled() {
        val state = PodManagementState()
        assertFalse(state.externalPodsEnabled)
        assertEquals(PodTab.INSTALLED, state.activeTab)
        assertFalse(state.showExternalPrompt)
        assertFalse(state.showUrlDialog)
    }
}
