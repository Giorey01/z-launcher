package dev.zlauncher

import dev.zlauncher.core.nativecore.SessionRow
import dev.zlauncher.core.nativecore.SessionSource
import dev.zlauncher.ui.info.ActivityDialog
import dev.zlauncher.ui.info.InfoTabs
import dev.zlauncher.ui.info.MAX_MANUAL_SECONDS
import dev.zlauncher.ui.info.MIN_MANUAL_SECONDS
import dev.zlauncher.ui.info.adjustDuration
import dev.zlauncher.ui.info.barFractions
import dev.zlauncher.ui.info.manualSessionStart
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/** The pure parts of the game sheet: the chart's bars, the date of a session typed in, and the tabs' state. */
class ActivityTabTest {
    /** Monday 2024-01-01 00:00 UTC. */
    private val monday = 1_704_067_200L

    /** The tallest week fills the chart, the others follow it, a week with time never vanishes and an empty one is flat. */
    @Test
    fun barsFollowTheBusiestWeek() {
        assertEquals(listOf(0f, 0.5f, 1f), barFractions(listOf(0L, 50L, 100L)))
        val tiny = barFractions(listOf(1L, 1_000_000L))
        assertTrue("a short week still shows", tiny[0] > 0f)
        assertEquals(listOf(0f, 0f), barFractions(listOf(0L, 0L)))
        assertEquals(emptyList<Float>(), barFractions(emptyList()))
    }

    /** A session added for today ends now; one for an earlier day ends at 22:00 there, in the user's time zone. */
    @Test
    fun aSessionAddedByHandStartsWhereTheDayAndDurationSay() {
        val now = monday + 15 * 3_600 + 600
        assertEquals(now - 3_600, manualSessionStart(now, 0, 0, 3_600))
        assertEquals(monday - 86_400 + 22 * 3_600 - 1_800, manualSessionStart(now, 0, 1, 1_800))
        val twoEast = 2 * 3_600L
        assertEquals(monday - 2 * 86_400 + 22 * 3_600 - twoEast - 600, manualSessionStart(now, twoEast, 2, 600))
    }

    /** Late on the evening of the same day, "today" never ends in the future. */
    @Test
    fun anEarlierDayNeverEndsAfterNow() {
        val now = monday + 3_600
        val start = manualSessionStart(now, 0, 1, 60)
        assertTrue(start + 60 <= now)
    }

    /** The length of a session typed in stays between a minute and a day. */
    @Test
    fun durationsStayInRange() {
        assertEquals(MIN_MANUAL_SECONDS, adjustDuration(MIN_MANUAL_SECONDS, -3_600))
        assertEquals(MAX_MANUAL_SECONDS, adjustDuration(MAX_MANUAL_SECONDS, 3_600))
        assertEquals(7_200L, adjustDuration(3_600, 3_600))
    }

    /** LB and RB step through the tabs and stop at the ends; a dialog freezes them and B closes it first. */
    @Test
    fun tabsStepAndStopAtTheEnds() {
        val tabs = InfoTabs()
        tabs.open(3)
        assertFalse("nothing before the first tab", tabs.step(-1))
        assertTrue(tabs.step(1))
        assertTrue(tabs.step(1))
        assertEquals(2, tabs.selected)
        assertFalse("nothing after the last", tabs.step(1))
        tabs.dialog = ActivityDialog.Add
        assertFalse(tabs.step(-1))
        assertEquals(2, tabs.selected)
        assertTrue(tabs.dismissDialog())
        assertNull(tabs.dialog)
        assertFalse("B leaves the screen when there is no dialog", tabs.dismissDialog())
        assertTrue(tabs.step(-1))
    }

    /** A Pod turned off while its tab is open leaves the sheet on a tab that exists; opening a sheet starts at Info. */
    @Test
    fun resizeKeepsTheOpenTabValid() {
        val tabs = InfoTabs()
        tabs.open(4)
        tabs.select(3)
        tabs.resize(2)
        assertEquals(1, tabs.selected)
        tabs.select(5)
        assertEquals(1, tabs.selected)
        tabs.dialog = ActivityDialog.Edit(SessionRow(1, 1, 0, 60, 60, SessionSource.MANUAL))
        tabs.open(2)
        assertEquals(0, tabs.selected)
        assertNull(tabs.dialog)
    }

    /** Changes of the sessions are told to the tab with a counter. */
    @Test
    fun sessionChangesBumpTheVersion() {
        val tabs = InfoTabs()
        val before = tabs.sessionsVersion
        tabs.sessionsChanged()
        assertEquals(before + 1, tabs.sessionsVersion)
    }
}
