package dev.zlauncher.ui.home

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyGridState
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.grid.rememberLazyGridState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import dev.zlauncher.R
import dev.zlauncher.core.nativecore.GameEntry
import dev.zlauncher.core.nativecore.LibraryMode
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.HeroTarget
import dev.zlauncher.nav.Overlay
import dev.zlauncher.nav.Screen
import dev.zlauncher.ui.pad.FocusMemory
import dev.zlauncher.ui.pad.FocusRestorer
import dev.zlauncher.ui.pad.PadButton
import dev.zlauncher.ui.theme.LocalZTheme

/** Id of the Home in the focus memories. */
private const val HOME_ID = "home"

/** Columns of the platform grid. */
private const val PLATFORM_COLUMNS = 3

/** Focus key of a platform tile. */
private fun platformKey(id: String) = "p:$id"

/** Focus key of a game tile. */
fun gameKey(id: Long) = "g:$id"

/** Finds the default focus key for the library state. */
private fun initialHomeKey(ui: HomeUi): String? = when (ui.mode) {
    LibraryMode.SIMPLE -> ui.games.firstOrNull()?.let { gameKey(it.id) }
    else -> ui.platforms.firstOrNull()?.let { platformKey(it.id) }
}

/**
 * The Home: the dock on top and the library below, as one folder icon per platform (Smart Folders, three
 * columns, no labels) or as all the games (Simple). A opens or launches, Y on a game opens its menu,
 * Start opens the Start menu. What is focused is shown in the hero area of the other display.
 */
@Composable
fun HomeScreen(app: AppViewModel, vm: HomeViewModel = viewModel()) {
    val ui by vm.ui.collectAsState()
    val memory = app.memory(HOME_ID)
    LaunchedEffect(app.resumeTick) { vm.refresh() }
    LaunchedEffect(ui.platforms) { app.platforms = ui.platforms.map { it.id to it.name } }
    val firstKey = initialHomeKey(ui)
    val gridState = rememberLazyGridState()
    val dockOnTop = LocalZTheme.current.settings.dockOnTop
    val gridKeys = if (ui.mode == LibraryMode.SIMPLE) ui.games.map { gameKey(it.id) } else ui.platforms.map { platformKey(it.id) }
    val dock: @Composable () -> Unit = {
        Dock(
            ui.dock,
            memory,
            { app.open(Screen.Drawer) },
            { app.launchApp(it.packageName) },
            { app.hero = HeroTarget.None },
            dockOnTop,
            // From the dock into the grid: the folder left last if it is on screen, else the first one
            // shown. Left to the geometric search, the focus got lost on the way.
            toGrid = { gridEntry(memory, gridKeys, gridState) },
        )
    }
    Column(Modifier.fillMaxSize()) {
        if (dockOnTop) dock()
        HomeContent(ui, app, firstKey, gridState, memory, Modifier.weight(1f))
        if (!dockOnTop) dock()
    }
    val target = firstKey ?: DOCK_ALL_APPS_KEY
    FocusRestorer(memory, app.resumeTick, !ui.loading && app.overlay == null, target) { key ->
        val keys = if (ui.mode == LibraryMode.SIMPLE) ui.games.map { gameKey(it.id) } else ui.platforms.map { platformKey(it.id) }
        keys.indexOf(key).takeIf { it >= 0 }?.let { gridState.scrollToItem(it) }
    }
}

/** The grid element the dock leads to: the one focused last if it is on screen, else the first on screen. */
private fun gridEntry(memory: FocusMemory, keys: List<String>, state: LazyGridState): FocusRequester {
    val visible = state.layoutInfo.visibleItemsInfo.map { it.index }.filter { it in keys.indices }.map { keys[it] }
    val key = memory.lastWhere { it in visible } ?: visible.firstOrNull() ?: return FocusRequester.Default
    return memory.requesterOf(key)
}

/** The body of the Home: loading spinner, empty library prompt, or game/platform grid. */
@Composable
private fun HomeContent(
    ui: HomeUi,
    app: AppViewModel,
    firstKey: String?,
    gridState: LazyGridState,
    memory: FocusMemory,
    modifier: Modifier = Modifier
) {
    Box(modifier) {
        when {
            ui.loading -> Box(Modifier.fillMaxSize())
            firstKey == null -> EmptyLibrary { app.open(Screen.Settings) }
            ui.mode == LibraryMode.SIMPLE -> SimpleGames(app, ui.games, gridState, memory)
            else -> PlatformGrid(app, ui.platforms, gridState, memory)
        }
    }
}

/** The platform folders in three full-width columns, each cell as tall as it is wide. */
@Composable
private fun PlatformGrid(
    app: AppViewModel,
    platforms: List<dev.zlauncher.core.nativecore.PlatformEntry>,
    state: LazyGridState,
    memory: FocusMemory
) {
    val spec = LocalGridSpec.current
    val tile: @Composable (dev.zlauncher.core.nativecore.PlatformEntry, Modifier) -> Unit = { platform, modifier ->
        PlatformTile(
            platform,
            {
                app.play(dev.zlauncher.core.nativecore.SoundAction.FOLDER_OPEN)
                app.open(Screen.Platform(platform.id, platform.name))
            },
            modifier.then(memory.track(platformKey(platform.id))),
            onFocused = { app.hero = HeroTarget.Platform(platform.id, platform.name) },
        )
    }
    if (spec.rows == 1) {
        OneRow(platforms, spec, Modifier.padding(top = 4.dp), tile)
        return
    }
    val columns = if (spec.rows == 0) PLATFORM_COLUMNS else spec.platformColumns
    val topPadding = if (spec.rows == 0) 29.dp else 4.dp
    LazyVerticalGrid(
        GridCells.Fixed(columns),
        Modifier.fillMaxSize(),
        state,
        contentPadding = PaddingValues(top = topPadding)
    ) {
        items(platforms, key = { it.id }) { platform -> tile(platform, Modifier) }
    }
}

/** Simple library mode: all games in the folder-style grid, without a header. */
@Composable
private fun SimpleGames(app: AppViewModel, games: List<GameEntry>, state: LazyGridState, memory: FocusMemory) {
    Column(Modifier.fillMaxSize().padding(top = 36.dp)) {
        GameGrid(app, games, state, memory)
    }
}

/** What the Home shows when the library has no games: a hint and a button to the settings. */
@Composable
private fun EmptyLibrary(onSettings: () -> Unit) {
    val theme = LocalZTheme.current
    Column(Modifier.fillMaxSize(), Arrangement.Center, Alignment.CenterHorizontally) {
        Text(stringResource(R.string.home_empty_title), color = theme.textPrimary, style = androidx.compose.material3.MaterialTheme.typography.titleLarge)
        Text(stringResource(R.string.home_empty_hint), color = theme.textSecondary, modifier = Modifier.padding(vertical = 12.dp).fillMaxWidth(0.8f))
        PadButton(stringResource(R.string.home_open_settings), onSettings)
        Box(Modifier.height(1.dp))
    }
}

/** Opens the Y menu of [game], remembering the tile it came from. */
fun openGameMenu(app: AppViewModel, game: GameEntry) {
    app.overlay = Overlay.GameMenu(game, gameKey(game.id))
}
