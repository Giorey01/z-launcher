package dev.zlauncher.ui.home

import android.app.Application
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBarsPadding
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.grid.rememberLazyGridState
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import androidx.lifecycle.viewmodel.compose.viewModel
import dev.zlauncher.R
import dev.zlauncher.ZLauncherApp
import dev.zlauncher.core.nativecore.CoreException
import dev.zlauncher.core.nativecore.GameEntry
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.Screen
import dev.zlauncher.launch.toMessage
import dev.zlauncher.ui.pad.FocusRestorer
import dev.zlauncher.ui.theme.TextSecondary
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

/** Columns of the game grid inside a platform folder. */
private const val COLUMNS = 4

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

/** Focus key of a game tile. */
private fun key(id: Long) = "g:$id"

/** The folder of a platform (RF-27): its games in a grid; A launches a game, B goes back. */
@Composable
fun PlatformScreen(app: AppViewModel, screen: Screen.Platform, vm: PlatformViewModel = viewModel()) {
    val games by vm.games.collectAsState()
    val memory = app.memory("platform:${screen.id}")
    LaunchedEffect(screen.id, app.resumeTick) { vm.load(screen.id) { app.message = it.toMessage() } }
    val list = games.orEmpty()
    val gridState = rememberLazyGridState()
    Column(Modifier.fillMaxSize().systemBarsPadding().padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
        Text(screen.name, style = MaterialTheme.typography.headlineSmall)
        Text(stringResource(R.string.hint_platform), color = TextSecondary)
        LazyVerticalGrid(
            GridCells.Fixed(COLUMNS),
            Modifier.fillMaxSize(),
            gridState,
            horizontalArrangement = Arrangement.spacedBy(12.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            items(list, key = { it.id }) { game ->
                GameTile(game, false, { app.launchGame(game) }, Modifier.then(memory.track(key(game.id))))
            }
        }
    }
    FocusRestorer(memory, app.resumeTick, games != null, list.firstOrNull()?.let { key(it.id) }) { focused ->
        list.indexOfFirst { key(it.id) == focused }.takeIf { it >= 0 }?.let { gridState.scrollToItem(it) }
    }
}
