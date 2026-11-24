package dev.zlauncher.ui.home

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.grid.rememberLazyGridState
import androidx.compose.foundation.layout.systemBarsPadding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import dev.zlauncher.R
import dev.zlauncher.core.nativecore.LibraryMode
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.Screen
import dev.zlauncher.ui.pad.FocusRestorer
import dev.zlauncher.ui.pad.PadButton
import dev.zlauncher.ui.theme.TextSecondary

/** Id of the Home in the focus memories. */
private const val HOME_ID = "home"

/** Columns of the platform grid. */
private const val PLATFORM_COLUMNS = 3

/** Columns of the game grid of the Simple mode. */
private const val GAME_COLUMNS = 4

/** Focus key of a platform tile. */
private fun platformKey(id: String) = "p:$id"

/** Focus key of a game tile. */
private fun gameKey(id: Long) = "g:$id"

/**
 * The Home (RF-27, RF-35): the dock on top and the library below, as one tile per platform (Smart
 * Folders) or as all games (Simple). A opens or launches, Start opens the app drawer.
 */
@Composable
fun HomeScreen(app: AppViewModel, vm: HomeViewModel = viewModel()) {
    val ui by vm.ui.collectAsState()
    val memory = app.memory(HOME_ID)
    LaunchedEffect(app.resumeTick) { vm.refresh() }
    val firstKey = when {
        ui.mode == LibraryMode.SIMPLE -> ui.games.firstOrNull()?.let { gameKey(it.id) }
        else -> ui.platforms.firstOrNull()?.let { platformKey(it.id) }
    }
    val gridState = rememberLazyGridState()
    Column(Modifier.fillMaxSize().systemBarsPadding().padding(16.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
        Dock(ui.dock, memory, { app.open(Screen.Drawer) }, { app.launchApp(it.packageName) }, app::reopenSetup)
        when {
            ui.loading -> Box(Modifier.fillMaxSize())
            firstKey == null -> EmptyLibrary(app::reopenSetup)
            ui.mode == LibraryMode.SIMPLE -> LibraryGrid(GAME_COLUMNS, gridState, ui.games, { gameKey(it.id) }) { game, modifier ->
                GameTile(game, true, { app.launchGame(game) }, modifier.then(memory.track(gameKey(game.id))))
            }
            else -> LibraryGrid(PLATFORM_COLUMNS, gridState, ui.platforms, { platformKey(it.id) }) { platform, modifier ->
                PlatformTile(platform, { app.open(Screen.Platform(platform.id, platform.name)) }, modifier.then(memory.track(platformKey(platform.id))))
            }
        }
    }
    val target = firstKey ?: DOCK_ALL_APPS_KEY
    FocusRestorer(memory, app.resumeTick, !ui.loading, target) { key ->
        val keys = if (ui.mode == LibraryMode.SIMPLE) ui.games.map { gameKey(it.id) } else ui.platforms.map { platformKey(it.id) }
        keys.indexOf(key).takeIf { it >= 0 }?.let { gridState.scrollToItem(it) }
    }
}

/** The grid of the library: [items] in [columns] columns, each drawn by [tile] with the grid cell modifier. */
@Composable
private fun <T> LibraryGrid(
    columns: Int,
    state: androidx.compose.foundation.lazy.grid.LazyGridState,
    items: List<T>,
    key: (T) -> String,
    tile: @Composable (T, Modifier) -> Unit,
) {
    LazyVerticalGrid(
        GridCells.Fixed(columns),
        Modifier.fillMaxSize(),
        state,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        items(items, key = key) { item -> tile(item, Modifier) }
    }
}

/** What the Home shows when the library has no games: a hint and a button that runs the setup again. */
@Composable
private fun EmptyLibrary(onSetup: () -> Unit) {
    Column(Modifier.fillMaxSize(), Arrangement.Center, Alignment.CenterHorizontally) {
        Text(stringResource(R.string.home_empty_title), style = androidx.compose.material3.MaterialTheme.typography.titleLarge)
        Text(stringResource(R.string.home_empty_hint), color = TextSecondary, modifier = Modifier.padding(vertical = 12.dp))
        PadButton(stringResource(R.string.home_open_setup), onSetup)
    }
}
