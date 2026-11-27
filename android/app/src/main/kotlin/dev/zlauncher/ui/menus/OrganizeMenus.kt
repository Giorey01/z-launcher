package dev.zlauncher.ui.menus

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.CheckCircle
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material.icons.rounded.CreateNewFolder
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material.icons.rounded.Edit
import androidx.compose.material.icons.rounded.Folder
import androidx.compose.material.icons.rounded.RadioButtonChecked
import androidx.compose.material.icons.rounded.RadioButtonUnchecked
import androidx.compose.material.icons.rounded.RemoveCircleOutline
import androidx.compose.material.icons.rounded.SelectAll
import androidx.compose.material.icons.rounded.VisibilityOff
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import dev.zlauncher.R
import dev.zlauncher.core.nativecore.CollectionInfo
import dev.zlauncher.core.nativecore.CoreException
import dev.zlauncher.core.nativecore.SortOrder
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.Overlay
import dev.zlauncher.nav.Screen

/** Share of the screen width of a menu panel. */
private const val PANEL_WIDTH = 0.49f

/** Height of a row of these menus. */
private val ROW = 50.dp

/** The manual folders, read again after every change of the library. */
@Composable
fun rememberCollections(app: AppViewModel): List<CollectionInfo> =
    produceState(emptyList<CollectionInfo>(), app.resumeTick) {
        value = try {
            app.zl.core.call { it.collections() }
        } catch (_: CoreException) {
            emptyList()
        }
    }.value

/** The ids of the manual folders that hold game [gameId], read again after every change of the library. */
@Composable
fun rememberGameFolders(app: AppViewModel, gameId: Long): Set<Long> =
    produceState(emptySet<Long>(), app.resumeTick, gameId) {
        value = try {
            app.zl.core.call { it.gameCollections(gameId) }.toSet()
        } catch (_: CoreException) {
            emptySet()
        }
    }.value

/** One row per manual folder (not [exclude]) and a last row that makes a new one; [marked] folders show a check. */
@Composable
fun folderEntries(
    folders: List<CollectionInfo>,
    exclude: Long?,
    marked: (CollectionInfo) -> Boolean,
    onPick: (CollectionInfo) -> Unit,
    onNew: () -> Unit,
): List<MenuEntry> {
    val rows = folders.filter { it.id != exclude }.map { folder ->
        MenuEntry(if (marked(folder)) Icons.Rounded.CheckCircle else Icons.Rounded.Folder, folder.name) { onPick(folder) }
    }
    return rows + MenuEntry(Icons.Rounded.CreateNewFolder, stringResource(R.string.menu_new_folder)) { onNew() }
}

/** A panel of [entries] at the right of the screen over a dimmed Home; [resetKey] puts the focus on the first row. */
@Composable
private fun SideMenu(entries: List<MenuEntry>, resetKey: Any) {
    Box(Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.35f))) {
        MenuPanel(
            entries,
            ROW,
            resetKey,
            RoundedCornerShape(28.dp),
            Modifier.fillMaxWidth(PANEL_WIDTH).align(Alignment.CenterEnd).padding(end = 12.dp),
        )
    }
}

/** The string that names a sort order. */
private fun orderLabel(order: SortOrder): Int = when (order) {
    SortOrder.NAME -> R.string.sort_name
    SortOrder.LAST_PLAYED -> R.string.sort_last_played
    SortOrder.PLAY_TIME -> R.string.sort_play_time
    SortOrder.ADDED -> R.string.sort_added
    SortOrder.RATING -> R.string.sort_rating
    SortOrder.MANUAL -> R.string.sort_manual
}

/** The name of a sort order, for the header of a folder. */
@Composable
fun sortLabel(order: SortOrder): String = stringResource(orderLabel(order))

/**
 * The sort orders of the folder on screen: the pick applies to this folder; one row makes it the order of every
 * folder, another goes back to the folder's default. A manual folder also has its own order.
 */
@Composable
fun SortMenu(app: AppViewModel) {
    val folder = app.organize.folderId
    val manual = app.organize.manualId != null
    val current by produceState<SortOrder?>(null, app.resumeTick, folder) {
        value = try {
            app.zl.core.call { it.sortOrder(folder) }
        } catch (_: CoreException) {
            null
        }
    }
    val orders = listOf(SortOrder.NAME, SortOrder.LAST_PLAYED, SortOrder.PLAY_TIME, SortOrder.ADDED, SortOrder.RATING) +
        listOfNotNull(SortOrder.MANUAL.takeIf { manual })
    val rows = orders.map { order ->
        val icon = if (order == current) Icons.Rounded.RadioButtonChecked else Icons.Rounded.RadioButtonUnchecked
        MenuEntry(icon, stringResource(orderLabel(order))) { app.organize.setSort(order) }
    }
    val shared = current?.takeIf { it != SortOrder.MANUAL }?.let { order ->
        MenuEntry(Icons.Rounded.Folder, stringResource(R.string.sort_apply_all, stringResource(orderLabel(order)))) {
            app.organize.setSortForAll(order)
        }
    }
    val reset = MenuEntry(Icons.Rounded.Close, stringResource(R.string.sort_reset)) { app.organize.setSort(null) }
    SideMenu(rows + listOfNotNull(shared, reset), Unit)
}

/** The levels of the selection menu. */
private enum class SelectionLevel {
    Main,
    Add,
    Move,
}

/** What can be done to the games selected in the grid: put them in a folder, move or take them out, hide them. */
@Composable
fun SelectionMenu(app: AppViewModel) {
    val org = app.organize
    val folders = rememberCollections(app)
    var level by remember { mutableStateOf(SelectionLevel.Main) }
    BackHandler(level != SelectionLevel.Main) { level = SelectionLevel.Main }
    val ids = org.selected.toList()
    val from = org.manualId
    val entries = when (level) {
        SelectionLevel.Main -> listOfNotNull(
            MenuEntry(Icons.Rounded.Folder, stringResource(R.string.menu_add_to_folder), submenu = true) { level = SelectionLevel.Add },
            MenuEntry(Icons.Rounded.Folder, stringResource(R.string.menu_move_to_folder), submenu = true) { level = SelectionLevel.Move }
                .takeIf { from != null },
            MenuEntry(Icons.Rounded.RemoveCircleOutline, stringResource(R.string.menu_remove_from_folder)) { org.removeFromFolder(ids) }
                .takeIf { from != null },
            MenuEntry(Icons.Rounded.VisibilityOff, stringResource(R.string.menu_hide)) { org.hideSelected() },
            MenuEntry(Icons.Rounded.SelectAll, stringResource(R.string.menu_select_all)) { org.selectAll() },
            MenuEntry(Icons.Rounded.Close, stringResource(R.string.menu_selection_cancel)) {
                app.overlay = null
                org.reset()
            },
        )
        SelectionLevel.Add -> folderEntries(folders, null, { false }, { org.addTo(it, ids) }) {
            app.open(Screen.FolderName(null, "", ids))
        }
        SelectionLevel.Move -> folderEntries(folders, from, { false }, { org.moveTo(it, ids) }) {
            app.open(Screen.FolderName(null, "", ids, from))
        }
    }
    SideMenu(entries, level)
}

/** The Y menu of a manual folder on the Home: rename it, or delete it after one more A (its games stay). */
@Composable
fun FolderTileMenu(app: AppViewModel, menu: Overlay.FolderMenu) {
    var confirming by remember { mutableStateOf(false) }
    val folder = menu.folder
    val id = dev.zlauncher.nav.manualFolderId(folder.id)
    BackHandler(confirming) { confirming = false }
    val entries = if (confirming) {
        listOf(
            MenuEntry(Icons.Rounded.Delete, stringResource(R.string.menu_folder_delete_confirm, folder.name)) {
                id?.let { app.organize.deleteFolder(it, folder.name) }
            },
        )
    } else {
        listOf(
            MenuEntry(Icons.Rounded.Edit, stringResource(R.string.menu_folder_rename)) {
                id?.let { app.open(Screen.FolderName(it, folder.name)) }
            },
            MenuEntry(Icons.Rounded.Delete, stringResource(R.string.menu_folder_delete), submenu = true) { confirming = true },
        )
    }
    SideMenu(entries, confirming)
}
