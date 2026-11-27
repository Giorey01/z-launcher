package dev.zlauncher.ui.home

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyGridState
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dev.zlauncher.core.nativecore.GameEntry
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.HeroTarget
import dev.zlauncher.nav.Screen
import dev.zlauncher.pods.LeadTileSlotItem
import dev.zlauncher.ui.pad.FocusMemory

/** Columns of the game grid. */
const val GAME_COLUMNS = 5

/** Rows the grid fills with empty slots when there are fewer games. */
private const val MIN_ROWS = 3

/** A cell of the game grid. */
private sealed interface Cell {
    /** A Pod lead tile (§7). */
    data class Lead(val item: LeadTileSlotItem) : Cell

    /** A game. */
    data class Game(val game: GameEntry) : Cell

    /** An empty slot at [index]. */
    data class Empty(val index: Int) : Cell
}

/** The lead items and games followed by empty slots that complete the last row and fill [MIN_ROWS] rows. */
private fun cellsOf(leads: List<LeadTileSlotItem>, games: List<GameEntry>): List<Cell> {
    val items = leads.map { Cell.Lead(it) } + games.map { Cell.Game(it) }
    val rows = maxOf(MIN_ROWS, (items.size + GAME_COLUMNS - 1) / GAME_COLUMNS)
    val total = rows * GAME_COLUMNS
    return items + (items.size until total).map { Cell.Empty(it) }
}

/**
 * The covers of [games] preceded by optional [leads] (Pod lead tiles), in five square columns
 * with faint empty slots after the last one.
 */
@Composable
fun GameGrid(
    app: AppViewModel,
    games: List<GameEntry>,
    state: LazyGridState,
    memory: FocusMemory,
    leads: List<LeadTileSlotItem> = emptyList(),
) {
    val spec = LocalGridSpec.current
    app.organize.columns = if (spec.rows == 0) GAME_COLUMNS else spec.gameColumns
    val allCells = cellsOf(leads, games)
    if (spec.rows == 1) {
        val nonEmpties = leads.map { Cell.Lead(it) } + games.map { Cell.Game(it) }
        OneRow(nonEmpties, spec) { cell, _ -> RenderCell(app, cell, memory) }
        return
    }
    LazyVerticalGrid(
        GridCells.Fixed(if (spec.rows == 0) GAME_COLUMNS else spec.gameColumns),
        Modifier.fillMaxSize().padding(horizontal = 5.dp),
        state,
        contentPadding = androidx.compose.foundation.layout.PaddingValues(top = 6.dp),
        horizontalArrangement = Arrangement.Start,
    ) {
        items(allCells, key = {
            when (it) {
                is Cell.Lead -> "lead:${it.item.podId}:${it.item.label}"
                is Cell.Game -> it.game.id
                is Cell.Empty -> -1L - it.index
            }
        }) { cell ->
            RenderCell(app, cell, memory)
        }
    }
}

/** Draws one cell inside the grid. */
@Composable
private fun RenderCell(app: AppViewModel, cell: Cell, memory: FocusMemory) {
    when (cell) {
        is Cell.Lead -> LeadTile(
            cell.item,
            { app.open(Screen.Pod(cell.item.podId, cell.item.route)) },
            Modifier.then(memory.track("lead:${cell.item.podId}:${cell.item.label}")),
        )
        is Cell.Game -> GameTile(
            cell.game,
            { app.organize.activate(cell.game) },
            { app.organize.menu { openGameMenu(app, cell.game) } },
            Modifier.then(memory.track(gameKey(cell.game.id))),
            onFocused = { app.hero = HeroTarget.Game(cell.game) },
            mark = app.organize.markOf(cell.game.id),
        )
        is Cell.Empty -> EmptySlot()
    }
}

/** An invisible spacer of the given height, for headers that sit between the dock and a grid. */
@Composable
fun GridTopSpace() {
    Box(Modifier.padding(top = 4.dp))
}
