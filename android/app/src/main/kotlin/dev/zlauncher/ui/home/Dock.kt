package dev.zlauncher.ui.home

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.zlauncher.R
import dev.zlauncher.apps.AppEntry
import dev.zlauncher.ui.AppIcon
import dev.zlauncher.ui.pad.FocusMemory
import dev.zlauncher.ui.pad.PadTile
import dev.zlauncher.ui.theme.TextPrimary

/** Focus key of the "All apps" slot. */
const val DOCK_ALL_APPS_KEY = "dock:all"

/** Focus key of the setup slot. */
const val DOCK_SETUP_KEY = "dock:setup"

/** Size of a dock slot. */
private val SLOT_SIZE = 56.dp

/** Size of the icons inside a slot. */
private val ICON_SIZE = 40.dp

/**
 * The dock (RF-27): "All apps" first, then the pinned apps, and at the end the button that runs the setup again.
 * The slots are focusable tiles reached with the D-pad like the rest of the Home.
 */
@Composable
fun Dock(
    apps: List<AppEntry>,
    memory: FocusMemory,
    onAllApps: () -> Unit,
    onApp: (AppEntry) -> Unit,
    onSetup: () -> Unit,
) {
    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
        DockGlyph("▦", stringResource(R.string.dock_all_apps), memory, DOCK_ALL_APPS_KEY, onAllApps)
        apps.forEach { app ->
            PadTile({ onApp(app) }, Modifier.size(SLOT_SIZE).then(memory.track("dock:${app.packageName}"))) {
                Box(Modifier.size(SLOT_SIZE), contentAlignment = Alignment.Center) { AppIcon(app.packageName, ICON_SIZE) }
            }
        }
        Box(Modifier.weight(1f))
        DockGlyph("⚙", stringResource(R.string.dock_setup), memory, DOCK_SETUP_KEY, onSetup)
    }
}

/** A dock slot drawn with a text glyph. */
@Composable
private fun DockGlyph(glyph: String, description: String, memory: FocusMemory, key: String, onClick: () -> Unit) {
    PadTile(onClick, Modifier.size(SLOT_SIZE).then(memory.track(key)).semantics { contentDescription = description }) {
        Box(Modifier.size(SLOT_SIZE), contentAlignment = Alignment.Center) {
            Text(glyph, color = TextPrimary, fontSize = 26.sp)
        }
    }
}
