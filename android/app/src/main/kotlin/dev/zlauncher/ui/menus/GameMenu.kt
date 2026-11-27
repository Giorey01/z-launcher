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
import androidx.compose.material.icons.rounded.Download
import androidx.compose.material.icons.rounded.DragHandle
import androidx.compose.material.icons.rounded.Folder
import androidx.compose.material.icons.rounded.SelectAll
import androidx.compose.material.icons.rounded.Edit
import androidx.compose.material.icons.rounded.Info
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material.icons.rounded.RemoveCircleOutline
import androidx.compose.material.icons.rounded.Star
import androidx.compose.material.icons.rounded.StarBorder
import androidx.compose.material.icons.rounded.VisibilityOff
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
import dev.zlauncher.core.nativecore.GameEntry
import dev.zlauncher.display.DisplayRoles
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.Overlay
import dev.zlauncher.nav.Screen
import dev.zlauncher.ui.art.CoverArt
import dev.zlauncher.ui.home.rememberOverlay
import dev.zlauncher.ui.theme.LocalZTheme

/** Id of the Hidden Smart Folder (from the core). */
private const val HIDDEN_FOLDER = "@hidden"

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

/** Submenu level for the game menu. */
private enum class MenuLevel {
    Main,
    Displays,
    Rating,
    Folders,
}

/**
 * The Y menu of a game: its preview and title at the left, the entries at the right. Launch starts the game
 * (with several displays it opens a submenu to pick the screen); Emulator picks the emulator for the platform.
 * B closes the menu, or goes back from the submenu.
 */
@Composable
fun GameMenu(app: AppViewModel, menu: Overlay.GameMenu) {
    val game = menu.game
    var level by remember { mutableStateOf(MenuLevel.Main) }
    val targets = displayTargets()
    BackHandler(level != MenuLevel.Main) { level = MenuLevel.Main }
    Box(Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.35f))) {
        Row(Modifier.fillMaxSize().padding(horizontal = 12.dp), verticalAlignment = Alignment.CenterVertically) {
            Preview(game.icon, game.platformId, game.displayTitle, Modifier.weight(1f))
            MenuPanel(
                entries(app, menu, targets, level) { level = it },
                ROW,
                level != MenuLevel.Main,
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
    level: MenuLevel,
    setLevel: (MenuLevel) -> Unit,
): List<MenuEntry> = when (level) {
    MenuLevel.Displays -> targetEntries(app, menu.game, targets)
    MenuLevel.Rating -> ratingEntries(app, menu.game.id)
    MenuLevel.Folders -> folderRows(app, menu.game)
    MenuLevel.Main -> mainEntries(app, menu, targets, setLevel)
}

@Composable
private fun targetEntries(app: AppViewModel, game: GameEntry, targets: List<Target>): List<MenuEntry> =
    targets.map { target ->
        val res = if (target.big) R.string.menu_screen_top else R.string.menu_screen_bottom
        MenuEntry(Icons.Rounded.Smartphone, stringResource(res)) {
            app.overlay = null
            app.launchGame(game, target.displayId)
        }
    }

@Composable
private fun ratingEntries(app: AppViewModel, gameId: Long): List<MenuEntry> {
    val stars = listOf(5.0f, 4.0f, 3.0f, 2.0f, 1.0f)
    val entries = stars.map { rating ->
        val label = "★".repeat(rating.toInt()) + "☆".repeat(5 - rating.toInt()) + "  (${rating.toInt()}/5)"
        MenuEntry(Icons.Rounded.Star, label) {
            app.overlay = null
            app.library.setRating(gameId, rating)
        }
    }
    val remove = MenuEntry(Icons.Rounded.StarBorder, stringResource(R.string.menu_rating_remove)) {
        app.overlay = null
        app.library.setRating(gameId, null)
    }
    return entries + remove
}

/** The manual folders as rows: a check marks those that hold the game, A puts it in or takes it out. */
@Composable
private fun folderRows(app: AppViewModel, game: GameEntry): List<MenuEntry> {
    val folders = rememberCollections(app)
    val inside = rememberGameFolders(app, game.id)
    return folderEntries(folders, null, { it.id in inside }, { app.organize.toggleIn(it, game.id, it.id in inside) }) {
        app.open(Screen.FolderName(null, "", listOf(game.id)))
    }
}

/** The entries that organise the game: folders, selection and, in a manual folder, taking it out and reordering. */
@Composable
private fun organizeEntries(app: AppViewModel, game: GameEntry, setLevel: (MenuLevel) -> Unit): List<MenuEntry> {
    val manual = app.organize.manualId != null
    val inFolder = app.top is Screen.Platform
    return listOfNotNull(
        MenuEntry(Icons.Rounded.Folder, stringResource(R.string.menu_add_to_folder), submenu = true) { setLevel(MenuLevel.Folders) },
        MenuEntry(Icons.Rounded.RemoveCircleOutline, stringResource(R.string.menu_remove_from_folder)) {
            app.organize.removeFromFolder(listOf(game.id))
        }.takeIf { manual },
        MenuEntry(Icons.Rounded.DragHandle, stringResource(R.string.menu_reorder)) { app.organize.grab(game) }.takeIf { manual },
        MenuEntry(Icons.Rounded.SelectAll, stringResource(R.string.menu_select_multiple)) { app.organize.startSelecting(game) }
            .takeIf { inFolder },
    )
}

@Composable
private fun mainEntries(
    app: AppViewModel,
    menu: Overlay.GameMenu,
    targets: List<Target>,
    setLevel: (MenuLevel) -> Unit,
): List<MenuEntry> {
    val game = menu.game
    val launch = MenuEntry(Icons.Rounded.PlayArrow, stringResource(R.string.menu_launch), submenu = targets.size > 1) {
        if (targets.size > 1) setLevel(MenuLevel.Displays) else { app.overlay = null; app.launchGame(game) }
    }
    val info = MenuEntry(Icons.Rounded.Info, stringResource(R.string.menu_info)) {
        app.open(Screen.GameInfo(game))
    }
    val favorite = MenuEntry(
        if (game.favorite) Icons.Rounded.Star else Icons.Rounded.StarBorder,
        stringResource(if (game.favorite) R.string.menu_favorite_remove else R.string.menu_favorite_add),
    ) { app.library.setFavorite(game, !game.favorite) }
    val rating = MenuEntry(Icons.Rounded.Star, stringResource(R.string.menu_rating), submenu = true) {
        setLevel(MenuLevel.Rating)
    }
    val edit = MenuEntry(Icons.Rounded.Edit, stringResource(R.string.menu_edit), badge = "Y") {
        app.overlay = null
        app.chooseEmulator(game)
    }
    val art = MenuEntry(Icons.Rounded.Download, stringResource(R.string.menu_download_art)) {
        app.overlay = null
        app.downloadArt(game)
    }
    val unfile = game.appPackage?.let {
        MenuEntry(Icons.Rounded.RemoveCircleOutline, stringResource(R.string.menu_unfile_console)) { app.library.unfileApp(game) }
    }
    val hidden = app.top.let { it is Screen.Platform && it.id == HIDDEN_FOLDER }
    val hide = MenuEntry(Icons.Rounded.VisibilityOff, stringResource(if (hidden) R.string.menu_unhide else R.string.menu_hide)) {
        app.library.setHidden(game, !hidden)
    }
    // The actions of the Pods, after the launcher's own and before hiding.
    val pods = app.zl.pods.gameActionSlots(game).map { slot ->
        MenuEntry(slot.icon, slot.label) {
            app.overlay = null
            app.open(Screen.Pod(slot.podId, slot.route))
        }
    }
    val organize = organizeEntries(app, game, setLevel)
    return listOfNotNull(launch, info, favorite, rating, edit.takeIf { game.appPackage == null }, art) + organize + pods + listOfNotNull(hide, unfile)
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
