package dev.zlauncher.pods

import android.content.Context
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.vector.ImageVector
import dev.zlauncher.core.nativecore.GameDetails
import dev.zlauncher.core.nativecore.GameEntry
import dev.zlauncher.core.nativecore.PlatformEntry
import dev.zlauncher.core.nativecore.PlayStats
import dev.zlauncher.core.nativecore.SoundAction
import dev.zlauncher.ui.theme.ZTheme

/**
 * A Pod (RF-73): a screen a build adds to the launcher, opened from the Start menu, that uses the launcher's theme,
 * sounds and library. This repository defines the contract and ships no Pod; a build that has some lists their
 * [PodProvider] in `META-INF/services/dev.zlauncher.pods.PodProvider`, so the dependency always goes from the Pod to
 * the launcher. See `docs/spec/pods.md`.
 */
interface Pod {
    /** Stable id: lower-case letters, digits, `.`, `-` and `_`, unique among the Pods of a build. */
    val id: String

    /** Icon of the Start menu entry. */
    val icon: ImageVector

    /** Name of the Start menu entry and of the screen, in the app's language. */
    fun title(context: Context): String

    /** The Pod's screen, drawn inside the launcher's theme (`LocalZTheme`); B goes back unless the Pod handles it. */
    @Composable
    fun Content(host: PodHost)
}

/** Gives the Pods of a build; implementations are found with `ServiceLoader` and need a public no-argument constructor. */
interface PodProvider {
    /** The Pods, in the order the Start menu lists them. */
    fun pods(): List<Pod>
}

/** What the launcher offers a Pod. Library calls run off the main thread and may throw a `CoreException`. */
interface PodHost {
    /** The active theme: colours, settings, wallpapers. */
    val theme: ZTheme

    /** Plays the sound of [action] the way the launcher does (nothing when effects are off). */
    fun play(action: SoundAction)

    /** The folders of the Home: Smart Folders and platforms, with their counts. */
    suspend fun folders(): List<PlatformEntry>

    /** The games of a folder id from [folders], or all of them with `null`. */
    suspend fun games(folder: String?): List<GameEntry>

    /** Everything known about one game. */
    suspend fun details(gameId: Long): GameDetails

    /** The play time of one game. */
    suspend fun playStats(gameId: Long): PlayStats

    /** Starts [game] the way the launcher does (emulator choice, display, play time). */
    fun launch(game: GameEntry)

    /** Closes the Pod's screen. */
    fun close()
}
