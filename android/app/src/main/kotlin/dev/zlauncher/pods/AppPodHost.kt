package dev.zlauncher.pods

import dev.zlauncher.core.nativecore.GameDetails
import dev.zlauncher.core.nativecore.GameEntry
import dev.zlauncher.core.nativecore.PlatformEntry
import dev.zlauncher.core.nativecore.PlayStats
import dev.zlauncher.core.nativecore.SoundAction
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.ui.theme.ZTheme

/** The launcher's side of [PodHost], over the app's state; [theme] is the one the screen is drawn with. */
class AppPodHost(private val app: AppViewModel, override val theme: ZTheme) : PodHost {
    /** Same sounds as the launcher's own screens. */
    override fun play(action: SoundAction) = app.play(action)

    /** The Home's folders. */
    override suspend fun folders(): List<PlatformEntry> = app.zl.core.call { it.platforms() }

    /** The games of a folder. */
    override suspend fun games(folder: String?): List<GameEntry> = app.zl.core.call { it.games(folder) }

    /** One game's details. */
    override suspend fun details(gameId: Long): GameDetails = app.zl.core.call { it.gameDetails(gameId) }

    /** One game's play time. */
    override suspend fun playStats(gameId: Long): PlayStats = app.zl.core.call { it.playStats(gameId) }

    /** The launcher's own launch. */
    override fun launch(game: GameEntry) = app.launchGame(game)

    /** Back to where the Pod was opened from. */
    override fun close() {
        app.back()
    }
}
