package dev.zlauncher.ui.home

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import dev.zlauncher.ZLauncherApp
import dev.zlauncher.apps.AppEntry
import dev.zlauncher.core.nativecore.CoreException
import dev.zlauncher.core.nativecore.GameEntry
import dev.zlauncher.core.nativecore.LibraryMode
import dev.zlauncher.core.nativecore.PlatformEntry
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

/** What the Home shows. */
data class HomeUi(
    /** The first load is still running. */
    val loading: Boolean = true,
    /** Smart Folders or Simple. */
    val mode: LibraryMode = LibraryMode.SMART,
    /** Platforms with games (Smart Folders). */
    val platforms: List<PlatformEntry> = emptyList(),
    /** All games (Simple mode). */
    val games: List<GameEntry> = emptyList(),
    /** Apps pinned to the dock, in order. */
    val dock: List<AppEntry> = emptyList(),
    /** The core failed with this error (the library could not be read). */
    val error: CoreException? = null,
)

/** Loads the Home from the core; every call runs off the main thread. */
class HomeViewModel(application: Application) : AndroidViewModel(application) {
    /** The shared application object. */
    private val zl = application as ZLauncherApp

    /** Backing state. */
    private val state = MutableStateFlow(HomeUi())

    /** The state the Home draws. */
    val ui: StateFlow<HomeUi> = state.asStateFlow()

    /** Reads the library mode, the platforms or games, and the dock again. */
    fun refresh() {
        viewModelScope.launch {
            val apps = zl.apps.list().associateBy { it.packageName }
            state.value = try {
                zl.core.call { core ->
                    val mode = core.libraryMode()
                    HomeUi(
                        loading = false,
                        mode = mode,
                        platforms = if (mode == LibraryMode.SMART) core.platforms() else emptyList(),
                        games = if (mode == LibraryMode.SIMPLE) core.games(null) else emptyList(),
                        dock = core.dock().mapNotNull { apps[it] },
                    )
                }
            } catch (e: CoreException) {
                HomeUi(loading = false, error = e)
            }
        }
    }
}
