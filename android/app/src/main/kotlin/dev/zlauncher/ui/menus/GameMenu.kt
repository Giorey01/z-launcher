package dev.zlauncher.ui.menus

import android.hardware.display.DisplayManager
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Edit
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material.icons.rounded.Smartphone
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.activity.compose.BackHandler
import dev.zlauncher.R
import dev.zlauncher.display.DisplayRoles
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.Overlay
import dev.zlauncher.ui.art.CoverArt
import dev.zlauncher.ui.home.rememberOverlay
import dev.zlauncher.ui.theme.LocalZTheme

/** Share of the screen width of the entries panel. */
private const val PANEL_WIDTH = 0.49f

/** Height of a row of the Y menu. */
private val ROW = 50.dp

/** Side of the preview card. */
private val PREVIEW = 97.dp

/** The screens a game can start on, from the displays that are on. */
private data class Target(
    /** Display id. */
    val displayId: Int,
    /** `true` for the larger display (shown as the top screen). */
    val big: Boolean,
)

/**
 * The Y menu of a game: its preview and title at the left, the entries at the right. Launch starts the game
 * (with several displays it opens a submenu to pick the screen); Emulator picks the emulator for the platform.
 * B closes the menu, or goes back from the submenu.
 */
@Composable
fun GameMenu(app: AppViewModel, menu: Overlay.GameMenu) {
    val game = menu.game
    var submenu by remember { mutableStateOf(false) }
    val targets = displayTargets()
    BackHandler(submenu) { submenu = false }
    Box(Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.35f))) {
        Row(Modifier.fillMaxSize().padding(horizontal = 12.dp), verticalAlignment = Alignment.CenterVertically) {
            Preview(game.icon, game.platformId, game.displayTitle, Modifier.weight(1f))
            MenuPanel(
                entries(app, menu, targets, submenu) { submenu = it },
                ROW,
                submenu,
                RoundedCornerShape(28.dp),
                Modifier.fillMaxWidth(PANEL_WIDTH).padding(start = 4.dp)
            )
        }
    }
}

/** The menu rows for the current level. */
@Composable
private fun entries(
    app: AppViewModel,
    menu: Overlay.GameMenu,
    targets: List<Target>,
    submenu: Boolean,
    setSubmenu: (Boolean) -> Unit
): List<MenuEntry> {
    val game = menu.game
    if (submenu) {
        return targets.map { target ->
            val res = if (target.big) R.string.menu_screen_top else R.string.menu_screen_bottom
            MenuEntry(Icons.Rounded.Smartphone, stringResource(res)) {
                app.overlay = null
                app.launchGame(game, target.displayId)
            }
        }
    }
    val launch = MenuEntry(Icons.Rounded.PlayArrow, stringResource(R.string.menu_launch), submenu = targets.size > 1) {
        if (targets.size > 1) setSubmenu(true) else { app.overlay = null; app.launchGame(game) }
    }
    val edit = MenuEntry(Icons.Rounded.Edit, stringResource(R.string.menu_edit), badge = "Y") {
        app.overlay = null
        app.chooseEmulator(game)
    }
    return listOf(launch, edit)
}

/** The displays that are on, the larger as the top screen. */
@Composable
private fun displayTargets(): List<Target> {
    val context = LocalContext.current
    val settings = LocalZTheme.current.settings
    val displays = remember(settings) { DisplayRoles.candidates(context.getSystemService(DisplayManager::class.java), settings.externalDisplayDocking) }
    if (displays.size < 2) return emptyList()
    val biggest = displays.maxBy { it.mode.physicalWidth.toLong() * it.mode.physicalHeight }
    return displays.map { Target(it.displayId, it.displayId == biggest.displayId) }.sortedByDescending { it.big }
}

/** The framed cover of the game and its title under it. */
@Composable
private fun Preview(cover: String?, platformId: String, title: String, modifier: Modifier) {
    val theme = LocalZTheme.current
    Column(modifier, horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(10.dp)) {
        CoverArt(cover, rememberOverlay(platformId, theme), theme.settings.iconRoundness, Modifier.size(PREVIEW)) {
            Box(Modifier.fillMaxSize().background(theme.cardBrush()))
        }
        Text(
            title,
            Modifier.width(150.dp),
            color = theme.textPrimary,
            fontSize = 15.sp,
            fontWeight = FontWeight.Medium,
            textAlign = TextAlign.Center,
            maxLines = 3
        )
    }
}
