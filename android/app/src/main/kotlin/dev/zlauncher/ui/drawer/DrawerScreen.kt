package dev.zlauncher.ui.drawer

import android.app.Application
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.grid.rememberLazyGridState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.ui.graphics.Color
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import androidx.lifecycle.viewmodel.compose.viewModel
import dev.zlauncher.R
import dev.zlauncher.ZLauncherApp
import dev.zlauncher.apps.AppEntry
import dev.zlauncher.core.nativecore.CoreException
import dev.zlauncher.launch.toMessage
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.ui.AppIcon
import dev.zlauncher.ui.common.BackPill
import dev.zlauncher.ui.common.HeaderTitle
import dev.zlauncher.ui.pad.FocusRestorer
import dev.zlauncher.ui.pad.PadTile
import dev.zlauncher.ui.pad.TileSurface
import dev.zlauncher.ui.theme.LocalZTheme
import dev.zlauncher.ui.theme.ZPanel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

/** Columns of the drawer grid. */
private const val COLUMNS = 5

/** Width of the app grid: five columns of the reference pitch. */
private val GRID_WIDTH = 430.dp

/** Size of the icon of an app tile. */
private val ICON_SIZE = 79.dp

/** What the drawer shows: the apps, the recent ones and which are pinned to the dock. */
data class DrawerUi(
    /** `false` once the apps are loaded. */
    val loading: Boolean = true,
    /** Installed apps by label. */
    val apps: List<AppEntry> = emptyList(),
    /** The apps launched last, newest first. */
    val recent: List<AppEntry> = emptyList(),
    /** Packages pinned to the dock. */
    val pinned: Set<String> = emptySet(),
)

/** Loads the installed apps and pins them to the dock. */
class DrawerViewModel(application: Application) : AndroidViewModel(application) {
    /** The shared application object. */
    private val zl = application as ZLauncherApp

    /** Backing state. */
    private val state = MutableStateFlow(DrawerUi())

    /** The drawer state. */
    val ui: StateFlow<DrawerUi> = state.asStateFlow()

    /** Reads the apps, the recent list and the dock. */
    fun load(onError: (CoreException) -> Unit) {
        viewModelScope.launch {
            val apps = zl.apps.list()
            val byPackage = apps.associateBy { it.packageName }
            val pinned = try {
                zl.core.call { it.dock() }.toSet()
            } catch (e: CoreException) {
                onError(e)
                emptySet()
            }
            state.value = DrawerUi(false, apps, zl.recent.list().mapNotNull { byPackage[it] }, pinned)
        }
    }

    /** Pins or unpins [packageName]; a full dock is reported through [onError]. */
    fun togglePin(packageName: String, onError: (CoreException) -> Unit) {
        viewModelScope.launch {
            try {
                val dock = zl.core.call { it.toggleDock(packageName) }
                state.value = state.value.copy(pinned = dock.toSet())
            } catch (e: CoreException) {
                onError(e)
            }
        }
    }
}

/** Focus key of an app tile (recent apps use a different prefix, they are the same apps again). */
private fun key(app: AppEntry, recent: Boolean = false) = "${if (recent) "r" else "a"}:${app.packageName}"

/**
 * The app drawer: a glass screen with the back pill and title, a row of recent apps, the apps in alphabetical
 * order in five columns and the A to Z index on the right. A launches, Y pins to the dock, B goes back.
 */
@Composable
fun DrawerScreen(app: AppViewModel, vm: DrawerViewModel = viewModel()) {
    val ui by vm.ui.collectAsState()
    val memory = app.memory("drawer")
    LaunchedEffect(app.resumeTick) { vm.load { app.message = it.toMessage() } }
    LaunchedEffect(Unit) { app.hero = dev.zlauncher.nav.HeroTarget.None }
    val gridState = rememberLazyGridState()
    var letter by remember { mutableStateOf<Char?>(null) }
    val searching = app.drawerSearching
    var query by remember { mutableStateOf("") }
    LaunchedEffect(searching) { if (!searching) query = "" }
    val displayApps = remember(ui.apps, query, searching) {
        if (!searching) ui.apps else ui.apps.filter { matchesSearch(it.label, query) }
    }
    // The search hides the recent row, so it is not part of the grid (or of the focus) while it is open.
    val shownRecent = if (searching) emptyList() else ui.recent
    Box(Modifier.fillMaxSize()) {
        ZPanel(Modifier.fillMaxSize(), RectangleShape, elevation = 0.dp) {
            Column(Modifier.fillMaxSize().padding(horizontal = 16.dp)) {
                DrawerHeader(
                    searching = searching,
                    query = query,
                    onBack = { if (searching) { app.drawerSearching = false; query = "" } else app.back() },
                    onToggleSearch = { app.drawerSearching = !app.drawerSearching }
                )
                DrawerBody(displayApps, ui.recent, searching, gridState, memory, app, vm, letter) { letter = it }
            }
        }
        if (searching) {
            GamepadKeyboard(
                query = query,
                onQueryChange = { query = it },
                onConfirm = { app.drawerSearching = false },
                onClose = { app.drawerSearching = false; query = "" },
                modifier = Modifier.align(Alignment.BottomCenter)
            )
        }
    }
    val first = (shownRecent.firstOrNull()?.let { key(it, true) } ?: displayApps.firstOrNull()?.let { key(it) })
    // Not while the keyboard is open: every letter changes the results and would pull the focus off the keyboard.
    FocusRestorer(memory, app.resumeTick, !ui.loading && !searching, first) { focused ->
        gridIndexOf(focused, shownRecent.map { key(it, true) }, displayApps.map { key(it) })?.let { gridState.scrollToItem(it) }
    }
}

/** The body layout containing the grid and the letter index. */
@Composable
private fun DrawerBody(
    apps: List<AppEntry>,
    recent: List<AppEntry>,
    searching: Boolean,
    gridState: androidx.compose.foundation.lazy.grid.LazyGridState,
    memory: dev.zlauncher.ui.pad.FocusMemory,
    app: AppViewModel,
    vm: DrawerViewModel,
    letter: Char?,
    onLetter: (Char?) -> Unit,
) {
    Row(Modifier.fillMaxSize().padding(bottom = if (searching) 245.dp else 0.dp)) {
        DrawerGrid(
            apps = apps,
            recent = if (searching) emptyList() else recent,
            gridState = gridState,
            memory = memory,
            app = app,
            vm = vm,
            onLetter = onLetter
        )
        Box(Modifier.weight(1f))
        if (!searching) LetterIndex(apps.map(::initialOf).distinct(), letter)
    }
}

/** Header row of the app drawer with back button, title or query, and search button. */
@Composable
private fun DrawerHeader(searching: Boolean, query: String, onBack: () -> Unit, onToggleSearch: () -> Unit) {
    Row(Modifier.fillMaxWidth().padding(top = 16.dp, bottom = 6.dp), verticalAlignment = Alignment.CenterVertically) {
        BackPill(onClick = onBack)
        val title = if (searching) query.ifEmpty { stringResource(R.string.menu_search) } else stringResource(R.string.drawer_title)
        HeaderTitle(title, Modifier.padding(start = 12.dp).weight(1f))
        SearchPill(onClick = onToggleSearch)
    }
}

/** The 5-column grid of recent and alphabetical installed apps. */
@Composable
private fun DrawerGrid(
    apps: List<AppEntry>,
    recent: List<AppEntry>,
    gridState: androidx.compose.foundation.lazy.grid.LazyGridState,
    memory: dev.zlauncher.ui.pad.FocusMemory,
    app: AppViewModel,
    vm: DrawerViewModel,
    onLetter: (Char?) -> Unit,
) {
    LazyVerticalGrid(GridCells.Fixed(COLUMNS), Modifier.padding(start = 28.dp).width(GRID_WIDTH).fillMaxHeight(), gridState) {
        items(recent, key = { key(it, true) }) { entry ->
            AppCell(
                entry,
                { app.launchApp(entry.packageName) },
                { vm.togglePin(entry.packageName) { app.message = it.toMessage() } },
                Modifier.then(memory.track(key(entry, true))),
                { onLetter(null) }
            )
        }
        if (recent.isNotEmpty()) item(span = { GridItemSpan(COLUMNS) }) { RecentDivider() }
        items(apps, key = { key(it) }) { entry ->
            AppCell(
                entry,
                { app.launchApp(entry.packageName) },
                { vm.togglePin(entry.packageName) { app.message = it.toMessage() } },
                Modifier.then(memory.track(key(entry))),
                { onLetter(initialOf(entry)) }
            )
        }
    }
}

/** The letter an app is filed under: its first letter in capitals, `#` for anything else. */
private fun initialOf(app: AppEntry): Char = app.label.firstOrNull()?.uppercaseChar()?.takeIf { it.isLetter() } ?: '#'

/** The separator between the recent apps and the rest. */
@Composable
private fun RecentDivider() {
    Box(
        Modifier
            .fillMaxWidth()
            .padding(vertical = 8.dp, horizontal = 14.dp)
            .height(3.dp)
            .background(LocalZTheme.current.textPrimary.copy(alpha = 0.12f), RoundedCornerShape(2.dp))
    )
}

/** One app: the icon with the cursor around it and the label below. */
@Composable
private fun AppCell(entry: AppEntry, onLaunch: () -> Unit, onPin: () -> Unit, modifier: Modifier, onFocused: () -> Unit) {
    val theme = LocalZTheme.current
    Column(Modifier.fillMaxWidth().padding(vertical = 6.dp), horizontalAlignment = Alignment.CenterHorizontally) {
        PadTile(
            onLaunch,
            modifier.size(ICON_SIZE),
            onSecondary = onPin,
            shape = RoundedCornerShape(22),
            surface = TileSurface.NONE,
            cursorOutset = 4.dp,
            onFocused = onFocused
        ) {
            AppIcon(entry.packageName, ICON_SIZE, Modifier.clip(RoundedCornerShape(22)))
        }
        Text(
            entry.label,
            Modifier.padding(top = 4.dp),
            color = theme.textPrimary.copy(alpha = 0.6f),
            fontSize = 13.sp,
            fontWeight = FontWeight.SemiBold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            textAlign = TextAlign.Center
        )
    }
}

/** The A to Z index: the letters that have apps, the one of the focused app brighter. */
@Composable
private fun LetterIndex(letters: List<Char>, current: Char?) {
    val theme = LocalZTheme.current
    Column(Modifier.fillMaxHeight().padding(start = 6.dp), verticalArrangement = Arrangement.SpaceEvenly, horizontalAlignment = Alignment.CenterHorizontally) {
        for (c in letters) {
            Text(c.toString(), color = theme.textPrimary.copy(alpha = if (c == current) 0.8f else 0.25f), fontSize = 10.sp, fontWeight = FontWeight.Bold)
        }
    }
}

/** Search pill in the drawer header matching Cocoon. */
@Composable
private fun SearchPill(modifier: Modifier = Modifier, onClick: () -> Unit = {}) {
    val theme = LocalZTheme.current
    Row(
        modifier
            .height(30.dp)
            .clip(CircleShape)
            .background(theme.accentStart)
            .clickable { onClick() }
            .padding(horizontal = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(6.dp)
    ) {
        Box(
            Modifier
                .size(20.dp)
                .background(Color.White.copy(alpha = 0.85f), CircleShape),
            contentAlignment = Alignment.Center
        ) {
            Icon(Icons.Rounded.PlayArrow, null, Modifier.size(13.dp), tint = theme.accentStart)
        }
        Icon(Icons.Rounded.Search, null, Modifier.size(17.dp), tint = Color.White)
    }
}
