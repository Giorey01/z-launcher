package dev.zlauncher.display

import android.hardware.display.DisplayManager
import android.view.Display
import dev.zlauncher.core.nativecore.ExternalDisplayDocking
import dev.zlauncher.core.nativecore.ScreenMode
import dev.zlauncher.core.nativecore.UserSettings

/** What one display shows. */
enum class Role {
    /** The grid, the dock and the menus (the small screen of a dual-screen handheld). */
    GRID,

    /** The hero area and the settings list (the big screen). */
    HERO,

    /** Everything on one display: hero on top, grid below. */
    SINGLE,
}

/** Words in a display name that mark an external screen (HDMI, casting, virtual). */
private val EXTERNAL_NAME_HINTS = listOf("hdmi", "cast", "virtual", "overlay", "wifi")

/** Decides which display shows what, from the settings and the displays that are on. Pure logic over a list, easy to test. */
object DisplayRoles {
    /**
     * The displays the launcher may use: those that are on and, when [docking] is `NEVER_DOCK`, not external
     * screens (HDMI and casting), which then keep their own content.
     */
    fun candidates(manager: DisplayManager, docking: ExternalDisplayDocking): List<Display> =
        manager.displays.filter { it.state == Display.STATE_ON || it.state == Display.STATE_UNKNOWN }
            .filter { docking != ExternalDisplayDocking.NEVER_DOCK || !isExternal(it) }

    /** Whether [display] looks like an external screen by its name. */
    private fun isExternal(display: Display): Boolean = EXTERNAL_NAME_HINTS.any { display.name.contains(it, ignoreCase = true) }

    /** Number of pixels of a display (used to tell the small screen from the big one). */
    private fun area(display: Display): Long = display.mode.let { it.physicalWidth.toLong() * it.physicalHeight }

    /**
     * The role of the display [currentId]: in the dual mode with two or more displays the smaller one is the grid
     * and the larger the hero (swapped by `reverse_display_roles`), otherwise one display does everything.
     */
    fun resolve(settings: UserSettings, candidates: List<Display>, currentId: Int): Role {
        if (settings.screenMode != ScreenMode.DUAL || candidates.size < 2) return Role.SINGLE
        val sorted = candidates.sortedBy(::area)
        val grid = if (settings.reverseDisplayRoles) sorted.last() else sorted.first()
        return if (grid.displayId == currentId) Role.GRID else Role.HERO
    }

    /** The display that is not [currentId] and is the best companion, or `null` when there is no other one. */
    fun otherDisplay(candidates: List<Display>, currentId: Int): Display? = candidates.firstOrNull { it.displayId != currentId }
}
