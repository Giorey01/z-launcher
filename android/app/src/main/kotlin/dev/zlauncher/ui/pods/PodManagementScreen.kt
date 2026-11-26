package dev.zlauncher.ui.pods

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
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
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Add
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.zlauncher.R
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.pods.PodManager
import dev.zlauncher.pods.PodTab
import dev.zlauncher.ui.common.BackPill
import dev.zlauncher.ui.common.HeaderTitle
import dev.zlauncher.ui.pad.PadTile
import dev.zlauncher.ui.pad.TileSurface
import dev.zlauncher.ui.theme.LocalBackdrop
import dev.zlauncher.ui.theme.LocalZTheme

/** Height of the faint band behind the header, as in the settings. */
private val HEADER_BAND = 50.dp

/** Width of the details panel beside the list in the single-screen mode. */
private val DETAILS_WIDTH = 420.dp

/** MIME types offered by the file picker for a `.zlpod` (a zip the providers rarely name). */
private val ZLPOD_TYPES = arrayOf("application/octet-stream", "application/zip", "application/x-zip-compressed")

/**
 * The Pod screen (§11): header, tabs and the list on the grid display. The details of the row under
 * the cursor go on the big display in the dual-screen mode ([PodAside]) and beside the list in the
 * single-screen mode.
 */
@Composable
fun PodManagementScreen(app: AppViewModel, single: Boolean) {
    val theme = LocalZTheme.current
    val manager = app.podManager
    LaunchedEffect(Unit) { manager.reload() }
    FilePickerEffect(manager)
    CompositionLocalProvider(LocalBackdrop provides null) {
        Box(Modifier.fillMaxSize().background(theme.backgroundBrush())) {
            val band = Brush.verticalGradient(listOf(theme.textPrimary.copy(alpha = 0.16f), Color.Transparent))
            Box(Modifier.fillMaxWidth().height(HEADER_BAND).background(band))
            Column(Modifier.fillMaxSize().padding(start = 16.dp, end = 16.dp, top = 8.dp)) {
                PodHeader(app, manager)
                PodTabs(manager, Modifier.padding(top = 14.dp, bottom = 12.dp))
                Row(Modifier.fillMaxSize(), horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                    PodList(app, manager, Modifier.weight(1f).fillMaxHeight())
                    if (single) PodDetails(app, manager.state.focus, Modifier.width(DETAILS_WIDTH).fillMaxHeight().padding(bottom = 16.dp))
                }
            }
            PodDialogs(app, manager)
        }
    }
}

/** Opens the system file picker when the manager asks for it and hands the chosen file back. */
@Composable
private fun FilePickerEffect(manager: PodManager) {
    val picker = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri -> uri?.let(manager::inspectUri) }
    LaunchedEffect(manager.state.wantFile) {
        if (manager.state.wantFile) {
            manager.fileRequested()
            picker.launch(ZLPOD_TYPES)
        }
    }
}

/** "B ←", the title and the **+** (store) on the right. */
@Composable
private fun PodHeader(app: AppViewModel, manager: PodManager) {
    val theme = LocalZTheme.current
    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
        BackPill(onClick = { app.back() })
        HeaderTitle(stringResource(R.string.pod_screen_title), Modifier.padding(start = 12.dp).weight(1f))
        if (manager.state.inspecting) {
            Text(stringResource(R.string.pod_inspecting), color = theme.textSecondary, fontSize = 13.sp, modifier = Modifier.padding(end = 12.dp))
        }
        PadTile({ manager.selectTab(PodTab.STORE) }, Modifier.size(40.dp), shape = RoundedCornerShape(50)) {
            Icon(Icons.Rounded.Add, null, Modifier.align(Alignment.Center).size(24.dp), tint = theme.accentStart)
        }
    }
}

/** The tabs; Built-in only when this build has built-in Pods. L1 and R1 step through them. */
@Composable
private fun PodTabs(manager: PodManager, modifier: Modifier) {
    Row(modifier, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        for (tab in manager.tabs()) {
            val label = when (tab) {
                PodTab.INSTALLED -> stringResource(R.string.pod_tab_installed)
                PodTab.BUILTIN -> stringResource(R.string.pod_tab_builtin)
                PodTab.STORE -> stringResource(R.string.pod_tab_store)
            }
            TabPill(label, manager.state.activeTab == tab) { manager.selectTab(tab) }
        }
    }
}

/** One tab: the selected one is a card with the title in the accent colour, the others plain text. */
@Composable
private fun TabPill(label: String, selected: Boolean, onClick: () -> Unit) {
    val theme = LocalZTheme.current
    val shape = RoundedCornerShape(50)
    PadTile(onClick, Modifier.height(38.dp), shape = shape, surface = if (selected) TileSurface.CARD else TileSurface.NONE) {
        Text(
            label,
            Modifier.align(Alignment.Center).padding(horizontal = 18.dp),
            color = if (selected) theme.accentStart else theme.textPrimary.copy(alpha = 0.6f),
            fontSize = 15.sp,
            fontWeight = if (selected) FontWeight.Bold else FontWeight.SemiBold,
            maxLines = 1,
        )
    }
}
