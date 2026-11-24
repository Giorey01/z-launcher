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
import dev.zlauncher.ui.pad.FocusMemory

/** Columns of the game grid. */
const val GAME_COLUMNS = 5

/** Rows the grid fills with empty slots when there are fewer games. */
private const val MIN_ROWS = 3

/** A cell of the game grid. */
private sealed interface Cell {
    /** A game. */
    data class Game(val game: GameEntry) : Cell

    /** An empty slot at [index]. */
    data class Empty(val index: Int) : Cell
}

/** The games followed by empty slots that complete the last row and fill [MIN_ROWS] rows. */
private fun cellsOf(games: List<GameEntry>): List<Cell> {
    val rows = maxOf(MIN_ROWS, (games.size + GAME_COLUMNS - 1) / GAME_COLUMNS)
    val total = rows * GAME_COLUMNS
    return games.map { Cell.Game(it) } + (games.size until total).map { Cell.Empty(it) }
}

/**
 * The covers of [games] in five square columns with faint empty slots after the last one. A launches, Y opens
 * the game's menu; the focused game is shown in the hero area.
 */
@Composable
fun GameGrid(app: AppViewModel, games: List<GameEntry>, state: LazyGridState, memory: FocusMemory) {
    val spec = LocalGridSpec.current
    if (spec.rows == 1) {
        OneRow(games, spec) { game, _ ->
            GameTile(
                game,
                { app.launchGame(game) },
                { openGameMenu(app, game) },
                Modifier.then(memory.track(gameKey(game.id))),
                onFocused = { app.hero = HeroTarget.Game(game) }
            )
        }
        return
    }
    LazyVerticalGrid(
        GridCells.Fixed(if (spec.rows == 0) GAME_COLUMNS else spec.gameColumns),
        Modifier.fillMaxSize().padding(horizontal = 5.dp),
        state,
        contentPadding = androidx.compose.foundation.layout.PaddingValues(top = 6.dp),
        horizontalArrangement = Arrangement.Start,
    ) {
        items(cellsOf(games), key = { if (it is Cell.Game) it.game.id else -1L - (it as Cell.Empty).index }) { cell ->
            when (cell) {
                is Cell.Game -> GameTile(
                    cell.game,
                    { app.launchGame(cell.game) },
                    { openGameMenu(app, cell.game) },
                    Modifier.then(memory.track(gameKey(cell.game.id))),
                    onFocused = { app.hero = HeroTarget.Game(cell.game) },
                )
                is Cell.Empty -> EmptySlot()
            }
        }
    }
}

/** An invisible spacer of the given height, for headers that sit between the dock and a grid. */
@Composable
fun GridTopSpace() {
    Box(Modifier.padding(top = 4.dp))
}
