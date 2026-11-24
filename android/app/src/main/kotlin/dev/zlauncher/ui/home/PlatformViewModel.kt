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

/** The games of one platform. */
class PlatformViewModel(application: Application) : AndroidViewModel(application) {
    /** The shared application object. */
    private val zl = application as ZLauncherApp

    /** Backing state: `null` until loaded. */
    private val state = MutableStateFlow<List<GameEntry>?>(null)

    /** The games, `null` while loading. */
    val games: StateFlow<List<GameEntry>?> = state.asStateFlow()

    /** Reads the games of [platformId]; a core error is reported through [onError]. */
    fun load(platformId: String, onError: (CoreException) -> Unit) {
        viewModelScope.launch {
            state.value = try {
                zl.core.call { it.games(platformId) }
            } catch (e: CoreException) {
                onError(e)
                emptyList()
            }
        }
    }
}
