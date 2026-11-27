package dev.zlauncher

import dev.zlauncher.core.nativecore.GameEntry
import dev.zlauncher.core.nativecore.GameKind
import dev.zlauncher.pods.MAX_BADGE_CHARS
import dev.zlauncher.pods.MAX_GAME_BADGES
import dev.zlauncher.pods.MAX_GAME_TABS
import dev.zlauncher.pods.MAX_TOAST_CHARS
import dev.zlauncher.pods.PodRegistry
import dev.zlauncher.pods.PodSlot
import dev.zlauncher.pods.PodWithMetadata
import dev.zlauncher.pods.toastText
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertThrows
import org.junit.Test

/** The slots a Pod uses on a game's sheet: tabs (RF-78), badges (RF-79), and the text of its notices (RF-80). */
class GameSheetSlotsTest {
    /** A game of a given platform. */
    private fun game(platform: String = "snes", id: Long = 7L) = GameEntry(
        id = id, title = "Game", platformId = platform, platformName = platform, kind = GameKind.ROM, missing = false,
        displayTitle = "Game", icon = null, hero = null, logo = null, appPackage = null, favorite = false,
    )

    /** A registry of one external Pod with [slots]; [declared] are the slot names its manifest lists. */
    private fun registry(
        id: String,
        slots: List<PodSlot>,
        declared: Set<String> = setOf("game_tab", "game_badge"),
        crashes: MutableList<String> = mutableListOf(),
    ): PodRegistry = PodRegistry.from(
        listOf(PodWithMetadata(FakePod(id, slots), declared, emptySet(), isExternal = true)),
        onCrash = { podId, _ -> crashes += podId },
    )

    /** A tab with a label and an optional condition. */
    private fun tab(label: String, condition: (GameEntry) -> Boolean = { true }) =
        PodSlot.GameTab(label = label, condition = condition, content = { _, _ -> })

    /** A Pod's tab shows in the game's sheet, in the order declared, and only for the games its condition accepts. */
    @Test
    fun aPodTabAppearsForTheGamesItAccepts() {
        val registry = registry("p1", listOf(tab("Achievements"), tab("Only GBA") { it.platformId == "gba" }))
        assertEquals(listOf("Achievements"), registry.gameTabSlots(game("snes")).map { it.label })
        assertEquals(listOf("Achievements", "Only GBA"), registry.gameTabSlots(game("gba")).map { it.label })
        assertEquals(listOf("p1", "p1"), registry.gameTabSlots(game("gba")).map { it.podId })
    }

    /** A tab the manifest did not declare is not drawn (external Pods), and a condition that throws costs the Pod a crash. */
    @Test
    fun undeclaredTabsAreIgnoredAndFailingConditionsCount() {
        val crashes = mutableListOf<String>()
        val undeclared = registry("p1", listOf(tab("Hidden")), declared = setOf("start_menu"))
        assertEquals(emptyList<String>(), undeclared.gameTabSlots(game()).map { it.label })
        val failing = registry("p2", listOf(tab("Boom") { error("boom") }, tab("Fine")), crashes = crashes)
        assertEquals(listOf("Fine"), failing.gameTabSlots(game()).map { it.label })
        assertEquals(listOf("p2"), crashes)
    }

    /** No more than [MAX_GAME_TABS] tabs, whatever the Pods declare (a Pod's own limit is 16 slots). */
    @Test
    fun tabsAreLimited() {
        val registry = registry("p1", (1..MAX_GAME_TABS + 3).map { tab("Tab $it") })
        assertEquals(MAX_GAME_TABS, registry.gameTabSlots(game()).size)
    }

    /** A badge is the Pod's text for the game, trimmed and cut; a blank one or a null is no badge. */
    @Test
    fun badgesAreTrimmedCutAndBlankOnesLeftOut() = runBlocking {
        val registry = registry(
            "p1",
            listOf(
                PodSlot.GameBadge { "  12/40  " },
                PodSlot.GameBadge { "x".repeat(MAX_BADGE_CHARS + 10) },
                PodSlot.GameBadge { "   " },
                PodSlot.GameBadge { null },
            ),
        )
        val texts = registry.gameBadges(game()).map { it.text }
        assertEquals(listOf("12/40", "x".repeat(MAX_BADGE_CHARS)), texts)
    }

    /** A badge that throws costs the Pod a crash and disappears; the others stay. Cancellation is not a crash. */
    @Test
    fun aFailingBadgeCountsAsACrashAndCancellationPasses() {
        val crashes = mutableListOf<String>()
        val registry = registry(
            "p1",
            listOf(PodSlot.GameBadge { error("boom") }, PodSlot.GameBadge { "ok" }),
            crashes = crashes,
        )
        assertEquals(listOf("ok"), runBlocking { registry.gameBadges(game()) }.map { it.text })
        assertEquals(listOf("p1"), crashes)
        val cancelling = registry("p2", listOf(PodSlot.GameBadge { throw CancellationException("stop") }), crashes = crashes)
        assertThrows(CancellationException::class.java) { runBlocking { cancelling.gameBadges(game()) } }
        assertEquals(listOf("p1"), crashes)
    }

    /** The badges follow the game they are asked for, and are limited in number. */
    @Test
    fun badgesFollowTheGameAndAreLimited() = runBlocking {
        val registry = registry("p1", (1..MAX_GAME_BADGES + 2).map { n -> PodSlot.GameBadge { g -> "$n/${g.id}" } })
        val badges = registry.gameBadges(game(id = 3L))
        assertEquals(MAX_GAME_BADGES, badges.size)
        assertEquals("1/3", badges.first().text)
    }

    /** The text of a Pod's notice is trimmed and cut at the limit; nothing left is no notice. */
    @Test
    fun noticeTextIsCut() {
        assertEquals("Level up", toastText("  Level up \n"))
        assertEquals(MAX_TOAST_CHARS, toastText("y".repeat(MAX_TOAST_CHARS * 2))?.length)
        assertNull(toastText("   "))
        assertNull(toastText(""))
    }
}
