package dev.zlauncher.ui.home

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import dev.zlauncher.R
import dev.zlauncher.ZLauncherApp
import dev.zlauncher.apps.AppEntry
import dev.zlauncher.core.nativecore.CoreException
import dev.zlauncher.core.nativecore.FolderKind
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

/** Default number of apps to populate in the dock when empty. */
private const val DEFAULT_DOCK_SIZE = 5

/** The string that names a Smart Folder of [kind]; `null` for a platform, which comes with its name. */
fun folderNameRes(kind: FolderKind): Int? = when (kind) {
    FolderKind.PLATFORM -> null
    FolderKind.FAVORITES -> R.string.folder_favorites
    FolderKind.RECENT -> R.string.folder_recent
    FolderKind.NEVER_PLAYED -> R.string.folder_never_played
    FolderKind.HIDDEN -> R.string.folder_hidden
}

/** Loads the Home from the core; every call runs off the main thread. */
class HomeViewModel(application: Application) : AndroidViewModel(application) {
    /** The shared application object. */
    private val zl = application as ZLauncherApp

    /** Backing state. */
    private val state = MutableStateFlow(HomeUi())

    /** The state the Home draws. */
    val ui: StateFlow<HomeUi> = state.asStateFlow()

    /** [entry] with its name in the app's language when it is a Smart Folder. */
    private fun named(entry: PlatformEntry): PlatformEntry =
        folderNameRes(entry.kind)?.let { entry.copy(name = getApplication<Application>().getString(it)) } ?: entry

    /** Reads the library mode, the platforms or games, and the dock again. */
    fun refresh() {
        viewModelScope.launch {
            val allApps = zl.apps.list()
            val apps = allApps.associateBy { it.packageName }
            state.value = try {
                zl.core.call { core ->
                    val mode = core.libraryMode()
                    val pinned = core.dock().mapNotNull { apps[it] }
                    val dockApps = if (pinned.isNotEmpty()) {
                        pinned
                    } else {
                        val recents = zl.recent.list().mapNotNull { apps[it] }
                        val others = allApps.filter { it.packageName != getApplication<Application>().packageName && it !in recents }
                        (recents + others).take(DEFAULT_DOCK_SIZE)
                    }
                    HomeUi(
                        loading = false,
                        mode = mode,
                        platforms = if (mode == LibraryMode.SMART) core.platforms().map(::named) else emptyList(),
                        games = if (mode == LibraryMode.SIMPLE) core.games(null) else emptyList(),
                        dock = dockApps,
                    )
                }
            } catch (e: CoreException) {
                HomeUi(loading = false, error = e)
            }
        }
    }
}
