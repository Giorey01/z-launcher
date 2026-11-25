package dev.zlauncher

import dev.zlauncher.ui.drawer.foldForSearch
import dev.zlauncher.ui.drawer.gridIndexOf
import dev.zlauncher.ui.drawer.matchesSearch
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/** The pure rules of the drawer: search matching and the position of a focus key in the grid. */
class DrawerSearchTest {
    /** Accents and case are ignored on both sides. */
    @Test
    fun searchIgnoresAccentsAndCase() {
        assertTrue(matchesSearch("Pokémon GO", "pokemon"))
        assertTrue(matchesSearch("Pokemon GO", "POKÉ"))
        assertTrue(matchesSearch("Caffè Nero", "caffe n"))
        assertFalse(matchesSearch("Pokémon GO", "zelda"))
        assertEquals("cafe", foldForSearch("Café"))
    }

    /** Blanks around the query do not hide matches; a blank query shows everything. */
    @Test
    fun blankQueryMatchesEverything() {
        assertTrue(matchesSearch("Anything", ""))
        assertTrue(matchesSearch("Anything", "   "))
        assertTrue(matchesSearch("Super Mario", "super "))
        assertFalse(matchesSearch("Super Mario", "super  mario"))
    }

    /** With recent apps the divider takes one cell; without them there is no offset. */
    @Test
    fun gridIndexCountsTheDividerOnlyWhenThereAreRecentApps() {
        val recent = listOf("r:a", "r:b")
        val apps = listOf("a:a", "a:b", "a:c")
        assertEquals(1, gridIndexOf("r:b", recent, apps))
        assertEquals(3, gridIndexOf("a:a", recent, apps))
        assertEquals(0, gridIndexOf("a:a", emptyList(), apps))
        assertEquals(2, gridIndexOf("a:c", emptyList(), apps))
    }

    /** A key that is not shown has no position, so the caller falls back to its default. */
    @Test
    fun hiddenKeyHasNoIndex() {
        assertNull(gridIndexOf("r:a", emptyList(), listOf("a:a")))
        assertNull(gridIndexOf("a:zzz", listOf("r:a"), listOf("a:a")))
    }
}
