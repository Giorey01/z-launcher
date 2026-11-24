package dev.zlauncher

import dev.zlauncher.core.nativecore.coreVersion
import dev.zlauncher.core.nativecore.dataDirName
import java.io.File
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/** Calls the real Rust library built for the host machine through the UniFFI bindings. */
class CoreVersionTest {
    /** The `version` of `[workspace.package]` in the workspace manifest. */
    private fun workspaceVersion(): String {
        val manifest = File(checkNotNull(System.getProperty("zl.workspaceManifest"))).readText()
        val section = manifest.substringAfter("[workspace.package]")
        return Regex("""(?m)^version\s*=\s*"([^"]+)"""").find(section)?.groupValues?.get(1)
            ?: error("no version in [workspace.package]")
    }

    /** The core reports the version the workspace declares. */
    @Test
    fun coreVersionMatchesWorkspace() {
        assertEquals(workspaceVersion(), coreVersion())
    }

    /** The data folder name crosses the FFI intact. */
    @Test
    fun dataDirNameComesFromConfig() {
        assertEquals("Z-Launcher", dataDirName())
        assertTrue(dataDirName().isNotBlank())
    }
}
