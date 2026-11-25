package dev.zlauncher.ui.home

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import dev.zlauncher.ZLauncherApp
import dev.zlauncher.core.nativecore.CoreException
import dev.zlauncher.core.nativecore.GameEntry
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

/** The games of [platformId], as last read from the core. */
data class PlatformGames(
    /** The platform the list belongs to. */
    val platformId: String,
    /** Its games. */
    val games: List<GameEntry>,
)

/** The games of one platform. */
class PlatformViewModel(application: Application) : AndroidViewModel(application) {
    /** The shared application object. */
    private val zl = application as ZLauncherApp

    /** Backing state: `null` until loaded. */
    private val state = MutableStateFlow<PlatformGames?>(null)

    /**
     * The games last loaded, with their platform: the view model is shared by every folder, so a screen must
     * check the platform before using the list (it still holds the previous folder while the next one loads).
     */
    val games: StateFlow<PlatformGames?> = state.asStateFlow()

    /** Reads the games of [platformId]; a core error is reported through [onError]. */
    fun load(platformId: String, onError: (CoreException) -> Unit) {
        viewModelScope.launch {
            val games = try {
                zl.core.call { it.games(platformId) }
            } catch (e: CoreException) {
                onError(e)
                emptyList()
            }
            state.value = PlatformGames(platformId, games)
        }
    }
}
