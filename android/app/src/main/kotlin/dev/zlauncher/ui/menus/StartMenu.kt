package dev.zlauncher.ui.menus

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Apps
import androidx.compose.material.icons.rounded.CreateNewFolder
import androidx.compose.material.icons.rounded.Extension
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material.icons.rounded.Settings
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import dev.zlauncher.R
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.Screen

/** Share of the screen width the Start menu panel takes. */
private const val PANEL_WIDTH = 0.535f

/** Height of a Start menu row. */
private val ROW = 73.dp

/**
 * The Start menu: a glass panel along the left edge. Only entries the core supports are listed.
 * Opened with Start on Home; B closes it.
 */
@Composable
fun StartMenu(app: AppViewModel) {
    Box(Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.35f))) {
        val context = LocalContext.current
        val pods = app.zl.pods.startMenuSlots(context).map { item ->
            MenuEntry(item.icon, item.label) { app.open(Screen.Pod(item.podId, item.route)) }
        }
        MenuPanel(
            pods + listOf(
                MenuEntry(Icons.Rounded.Apps, stringResource(R.string.menu_apps)) { app.open(Screen.Drawer) },
                MenuEntry(Icons.Rounded.Search, stringResource(R.string.menu_search), badge = "X") {
                    app.open(Screen.Search)
                },
                MenuEntry(Icons.Rounded.CreateNewFolder, stringResource(R.string.menu_new_folder)) {
                    app.open(Screen.FolderName(null, ""))
                },
                MenuEntry(Icons.Rounded.Settings, stringResource(R.string.menu_settings), submenu = true) {
                    app.open(Screen.Settings)
                },
                MenuEntry(Icons.Rounded.Extension, stringResource(R.string.pod_screen_title)) {
                    app.open(Screen.PodManagement)
                },
            ),
            ROW,
            Unit,
            RoundedCornerShape(topEnd = 28.dp, bottomEnd = 28.dp),
            Modifier.fillMaxWidth(PANEL_WIDTH).align(Alignment.TopStart),
        )
    }
}
