package dev.zlauncher

import dev.zlauncher.core.nativecore.Core
import dev.zlauncher.core.nativecore.GameEntry
import dev.zlauncher.core.nativecore.GameKind
import dev.zlauncher.core.nativecore.SoundAction
import dev.zlauncher.pods.AppPodHost
import dev.zlauncher.pods.PermissionNotDeclaredException
import dev.zlauncher.pods.PodEvent
import dev.zlauncher.pods.PodHostRegistry
import dev.zlauncher.ui.theme.ZTheme
import dev.zlauncher.ui.theme.toZTheme
import java.nio.file.Files
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.yield
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
    fun archiveRequiredForExtract() {
        val host = AppPodHost(podId = "test.pod", permissions = setOf("http"), app = null, theme = theme)
        val dir = Files.createTempDirectory("pod-archive").toFile()
        val err = assertThrows(PermissionNotDeclaredException::class.java) {
            runBlocking { host.extract(java.io.File(dir, "a.zip"), dir) }
        }
        assertEquals("archive", err.permission)
    }

    @Test
    fun extractGoesThroughTheCoreAndKeepsEntriesInside() {
        val host = AppPodHost(podId = "test.pod", permissions = setOf("archive"), app = null, theme = theme)
        val dir = Files.createTempDirectory("pod-archive").toFile()
        val zip = java.io.File(dir, "mod.zip")
        java.util.zip.ZipOutputStream(zip.outputStream()).use { out ->
            out.putNextEntry(java.util.zip.ZipEntry("/"))
            out.closeEntry()
            out.putNextEntry(java.util.zip.ZipEntry("Mod/romfs/a.bin"))
            out.write(ByteArray(1000) { 7 })
            out.closeEntry()
        }
        var last = -1L
        val files = runBlocking { host.extract(zip, java.io.File(dir, "out")) { done, _ -> last = done } }
        assertEquals(listOf("a.bin"), files.map { it.name })
        assertEquals(1000L, java.io.File(dir, "out/Mod/romfs/a.bin").length())
        assertEquals(1000L, last)
        val bad = java.io.File(dir, "bad.zip")
        java.util.zip.ZipOutputStream(bad.outputStream()).use { out ->
            out.putNextEntry(java.util.zip.ZipEntry("../evil.bin"))
            out.write(1)
            out.closeEntry()
        }
        assertThrows(dev.zlauncher.core.nativecore.TransferException.Unsafe::class.java) {
            runBlocking { host.extract(bad, java.io.File(dir, "out2")) }
        }
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

    /** The notice needs no permission and, with no launcher behind the host (a test), does nothing and does not throw. */
    @Test
    fun toastNeedsNoPermission() {
        val host = AppPodHost(podId = "test.pod", permissions = emptySet(), app = null, theme = theme)
        host.toast("Level up", SoundAction.NOTIFICATION)
        host.toast("   ", null)
    }

    /** The end of a game reaches the hosts on screen that may read the library, and only them. */
    @Test
    fun gameEndedReachesTheHostsOnScreen() = runBlocking {
        val reader = AppPodHost(podId = "reader", permissions = setOf("library.read"), app = null, theme = theme)
        val blind = AppPodHost(podId = "blind", permissions = emptySet(), app = null, theme = theme)
        val away = AppPodHost(podId = "away", permissions = setOf("library.read"), app = null, theme = theme)
        val hosts = PodHostRegistry()
        hosts.add(reader)
        hosts.add(blind)
        hosts.add(away)
        hosts.remove(away)
        assertEquals(2, hosts.size())
        val seen = mutableListOf<PodEvent>()
        val job = launch(start = CoroutineStart.UNDISPATCHED) { reader.events.collect { seen += it } }
        var asked = 0L
        hosts.announceGameEnded(dummyGame().id, 600L) { id -> dummyGame().also { asked = id } }
        yield()
        job.cancel()
        assertEquals(dummyGame().id, asked)
        assertEquals(listOf<PodEvent>(PodEvent.GameEnded(dummyGame(), 600L)), seen)
        val none = PodHostRegistry()
        none.announceGameEnded(1L, 1L) { error("nobody listens: the game is not looked up") }
    }
}
