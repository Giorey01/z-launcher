package dev.zlauncher.ui.home

import android.app.Application
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.background
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.Sort
import androidx.compose.material.icons.rounded.Tune
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.foundation.lazy.grid.rememberLazyGridState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import androidx.lifecycle.viewmodel.compose.viewModel
import dev.zlauncher.R
import dev.zlauncher.ZLauncherApp
import dev.zlauncher.core.nativecore.CoreException
import dev.zlauncher.core.nativecore.GameEntry
import dev.zlauncher.launch.toMessage
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.HeroTarget
import dev.zlauncher.nav.Screen
import dev.zlauncher.ui.common.BackPill
import dev.zlauncher.ui.common.ButtonChip
import dev.zlauncher.ui.common.FaintPill
import dev.zlauncher.ui.common.HeaderTitle
import dev.zlauncher.ui.pad.FocusRestorer
import dev.zlauncher.ui.theme.LocalZTheme
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

/**
 * The folder of a platform: the dock, a header (back pill, name, sort label) and the games in a grid with the
 * covers framed by the theme's overlay. A launches a game, Y opens its menu, LB and RB go to the previous and
 * next platform, B goes back.
 */
@Composable
fun PlatformScreen(
    app: AppViewModel,
    screen: Screen.Platform,
    vm: PlatformViewModel = viewModel(),
    home: HomeViewModel = viewModel()
) {
    val loaded by vm.games.collectAsState()
    val games = loaded?.takeIf { it.platformId == screen.id }?.games
    val homeUi by home.ui.collectAsState()
    val memory = app.memory("platform:${screen.id}")
    LaunchedEffect(screen.id, app.resumeTick) { vm.load(screen.id) { app.message = it.toMessage() } }
    LaunchedEffect(screen.id) { app.hero = HeroTarget.Platform(screen.id, screen.name) }
    val list = games.orEmpty()
    val gridState = rememberLazyGridState()
    val dockOnTop = LocalZTheme.current.settings.dockOnTop
    val dock: @Composable () -> Unit = {
        Dock(
            homeUi.dock,
            memory,
            { app.open(Screen.Drawer) },
            { app.launchApp(it.packageName) },
            { app.hero = HeroTarget.Platform(screen.id, screen.name) },
            dockOnTop
        )
    }
    val leads = app.zl.pods.platformGridLeadSlots(screen.id)
    val defaultKey = leads.firstOrNull()?.let { "lead:${it.podId}:${it.label}" } ?: list.firstOrNull()?.let { gameKey(it.id) }
    Column(Modifier.fillMaxSize()) {
        if (dockOnTop) dock()
        FolderHeader(screen.name, Modifier.padding(top = if (dockOnTop) 6.dp else 20.dp, bottom = 6.dp))
        Box(Modifier.weight(1f)) { GameGrid(app, list, gridState, memory, leads) }
        if (!dockOnTop) dock()
    }
    FocusRestorer(memory, app.resumeTick, games != null && app.overlay == null, defaultKey) { focused ->
        val leadIdx = leads.indexOfFirst { "lead:${it.podId}:${it.label}" == focused }
        if (leadIdx >= 0) {
            gridState.scrollToItem(leadIdx)
        } else {
            val gameIdx = list.indexOfFirst { gameKey(it.id) == focused }
            if (gameIdx >= 0) gridState.scrollToItem(leads.size + gameIdx)
        }
    }
}

/** The header of a folder: back hint, the folder name, and on the right the sort label with the bumper hint. */
@Composable
private fun FolderHeader(title: String, modifier: Modifier = Modifier) {
    val theme = LocalZTheme.current
    Row(modifier.fillMaxWidth().padding(horizontal = 8.dp), verticalAlignment = Alignment.CenterVertically) {
        if (theme.settings.cornerHints) FaintPill { ButtonChip("LB") }
        BackPill(Modifier.padding(start = 6.dp))
        HeaderTitle(title, Modifier.padding(start = 14.dp).weight(1f))
        FaintPill(Modifier.padding(end = 6.dp)) {
            Icon(Icons.AutoMirrored.Rounded.Sort, null, Modifier.size(20.dp), tint = theme.textPrimary.copy(alpha = 0.5f))
            Text(
                stringResource(R.string.sort_manual),
                color = theme.textPrimary.copy(alpha = 0.6f),
                fontSize = 15.sp,
                fontWeight = FontWeight.SemiBold,
                modifier = Modifier.padding(end = 6.dp)
            )
        }
        Box(
            Modifier
                .padding(end = 6.dp)
                .size(31.dp)
                .background(theme.textPrimary.copy(alpha = 0.10f), CircleShape),
            contentAlignment = Alignment.Center
        ) {
            Icon(Icons.Rounded.Tune, null, Modifier.size(17.dp), tint = theme.textPrimary.copy(alpha = 0.6f))
        }
        if (theme.settings.cornerHints) {
            ButtonChip("RB", Modifier.background(theme.textPrimary.copy(alpha = 0.10f), CircleShape), 31.dp)
        }
    }
}
