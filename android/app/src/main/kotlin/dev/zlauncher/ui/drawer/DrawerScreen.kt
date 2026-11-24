package dev.zlauncher.ui.drawer

import android.app.Application
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.aspectRatio
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
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
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
import dev.zlauncher.ui.pad.FocusRestorer
import dev.zlauncher.ui.pad.PadTile
import dev.zlauncher.ui.theme.TextPrimary
import dev.zlauncher.ui.theme.TextSecondary
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

/** Columns of the drawer grid. */
private const val COLUMNS = 5

/** Size of the icon of an app tile. */
private val ICON_SIZE = 48.dp

/** What the drawer shows: the apps and which are pinned to the dock. */
data class DrawerUi(
    /** `false` once the apps are loaded. */
    val loading: Boolean = true,
    /** Installed apps by label. */
    val apps: List<AppEntry> = emptyList(),
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

    /** Reads the apps and the dock. */
    fun load(onError: (CoreException) -> Unit) {
        viewModelScope.launch {
            val apps = zl.apps.list()
            val pinned = try {
                zl.core.call { it.dock() }.toSet()
            } catch (e: CoreException) {
                onError(e)
                emptySet()
            }
            state.value = DrawerUi(false, apps, pinned)
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

/** Focus key of an app tile. */
private fun key(app: AppEntry) = "a:${app.packageName}"

/** The app drawer (RF-27): alphabetical grid of installed apps. A launches, Y pins to the dock, B goes back. */
@Composable
fun DrawerScreen(app: AppViewModel, vm: DrawerViewModel = viewModel()) {
    val ui by vm.ui.collectAsState()
    val memory = app.memory("drawer")
    LaunchedEffect(app.resumeTick) { vm.load { app.message = it.toMessage() } }
    val gridState = rememberLazyGridState()
    Column(Modifier.fillMaxSize().systemBarsPadding().padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
        Text(stringResource(R.string.drawer_title), style = MaterialTheme.typography.headlineSmall)
        Text(stringResource(R.string.hint_drawer), color = TextSecondary)
        LazyVerticalGrid(
            GridCells.Fixed(COLUMNS),
            Modifier.fillMaxSize(),
            gridState,
            horizontalArrangement = Arrangement.spacedBy(10.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            items(ui.apps, key = { it.packageName }) { entry ->
                val pin = { vm.togglePin(entry.packageName) { app.message = it.toMessage() } }
                AppTile(entry, entry.packageName in ui.pinned, { app.launchApp(entry.packageName) }, pin, Modifier.then(memory.track(key(entry))))
            }
        }
    }
    FocusRestorer(memory, app.resumeTick, !ui.loading, ui.apps.firstOrNull()?.let(::key)) { focused ->
        ui.apps.indexOfFirst { key(it) == focused }.takeIf { it >= 0 }?.let { gridState.scrollToItem(it) }
    }
}

/** One app: icon and label, with a star when pinned. */
@Composable
private fun AppTile(entry: AppEntry, pinned: Boolean, onLaunch: () -> Unit, onPin: () -> Unit, modifier: Modifier) {
    PadTile(onLaunch, modifier.aspectRatio(1f), onPin) {
        Column(
            Modifier.align(Alignment.Center).padding(6.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(4.dp),
        ) {
            AppIcon(entry.packageName, ICON_SIZE)
            Text(entry.label, color = TextPrimary, fontSize = 12.sp, maxLines = 2, overflow = TextOverflow.Ellipsis, textAlign = TextAlign.Center)
        }
        if (pinned) Text("★", color = TextPrimary, modifier = Modifier.align(Alignment.TopEnd).padding(6.dp))
    }
}
