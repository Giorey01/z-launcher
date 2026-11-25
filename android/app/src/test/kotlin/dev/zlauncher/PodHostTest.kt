package dev.zlauncher

import dev.zlauncher.core.nativecore.Core
import dev.zlauncher.core.nativecore.GameEntry
import dev.zlauncher.core.nativecore.GameKind
import dev.zlauncher.pods.AppPodHost
import dev.zlauncher.pods.PermissionNotDeclaredException
import dev.zlauncher.ui.theme.ZTheme
import dev.zlauncher.ui.theme.toZTheme
import java.nio.file.Files
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Test

/** Tests runtime permission checking in PodHost (§8). */
class PodHostTest {
    private val core = Core(Files.createTempDirectory("zl-pod-host-test").toString())
    private val theme: ZTheme = core.activeTheme().toZTheme(1)

    private fun dummyGame(): GameEntry = GameEntry(
        id = 1L,
        title = "Super Mario",
        platformId = "snes",
        platformName = "Super Nintendo",
        kind = GameKind.ROM,
        missing = false,
        displayTitle = "Super Mario",
        icon = null,
        hero = null,
        logo = null,
        appPackage = null,
        favorite = false,
    )

    @Test
    fun libraryReadRequiredForFoldersAndGames() {
        val host = AppPodHost(
            podId = "test.pod",
            permissions = emptySet(),
            app = null,
            theme = theme,
        )

        val errFolders = assertThrows(PermissionNotDeclaredException::class.java) {
            runBlocking { host.folders() }
        }
        assertEquals("library.read", errFolders.permission)

        val errGames = assertThrows(PermissionNotDeclaredException::class.java) {
            runBlocking { host.games(null) }
        }
        assertEquals("library.read", errGames.permission)

        val errDetails = assertThrows(PermissionNotDeclaredException::class.java) {
            runBlocking { host.details(123L) }
        }
        assertEquals("library.read", errDetails.permission)

        val errStats = assertThrows(PermissionNotDeclaredException::class.java) {
            runBlocking { host.playStats(123L) }
        }
        assertEquals("library.read", errStats.permission)
    }

    @Test
    fun launchRequiredForLaunch() {
        val host = AppPodHost(
            podId = "test.pod",
            permissions = setOf("library.read"),
            app = null,
            theme = theme,
        )
        val err = assertThrows(PermissionNotDeclaredException::class.java) {
            host.launch(dummyGame())
        }
        assertEquals("launch", err.permission)
    }

    @Test
    fun rescanRequiredForRescan() {
        val host = AppPodHost(
            podId = "test.pod",
            permissions = emptySet(),
            app = null,
            theme = theme,
        )
        val err = assertThrows(PermissionNotDeclaredException::class.java) {
            runBlocking { host.rescan("snes") }
        }
        assertEquals("library.rescan", err.permission)
    }

    @Test
    fun secretsRequiredForKeystore() {
        val host = AppPodHost(
            podId = "test.pod",
            permissions = emptySet(),
            app = null,
            theme = theme,
        )
        val errGet = assertThrows(PermissionNotDeclaredException::class.java) {
            runBlocking { host.getSecret("api_key") }
        }
        assertEquals("secrets", errGet.permission)

        val errSet = assertThrows(PermissionNotDeclaredException::class.java) {
            runBlocking { host.setSecret("api_key", "val") }
        }
        assertEquals("secrets", errSet.permission)
    }

    @Test
    fun httpRequiredForHttp() {
        val host = AppPodHost(
            podId = "test.pod",
            permissions = emptySet(),
            app = null,
            theme = theme,
        )
        val errGet = assertThrows(PermissionNotDeclaredException::class.java) {
            runBlocking { host.httpGet("https://example.com") }
        }
        assertEquals("http", errGet.permission)

        val errPost = assertThrows(PermissionNotDeclaredException::class.java) {
            runBlocking { host.httpPost("https://example.com", ByteArray(0), "application/json") }
        }
        assertEquals("http", errPost.permission)
    }

    @Test
    fun foldersRequiredForPlatformRoot() {
        val host = AppPodHost(
            podId = "test.pod",
            permissions = emptySet(),
            app = null,
            theme = theme,
        )
        val err = assertThrows(PermissionNotDeclaredException::class.java) {
            runBlocking { host.platformRoot("snes") }
        }
        assertEquals("folders", err.permission)
    }

    @Test
    fun storageAndStringsAlwaysAccessible() {
        val host = AppPodHost(
            podId = "test.pod",
            permissions = emptySet(),
            app = null,
            theme = theme,
        )
        assertEquals("hello", host.strings.get("hello"))
        host.storage.setString("k", "v")
        assertEquals("v", host.storage.getString("k"))
        host.storage.setBoolean("flag", true)
        assertEquals(true, host.storage.getBoolean("flag"))
        host.storage.setInt("count", 42)
        assertEquals(42, host.storage.getInt("count"))
    }
}
