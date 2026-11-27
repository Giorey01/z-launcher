package dev.zlauncher.nav

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.viewModelScope
import dev.zlauncher.R
import dev.zlauncher.core.nativecore.CollectionInfo
import dev.zlauncher.core.nativecore.Core
import dev.zlauncher.core.nativecore.CoreException
import dev.zlauncher.core.nativecore.GameEntry
import dev.zlauncher.core.nativecore.SortOrder
import dev.zlauncher.launch.toMessage
import kotlinx.coroutines.launch

/** How a tile of the game grid is marked. */
enum class TileMark {
    /** Not marked. */
    NONE,

    /** Part of the selection (multiple selection). */
    SELECTED,

    /** Picked up to be put down somewhere else (manual order). */
    GRABBED,
}

/**
 * What the user does to organise the library: manual folders (RF-15), sort orders and the multiple selection of the
 * grid (RF-16), and the pick-up-and-put-down order of a manual folder (RF-26). The rules live in the core; this keeps
 * the state of the grid (what is selected, what is picked up) and calls it.
 */
class OrganizeActions(private val app: AppViewModel) {
    /** The grid is in selection mode: A marks games instead of launching them. */
    var selecting by mutableStateOf(false)
        private set

    /** The ids of the selected games. */
    var selected by mutableStateOf<Set<Long>>(emptySet())
        private set

    /** The game picked up to be moved in a manual folder, `null` when none. */
    var grabbed by mutableStateOf<Long?>(null)
        private set

    /** The games of the folder on screen, in the order shown; the grid keeps it up to date. */
    var shown: List<GameEntry> = emptyList()

    /** Columns of the grid on screen, to move a picked up game one row up or down. */
    var columns: Int = 1

    /** The id of the folder on screen, `null` on the Home or elsewhere. */
    val folderId: String? get() = (app.top as? Screen.Platform)?.id

    /** The name of the folder on screen, empty when none. */
    private val folderName: String get() = (app.top as? Screen.Platform)?.name.orEmpty()

    /** The row id of the manual folder on screen, `null` when it is another kind of folder. */
    val manualId: Long? get() = manualFolderId(folderId)

    /** The mark of the tile of game [id]. */
    fun markOf(id: Long): TileMark = when {
        grabbed == id -> TileMark.GRABBED
        id in selected -> TileMark.SELECTED
        else -> TileMark.NONE
    }

    /** A on a game tile: puts down the picked up game, marks or unmarks in selection mode, else launches. */
    fun activate(game: GameEntry) {
        when {
            grabbed != null -> grabbed = null
            selecting -> toggle(game.id)
            else -> app.launchGame(game)
        }
    }

    /** Y on a game tile: the actions of the selection in selection mode, else [openGameMenu]; nothing while a game is picked up. */
    fun menu(openGameMenu: () -> Unit) {
        when {
            grabbed != null -> Unit
            selecting -> app.overlay = Overlay.SelectionMenu
            else -> openGameMenu()
        }
    }

    /** Starts the multiple selection with [game] marked. */
    fun startSelecting(game: GameEntry) {
        app.overlay = null
        selecting = true
        selected = setOf(game.id)
    }

    /** Marks game [id], or unmarks it. */
    private fun toggle(id: Long) {
        selected = if (id in selected) selected - id else selected + id
    }

    /** Marks every game of the folder on screen. */
    fun selectAll() {
        app.overlay = null
        selected = shown.map { it.id }.toSet()
    }

    /** Leaves the selection mode and the grab; the screen changed or the action is done. */
    fun reset() {
        selecting = false
        selected = emptySet()
        grabbed = null
    }

    /** B: puts down a picked up game, or leaves the selection; `false` when there is nothing to leave. */
    fun cancel(): Boolean {
        if (grabbed == null && !selecting) return false
        reset()
        return true
    }

    /** Picks up [game] to move it in the manual folder on screen, shown in the user's own order. */
    fun grab(game: GameEntry) {
        app.overlay = null
        reset()
        app.viewModelScope.launch {
            try {
                // Moving only makes sense in the user's own order: drop another order chosen for this folder.
                app.zl.core.call { it.setSortOrder(folderId, null) }
                app.refresh()
                grabbed = game.id
                app.message = UiMessage(R.string.msg_grab_hint)
            } catch (e: CoreException) {
                app.message = e.toMessage()
            }
        }
    }

    /** Moves the picked up game by [delta] places (a row is [columns] places) in the manual folder on screen. */
    fun moveGrabbed(delta: Int) {
        val id = grabbed
        val folder = manualId
        if (id == null || folder == null) return
        val from = shown.indexOfFirst { it.id == id }
        val to = (from + delta).coerceIn(0, shown.lastIndex)
        if (from < 0 || to == from) return
        // Reorder the list at once so a second press before the core answers starts from the new place.
        shown = shown.toMutableList().apply { add(to, removeAt(from)) }
        run { it.moveInCollection(folder, id, to.toUInt()) }
    }

    /** Runs [block] with the core, then reloads the lists; a core error becomes a message. */
    private fun run(done: UiMessage? = null, block: (Core) -> Unit) {
        app.viewModelScope.launch {
            try {
                app.zl.core.call(block)
                done?.let { app.message = it }
                app.refresh()
            } catch (e: CoreException) {
                app.message = e.toMessage()
            }
        }
    }

    /** Makes the folder [name] (then puts [thenAdd] in it, leaving [thenMoveFrom]) or renames folder [renameId]. */
    fun saveFolderName(renameId: Long?, name: String, thenAdd: List<Long>, thenMoveFrom: Long?) {
        app.viewModelScope.launch {
            try {
                val shownName = app.zl.core.call { core ->
                    if (renameId != null) {
                        core.renameCollection(renameId, name)
                        name
                    } else {
                        val made = core.createCollection(name)
                        if (thenMoveFrom != null) {
                            core.moveBetweenCollections(thenMoveFrom, made.id, thenAdd)
                        } else if (thenAdd.isNotEmpty()) {
                            core.addToCollection(made.id, thenAdd)
                        }
                        made.name
                    }
                }
                app.back()
                reset()
                app.message = UiMessage(if (renameId != null) R.string.msg_folder_renamed else R.string.msg_folder_created, listOf(shownName))
                app.refresh()
            } catch (_: CoreException.InvalidInput) {
                app.message = UiMessage(R.string.msg_folder_name_refused)
            } catch (e: CoreException) {
                app.message = e.toMessage()
            }
        }
    }

    /** Deletes the manual folder [id]; its games stay in the library. */
    fun deleteFolder(id: Long, name: String) {
        app.overlay = null
        run(UiMessage(R.string.msg_folder_deleted, listOf(name))) { it.deleteCollection(id) }
    }

    /** Puts [ids] in [folder] (at its end). */
    fun addTo(folder: CollectionInfo, ids: List<Long>) {
        app.overlay = null
        reset()
        run(UiMessage(R.string.msg_added_to_folder, listOf(ids.size.toString(), folder.name))) { it.addToCollection(folder.id, ids) }
    }

    /** Takes game [id] out of [folder], or puts it in when [inside] is false (the toggle of the game menu). */
    fun toggleIn(folder: CollectionInfo, id: Long, inside: Boolean) {
        app.overlay = null
        if (inside) {
            run(UiMessage(R.string.msg_removed_from_folder, listOf("1", folder.name))) { it.removeFromCollection(folder.id, listOf(id)) }
        } else {
            addTo(folder, listOf(id))
        }
    }

    /** Moves [ids] from the manual folder on screen to [folder]. */
    fun moveTo(folder: CollectionInfo, ids: List<Long>) {
        val from = manualId ?: return addTo(folder, ids)
        app.overlay = null
        reset()
        run(UiMessage(R.string.msg_moved_to_folder, listOf(ids.size.toString(), folder.name))) { it.moveBetweenCollections(from, folder.id, ids) }
    }

    /** Takes [ids] out of the manual folder on screen. */
    fun removeFromFolder(ids: List<Long>) {
        val from = manualId ?: return
        app.overlay = null
        reset()
        run(UiMessage(R.string.msg_removed_from_folder, listOf(ids.size.toString(), folderName))) { it.removeFromCollection(from, ids) }
    }

    /** Hides the selected games. */
    fun hideSelected() {
        val ids = selected.toList()
        app.overlay = null
        reset()
        run(UiMessage(R.string.msg_selection_hidden, listOf(ids.size.toString()))) { core -> ids.forEach { core.setHidden(it, true) } }
    }

    /** Sets the order of the folder on screen, or goes back to its default with `null`. */
    fun setSort(order: SortOrder?) {
        val folder = folderId
        app.overlay = null
        run { it.setSortOrder(folder, order) }
    }

    /** Makes [order] the order of every folder and drops the choice of the one on screen. */
    fun setSortForAll(order: SortOrder) {
        val folder = folderId
        app.overlay = null
        run { core ->
            core.setSortOrder(null, order)
            core.setSortOrder(folder, null)
        }
    }
}
