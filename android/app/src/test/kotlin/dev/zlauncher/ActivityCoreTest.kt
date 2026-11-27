package dev.zlauncher

import dev.zlauncher.core.nativecore.Core
import dev.zlauncher.core.nativecore.CoreException
import dev.zlauncher.core.nativecore.ScanEvent
import dev.zlauncher.core.nativecore.ScanListener
import dev.zlauncher.core.nativecore.SessionSource
import java.io.File
import java.nio.file.Files
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Test

/** The Activity tab's data and the session edits, through the real Rust library (RF-31, RF-41). */
class ActivityCoreTest {
    /** A scan that ignores its progress. */
    private object Quiet : ScanListener {
        override fun onEvent(event: ScanEvent) = Unit
    }

    /** A core over one GBA game; the core and the game's id. */
    private fun coreWithGame(): Pair<Core, Long> {
        val dir = Files.createTempDirectory("zl-activity-test").toFile()
        val roms = File(dir, "Games/gba").apply { mkdirs() }
        File(roms, "Alpha.gba").writeText("x")
        val core = Core(File(dir, "data").path)
        core.addRoot(File(dir, "Games").path, null)
        core.scan(Quiet)
        return core to core.games("gba").single().id
    }

    /** Monday 2024-01-01 00:00 UTC. */
    private val monday = 1_704_067_200L

    /** Sessions added by hand count in the totals and the chart, and can be corrected and deleted. */
    @Test
    fun sessionsAreSummedCorrectedAndDeleted() {
        val (core, game) = coreWithGame()
        val first = core.addManualSession(game, monday + 3_600, 3_600)
        core.addManualSession(game, monday - 86_400, 1_800)
        val tab = core.gameActivity(game, monday + 2 * 86_400, 0)
        assertEquals(5_400L, tab.summary.totalSeconds)
        assertEquals(2L, tab.summary.sessionCount)
        assertEquals(2_700L, tab.summary.averageSeconds)
        assertEquals(monday + 7_200, tab.summary.lastPlayed)
        assertEquals(listOf(1_800L, 3_600L), tab.summary.weeks.takeLast(2).map { it.seconds })
        assertTrue(tab.sessions.all { it.source == SessionSource.MANUAL })
        core.setSessionSeconds(first, 600)
        assertEquals(600L, core.gameActivity(game, monday, 0).sessions.first().seconds)
        core.deleteSession(first)
        assertEquals(1L, core.gameActivity(game, monday, 0).summary.sessionCount)
    }

    /** What the core refuses reaches Kotlin as an exception, not as a wrong number. */
    @Test
    fun badEditsAreRefused() {
        val (core, game) = coreWithGame()
        assertThrows(CoreException::class.java) { core.addManualSession(game, monday, 0) }
        assertThrows(CoreException::class.java) { core.addManualSession(game, monday, 90_000) }
        assertThrows(CoreException::class.java) { core.deleteSession(12_345) }
    }

    /** An excluded app records no session; putting it back records again. */
    @Test
    fun anExcludedAppIsNotTracked() {
        val (core, game) = coreWithGame()
        core.beginSession(game, "com.emu", 0)
        assertEquals("com.emu", core.trackingPackage(game))
        core.setAppTrackingExcluded("com.emu", true)
        assertTrue(core.isAppTrackingExcluded("com.emu"))
        core.beginSession(game, "com.emu", 0)
        assertEquals(null, core.pendingSession())
        core.setAppTrackingExcluded("com.emu", false)
        assertFalse(core.isAppTrackingExcluded("com.emu"))
        core.beginSession(game, "com.emu", 0)
        assertEquals("com.emu", core.pendingSession()?.`package`)
    }
}
