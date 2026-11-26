package dev.zlauncher.ui.home

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Apps
import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusProperties
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import dev.zlauncher.R
import dev.zlauncher.apps.AppEntry
import dev.zlauncher.core.nativecore.DockIconShape
import dev.zlauncher.ui.AppIcon
import dev.zlauncher.ui.pad.FocusMemory
import dev.zlauncher.ui.pad.PadTile
import dev.zlauncher.ui.pad.TileSurface
import dev.zlauncher.ui.theme.LocalZTheme

/** Focus key of the "All apps" slot. */
const val DOCK_ALL_APPS_KEY = "dock:all"

/** Size of a dock icon. */
private val DOCK_ICON = 32.dp

/** Space between dock icons. */
private val DOCK_GAP = 12.dp

/** Corner size of the squircle shape, as a fraction of the side. */
private const val SQUIRCLE_PERCENT = 32

/** Corner size of the rounded-square shape, as a fraction of the side. */
private const val ROUNDED_PERCENT = 22

/** The shape of the dock icons from the `dock_icon_shape` setting. */
@Composable
private fun dockShape(): Shape = when (LocalZTheme.current.settings.dockIconShape) {
    DockIconShape.CIRCLE -> CircleShape
    DockIconShape.SQUIRCLE -> RoundedCornerShape(SQUIRCLE_PERCENT)
    DockIconShape.ROUNDED_SQUARE -> RoundedCornerShape(ROUNDED_PERCENT)
    DockIconShape.SQUARE -> RoundedCornerShape(4.dp)
}

/**
 * The dock: a row of small app icons centred at the top of the grid screen, starting with "All apps" then the
 * pinned apps. The slots are focusable and reached with the D-pad like the rest of the Home; focusing one
 * clears the hero area. [toGrid] names where the D-pad goes from the dock into the grid.
 */
@Composable
fun Dock(
    apps: List<AppEntry>,
    memory: FocusMemory,
    onAllApps: () -> Unit,
    onApp: (AppEntry) -> Unit,
    onFocused: () -> Unit,
    onTop: Boolean = true,
    toGrid: (() -> FocusRequester)? = null,
) {
    val theme = LocalZTheme.current
    val shape = dockShape()
    // On every slot: the focus properties apply to the focus target that follows them.
    val exit = Modifier.focusProperties { toGrid?.let { if (onTop) down = it() else up = it() } }
    Row(
        Modifier
            .fillMaxWidth()
            .padding(top = if (onTop) 28.dp else 0.dp, bottom = if (onTop) 0.dp else 20.dp),
        horizontalArrangement = Arrangement.spacedBy(DOCK_GAP, Alignment.CenterHorizontally),
        verticalAlignment = Alignment.CenterVertically
    ) {
        val allApps = stringResource(R.string.dock_all_apps)
        PadTile(
            onAllApps,
            exit.size(DOCK_ICON).then(memory.track(DOCK_ALL_APPS_KEY)).semantics { contentDescription = allApps },
            shape = shape,
            surface = TileSurface.NONE,
            cursorOutset = 3.dp,
            onFocused = onFocused
        ) {
            Box(Modifier.size(DOCK_ICON).clip(shape), contentAlignment = Alignment.Center) {
                Icon(Icons.Rounded.Apps, null, Modifier.size(DOCK_ICON), tint = theme.accentStart)
            }
        }
        apps.forEach { app ->
            PadTile(
                { onApp(app) },
                exit.size(DOCK_ICON).then(memory.track("dock:${app.packageName}")),
                shape = shape,
                surface = TileSurface.NONE,
                cursorOutset = 3.dp,
                onFocused = onFocused
            ) {
                AppIcon(app.packageName, DOCK_ICON, Modifier.clip(shape))
            }
        }
    }
}
