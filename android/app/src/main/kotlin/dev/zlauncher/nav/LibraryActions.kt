package dev.zlauncher.nav

import androidx.lifecycle.viewModelScope
import dev.zlauncher.R
import dev.zlauncher.core.nativecore.CoreException
import dev.zlauncher.core.nativecore.GameEntry
import dev.zlauncher.core.nativecore.ScanEvent
import dev.zlauncher.core.nativecore.ScanListener
import dev.zlauncher.launch.toMessage
import kotlinx.coroutines.launch

/** A scan listener that ignores the progress (filing an app rescans in the background). */
private object QuietScan : ScanListener {
    /** Progress is not shown. */
    override fun onEvent(event: ScanEvent) = Unit
}

/** What the user changes on games and apps from their menus; each change makes the screens read their lists again. */
class LibraryActions(private val app: AppViewModel) {
    /** Pins [packageName] to the dock or unpins it; the drawer reloads to show it. */
    fun togglePin(packageName: String) {
        app.overlay = null
        app.viewModelScope.launch {
            try {
                app.zl.core.call { it.toggleDock(packageName) }
                app.refresh()
            } catch (e: CoreException) {
                app.message = e.toMessage()
            }
        }
    }

    /**
     * Files the app [packageName] under the console [platformId] as a game titled [label] (a recomp or port),
     * then goes back to the drawer and says where it went.
     */
    fun fileApp(packageName: String, label: String, platformId: String, platformName: String) {
        app.viewModelScope.launch {
            app.message = try {
                app.zl.core.call { it.fileApp(packageName, label, platformId, QuietScan) }
                if (app.top is Screen.ConsolePicker) app.stack.removeLastOrNull()
                app.refresh()
                UiMessage(R.string.msg_app_filed, listOf(label, platformName))
            } catch (_: CoreException.Io) {
                UiMessage(R.string.msg_app_file_failed)
            } catch (_: CoreException.NotFound) {
                UiMessage(R.string.msg_app_no_folder)
            } catch (e: CoreException) {
                e.toMessage()
            }
        }
    }

    /** Marks [game] as a favourite or not; the lists reload. */
    fun setFavorite(game: GameEntry, favorite: Boolean) {
        app.overlay = null
        app.viewModelScope.launch {
            try {
                app.zl.core.call { it.setFavorite(game.id, favorite) }
                app.refresh()
            } catch (e: CoreException) {
                app.message = e.toMessage()
            }
        }
    }

    /** Hides [game] (it stays in the Hidden folder) or shows it again; the lists reload. */
    fun setHidden(game: GameEntry, hidden: Boolean) {
        app.overlay = null
        app.viewModelScope.launch {
            try {
                app.zl.core.call { it.setHidden(game.id, hidden) }
                if (hidden) app.message = UiMessage(R.string.msg_game_hidden, listOf(game.displayTitle))
                app.refresh()
            } catch (e: CoreException) {
                app.message = e.toMessage()
            }
        }
    }

    /** Takes [game], an app filed under a console, out of it again. */
    fun unfileApp(game: GameEntry) {
        app.overlay = null
        app.viewModelScope.launch {
            app.message = try {
                app.zl.core.call { it.unfileApp(game.id, QuietScan) }
                app.refresh()
                UiMessage(R.string.msg_app_unfiled, listOf(game.displayTitle))
            } catch (_: CoreException.Io) {
                UiMessage(R.string.msg_app_file_failed)
            } catch (e: CoreException) {
                e.toMessage()
            }
        }
    }

    /** Sets or clears the user rating for a game (0.0 to 5.0). */
    fun setRating(gameId: Long, rating: Float?, onDone: (() -> Unit)? = null) {
        app.viewModelScope.launch {
            try {
                app.zl.core.call { it.setGameRating(gameId, rating) }
                app.refresh()
                onDone?.invoke()
            } catch (e: CoreException) {
                app.message = e.toMessage()
            }
        }
    }

    /** Sets or clears a custom title override for a game. */
    fun setTitle(gameId: Long, title: String?, onDone: (() -> Unit)? = null) {
        app.viewModelScope.launch {
            try {
                app.zl.core.call { it.setGameTitle(gameId, title) }
                app.refresh()
                onDone?.invoke()
            } catch (e: CoreException) {
                app.message = e.toMessage()
            }
        }
    }
}
