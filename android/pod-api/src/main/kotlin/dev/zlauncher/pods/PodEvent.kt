package dev.zlauncher.pods

import dev.zlauncher.core.nativecore.GameEntry
import dev.zlauncher.ui.theme.ZTheme

/**
 * Events dispatched by the launcher host to active Pods (§8).
 * Note: [GameStarted], [GameEnded], and [LibraryChanged] require the `library.read` permission.
 */
sealed interface PodEvent {
    /** A game has started running. */
    data class GameStarted(
        /** The game that started running. */
        val game: GameEntry,
    ) : PodEvent

    /** A game session has concluded after [seconds] of active gameplay. */
    data class GameEnded(
        /** The game that ended. */
        val game: GameEntry,
        /** Active session duration in seconds. */
        val seconds: Long,
    ) : PodEvent

    /** Games or folders have been rescanned, added, or modified for [platformIds]. */
    data class LibraryChanged(
        /** Identifiers of affected platforms. */
        val platformIds: Set<String>,
    ) : PodEvent

    /** The active theme or visual settings have changed. */
    data class ThemeChanged(
        /** The newly applied visual theme. */
        val theme: ZTheme,
    ) : PodEvent
}
