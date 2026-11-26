package dev.zlauncher.ui.search

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil.compose.AsyncImage
import dev.zlauncher.R
import dev.zlauncher.core.nativecore.GameEntry
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.HeroTarget
import dev.zlauncher.nav.Overlay
import dev.zlauncher.nav.Screen
import dev.zlauncher.ui.drawer.GamepadKeyboard
import dev.zlauncher.ui.pad.PadButton
import dev.zlauncher.ui.pad.PadTile
import dev.zlauncher.ui.pad.TileSurface
import dev.zlauncher.ui.theme.LocalZTheme
import dev.zlauncher.ui.theme.ZPanel
import kotlinx.coroutines.delay

private const val SEARCH_LIMIT = 50u
private const val SEARCH_DEBOUNCE_MS = 150L
private val CARD_SHAPE = RoundedCornerShape(12.dp)
private val QUERY_BOX_SHAPE = RoundedCornerShape(10.dp)

/**
 * Game search screen across all platforms using typo-tolerant fuzzy search (RF-16).
 * Supports D-pad navigation, live hero updates, and the on-screen GamepadKeyboard.
 */
@Composable
fun SearchScreen(app: AppViewModel) {
    val theme = LocalZTheme.current
    var query by remember { mutableStateOf("") }
    var games by remember { mutableStateOf<List<GameEntry>>(emptyList()) }
    var showKeyboard by remember { mutableStateOf(false) }

    LaunchedEffect(query) {
        delay(SEARCH_DEBOUNCE_MS)
        games = app.zl.core.call { it.searchGames(query, SEARCH_LIMIT) } ?: emptyList()
    }

    Box(
        Modifier.fillMaxSize()
            .background(theme.backgroundBrush())
            .padding(horizontal = 24.dp, vertical = 20.dp),
    ) {
        Column(Modifier.fillMaxSize()) {
            SearchHeader(
                query = query,
                count = games.size,
                onToggleKeyboard = { showKeyboard = !showKeyboard },
                onClear = { query = "" },
                onBack = { app.back() },
            )
            Spacer(Modifier.height(14.dp))
            if (showKeyboard) {
                Box(Modifier.fillMaxWidth().weight(1f), contentAlignment = Alignment.Center) {
                    GamepadKeyboard(
                        query = query,
                        onQueryChange = { query = it },
                        onConfirm = { showKeyboard = false },
                        onClose = { showKeyboard = false },
                    )
                }
            } else {
                SearchResultsGrid(app, games, query, Modifier.fillMaxWidth().weight(1f))
            }
        }
    }
}

/** Header row with search query bar and action buttons. */
@Composable
private fun SearchHeader(
    query: String,
    count: Int,
    onToggleKeyboard: () -> Unit,
    onClear: () -> Unit,
    onBack: () -> Unit,
) {
    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
        SearchQueryBar(query, count, onToggleKeyboard, onClear, Modifier.weight(1f))
        Spacer(Modifier.width(12.dp))
        PadButton(
            label = stringResource(R.string.search_keyboard),
            onClick = onToggleKeyboard,
        )
        Spacer(Modifier.width(8.dp))
        PadButton(stringResource(R.string.cancel), onClick = onBack)
    }
}

/** Displays the current query text or placeholder. */
@Composable
private fun SearchQueryBar(
    query: String,
    count: Int,
    onToggleKeyboard: () -> Unit,
    onClear: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val theme = LocalZTheme.current
    val barMod = modifier.height(52.dp).clip(QUERY_BOX_SHAPE).clickable { onToggleKeyboard() }
    ZPanel(barMod, QUERY_BOX_SHAPE, elevation = 2.dp) {
        Row(Modifier.fillMaxSize().padding(horizontal = 14.dp), verticalAlignment = Alignment.CenterVertically) {
            Icon(Icons.Rounded.Search, contentDescription = null, tint = theme.accentStart, modifier = Modifier.size(24.dp))
            Spacer(Modifier.width(10.dp))
            val text = query.ifEmpty { stringResource(R.string.search_hint) }
            val color = if (query.isEmpty()) theme.textSecondary else theme.textPrimary
            Text(
                text,
                color = color,
                fontSize = 16.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.weight(1f),
            )
            if (query.isNotEmpty()) {
                Text(stringResource(R.string.search_count, count), color = theme.textSecondary, fontSize = 12.sp)
                Spacer(Modifier.width(8.dp))
                PadTile(onClick = onClear, modifier = Modifier.size(28.dp)) {
                    Icon(Icons.Rounded.Close, contentDescription = null, tint = theme.textSecondary, modifier = Modifier.size(18.dp))
                }
            }
        }
    }
}

/** Grid showing matching game cards. */
@Composable
private fun SearchResultsGrid(
    app: AppViewModel,
    games: List<GameEntry>,
    query: String,
    modifier: Modifier = Modifier,
) {
    val theme = LocalZTheme.current
    if (games.isEmpty()) {
        Box(modifier, contentAlignment = Alignment.Center) {
            val emptyMsg = if (query.isEmpty()) "" else stringResource(R.string.search_no_results, query)
            Text(emptyMsg, color = theme.textSecondary, fontSize = 16.sp)
        }
    } else {
        LazyVerticalGrid(
            columns = GridCells.Adaptive(minSize = 160.dp),
            modifier = modifier,
            horizontalArrangement = Arrangement.spacedBy(12.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            items(games, key = { it.id }) { game ->
                SearchGameCard(app, game)
            }
        }
    }
}

/** Single game card in the search grid. */
@Composable
private fun SearchGameCard(app: AppViewModel, game: GameEntry) {
    val theme = LocalZTheme.current
    PadTile(
        onClick = { app.open(Screen.GameInfo(game)) },
        onSecondary = { app.overlay = Overlay.GameMenu(game, originKey = "search_${game.id}") },
        onFocused = { app.hero = HeroTarget.Game(game) },
        surface = TileSurface.CARD,
        shape = CARD_SHAPE,
        modifier = Modifier.fillMaxWidth().height(140.dp),
    ) {
        Column(Modifier.fillMaxSize().padding(10.dp)) {
            GameArtwork(game)
            Spacer(Modifier.height(8.dp))
            Text(
                game.displayTitle,
                color = theme.textPrimary,
                fontSize = 14.sp,
                fontWeight = FontWeight.SemiBold,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                game.platformName,
                color = theme.accentStart,
                fontSize = 11.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

/** Artwork thumbnail for a game card. */
@Composable
private fun GameArtwork(game: GameEntry) {
    val theme = LocalZTheme.current
    val artUrl = game.icon ?: game.hero
    if (artUrl != null) {
        AsyncImage(
            model = artUrl,
            contentDescription = null,
            contentScale = ContentScale.Crop,
            modifier = Modifier.fillMaxWidth().height(80.dp).clip(RoundedCornerShape(8.dp)),
        )
    } else {
        Box(
            Modifier.fillMaxWidth().height(80.dp).background(theme.cardEnd, RoundedCornerShape(8.dp)),
            contentAlignment = Alignment.Center,
        ) {
            Icon(
                Icons.Rounded.Search,
                null,
                tint = theme.textSecondary.copy(alpha = 0.4f),
                modifier = Modifier.size(32.dp),
            )
        }
    }
}
