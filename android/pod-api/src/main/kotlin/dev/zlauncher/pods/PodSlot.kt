package dev.zlauncher.pods

import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.unit.Dp
import dev.zlauncher.core.nativecore.GameEntry

/**
 * A UI slot declared by a Pod (§7).
 * Each slot provides structured data or composable content that the host renders
 * inside host surfaces matching launcher styles, focus cursor and navigation sounds.
 */
sealed interface PodSlot {
    /** The manifest slot name required for this slot to be valid and rendered. */
    val manifestSlotName: String

    /** An entry in the launcher's Start menu. */
    data class StartMenu(
        /** Display label for the Start menu row. */
        val label: String,
        /** Icon displayed alongside the label. */
        val icon: ImageVector,
        /** Target route opened when selecting this entry. */
        val route: PodRoute = PodRoute("main"),
    ) : PodSlot {
        override val manifestSlotName: String get() = "start_menu"
    }

    /** The first tile of a platform game grid (the '+' or action tile, up to MAX_LEAD_TILES = 3). */
    data class PlatformGridLead(
        /** Target platform identifiers; `null` or empty sets match every platform. */
        val platforms: Set<String>? = null,
        /** Display label on the lead tile. */
        val label: String,
        /** Leading icon on the tile. */
        val icon: ImageVector,
        /** Computes the destination route for the selected platform. */
        val routeForPlatform: (platformId: String) -> PodRoute = { PodRoute("main", mapOf("platform" to it)) },
    ) : PodSlot {
        override val manifestSlotName: String get() = "platform_grid_lead"
    }

    /** Action button in the game's menu or info sheet. */
    data class GameActions(
        /** Label on the action button. */
        val label: String,
        /** Icon on the action button. */
        val icon: ImageVector,
        /** Predicate checking whether the action applies to [game]. */
        val condition: (game: GameEntry) -> Boolean = { true },
        /** Computes the destination route for [game]. */
        val routeForGame: (game: GameEntry) -> PodRoute = { PodRoute("game", mapOf("gameId" to it.id.toString())) },
    ) : PodSlot {
        override val manifestSlotName: String get() = "game_actions"
    }

    /** A dedicated row or section in the launcher settings screen. */
    data class SettingsSection(
        /** Title of the settings section. */
        val label: String,
        /** Route to open when selected. */
        val route: PodRoute = PodRoute("settings"),
    ) : PodSlot {
        override val manifestSlotName: String get() = "settings_section"
    }

    /** A compact widget embedded into the Home bar (such as a media mini-player). */
    data class StatusWidget(
        /** Maximum width the widget may occupy. */
        val maxWidth: Dp,
        /** Composable widget body. */
        val content: @Composable (host: PodHost) -> Unit,
    ) : PodSlot {
        override val manifestSlotName: String get() = "status_widget"
    }

    /** A shortcut tile on the Home screen. */
    data class HomeShortcut(
        /** Label of the shortcut. */
        val label: String,
        /** Icon of the shortcut. */
        val icon: ImageVector,
        /** Route to open when selected. */
        val route: PodRoute = PodRoute("shortcut"),
    ) : PodSlot {
        override val manifestSlotName: String get() = "home_shortcut"
    }

    /**
     * A tab in the game's sheet (RF-78, API 3): the launcher draws the tab row and the frame, the Pod draws [content]
     * inside it with the launcher's components. Tabs show after the launcher's own (Info, Activity), in the order the
     * Pods were installed, at most `MAX_GAME_TABS` per game.
     */
    data class GameTab(
        /** Title on the tab. */
        val label: String,
        /** Predicate checking whether the tab applies to the game; a tab whose condition fails or throws is left out. */
        val condition: (game: GameEntry) -> Boolean = { true },
        /** The tab's content for the game; the focus of the pad moves into it from the tab row. */
        val content: @Composable (host: PodHost, game: GameEntry) -> Unit,
    ) : PodSlot {
        override val manifestSlotName: String get() = "game_tab"
    }

    /**
     * A short text the Pod puts on the game's sheet (RF-79, API 3), as "12/40" for the achievements won. The launcher
     * draws it as a badge; [text] returns `null` or a blank text for no badge, and the host cuts it at
     * `MAX_BADGE_CHARS`. It may suspend (the Pod reads its own data); a failure counts as a crash of the Pod and
     * the badge is left out.
     */
    data class GameBadge(
        /** The text for the game, or `null` when the Pod has nothing to say about it. */
        val text: suspend (game: GameEntry) -> String?,
    ) : PodSlot {
        override val manifestSlotName: String get() = "game_badge"
    }
}
