package dev.zlauncher.ui.menus

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.PushPin
import androidx.compose.material.icons.rounded.SportsEsports
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.zlauncher.R
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.Overlay
import dev.zlauncher.nav.Screen
import dev.zlauncher.ui.AppIcon
import dev.zlauncher.ui.theme.LocalZTheme

/** Share of the screen width of the entries panel. */
private const val PANEL_WIDTH = 0.49f

/** Height of a row of the menu. */
private val ROW = 50.dp

/** Side of the app icon at the left. */
private val ICON = 97.dp

/**
 * The Y menu of an app in the drawer: the icon and label at the left, the entries at the right. Pin adds the app
 * to the dock or removes it; Move to a console opens the list of consoles to file it under (a recomp or a port
 * then sits with that console's games). B closes the menu.
 */
@Composable
fun AppMenu(app: AppViewModel, menu: Overlay.AppMenu) {
    val pin = MenuEntry(
        Icons.Rounded.PushPin,
        stringResource(if (menu.pinned) R.string.menu_unpin_dock else R.string.menu_pin_dock),
    ) { app.togglePin(menu.packageName) }
    val file = MenuEntry(Icons.Rounded.SportsEsports, stringResource(R.string.menu_file_console), submenu = true) {
        app.open(Screen.ConsolePicker(menu.packageName, menu.label))
    }
    Box(Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.35f))) {
        Row(Modifier.fillMaxSize().padding(horizontal = 12.dp), verticalAlignment = Alignment.CenterVertically) {
            Preview(menu.packageName, menu.label, Modifier.weight(1f))
            MenuPanel(
                listOf(pin, file),
                ROW,
                menu.packageName,
                RoundedCornerShape(28.dp),
                Modifier.fillMaxWidth(PANEL_WIDTH).padding(start = 4.dp),
            )
        }
    }
}

/** The app's icon and its label under it. */
@Composable
private fun Preview(packageName: String, label: String, modifier: Modifier) {
    val theme = LocalZTheme.current
    Column(modifier, horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(10.dp)) {
        AppIcon(packageName, ICON, Modifier.clip(RoundedCornerShape(22)))
        Text(
            label,
            Modifier.width(150.dp),
            color = theme.textPrimary,
            fontSize = 15.sp,
            fontWeight = FontWeight.Medium,
            textAlign = TextAlign.Center,
            maxLines = 3,
        )
    }
}
