package dev.zlauncher.ui.info

import androidx.compose.foundation.background
import androidx.compose.foundation.focusGroup
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBarsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.focusProperties
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.zlauncher.R
import dev.zlauncher.core.nativecore.CoreException
import dev.zlauncher.core.nativecore.GameDetails
import dev.zlauncher.core.nativecore.GameEntry
import dev.zlauncher.core.nativecore.PlayStats
import dev.zlauncher.launch.toMessage
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.Screen
import dev.zlauncher.ui.art.ArtImage
import dev.zlauncher.pods.AppPodHost
import dev.zlauncher.pods.GameBadgeItem
import dev.zlauncher.pods.GameTabItem
import dev.zlauncher.ui.common.TabStrip
import dev.zlauncher.ui.pad.FocusMemory
import dev.zlauncher.ui.pad.FocusRestorer
import dev.zlauncher.ui.pad.PadButton
import dev.zlauncher.ui.pad.requestSoon
import dev.zlauncher.ui.pods.PodAlive
import dev.zlauncher.ui.theme.LocalZTheme
import dev.zlauncher.ui.theme.ZPanel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

/** Opacity of the dark layer over the hero image, so the text reads over any picture. */
private const val SCRIM_ALPHA = 0.72f

/** Opacity of the white behind a Pod's badge. */
private const val BADGE_ALPHA = 0.18f

/** Corner radius of the panel around a tab's content. */
private val FRAME_RADIUS = 20.dp

/** Padding inside the panel around a tab's content. */
private val FRAME_PADDING = 18.dp

/** Share of the width the text column takes; the media take the rest. */
private const val TEXT_SHARE = 0.46f

/** What the info screen shows, read from the core. */
private data class InfoData(
    /** Metadata and media. */
    val details: GameDetails,
    /** Play time. */
    val stats: PlayStats,
)

/**
 * The sheet of a game: a row of tabs (Info, Activity, then those of the Pods) over the content of the open one (RF-30,
 * RF-31, RF-78). LB and RB, or left and right on the row, change tab. Info has the hero image behind, at the left the logo
 * or title, the facts, the play time, the badges of the Pods, the description and the actions (Play, favourite, the emulator
 * for this game); at the right the trailer, streamed and muted, and the screenshots. A on the trailer or a screenshot
 * opens it full screen; B goes back.
 */
@Composable
fun GameInfoScreen(app: AppViewModel, screen: Screen.GameInfo) {
    val game = screen.game
    val tabs = app.infoTabs
    val podTabs = remember(game.id, app.resumeTick) { app.zl.pods.gameTabSlots(game) }
    val total = InfoTabs.FIXED_TABS + podTabs.size
    DisposableEffect(game.id) {
        tabs.open(total)
        onDispose { tabs.dialog = null }
    }
    SideEffect { tabs.resize(total) }
    val data by produceState<InfoData?>(null, game.id, app.resumeTick) {
        value = try {
            app.zl.core.call { InfoData(it.gameDetails(game.id), it.playStats(game.id)) }
        } catch (e: CoreException) {
            app.message = e.toMessage()
            null
        }
    }
    val badges by produceState(emptyList<GameBadgeItem>(), game.id, app.resumeTick) {
        value = withContext(Dispatchers.Default) { app.zl.pods.gameBadges(game) }
    }
    var viewer by remember { mutableStateOf<Viewer?>(null) }
    val memory = app.memory("info:" + game.id)
    Box(Modifier.fillMaxSize().background(Color.Black)) {
        val info = data ?: return@Box
        info.details.hero?.let { ArtImage(it, Modifier.fillMaxSize(), ContentScale.Crop) }
        Box(Modifier.fillMaxSize().background(Color.Black.copy(alpha = SCRIM_ALPHA)))
        Column(Modifier.fillMaxSize().systemBarsPadding().padding(24.dp), verticalArrangement = Arrangement.spacedBy(14.dp)) {
            InfoTabRow(app, memory, podTabs)
            Box(Modifier.weight(1f).fillMaxWidth()) {
                when (val tab = tabs.selected) {
                    0 -> InfoTabBody(app, screen, info, badges, memory) { viewer = it }
                    1 -> FramedTab { ActivityTab(app, game, memory) }
                    else -> podTabs.getOrNull(tab - InfoTabs.FIXED_TABS)?.let { item -> FramedTab { PodTabContent(app, item, game) } }
                }
            }
        }
        viewer?.let { FullScreenViewer(it, info.details.video, info.details.screenshots) { viewer = null } }
        ActivityDialogs(app, game)
    }
    val firstKey = when (tabs.selected) {
        0 -> PLAY_KEY
        1 -> ACTIVITY_FIRST_KEY
        else -> tabKey(tabs.selected)
    }
    FocusRestorer(memory, app.resumeTick, data != null && viewer == null && tabs.dialog == null, firstKey)
    FollowTab(memory, tabs.selected)
}

/** The Info tab: the text column at the left, the trailer and screenshots at the right. */
@Composable
private fun InfoTabBody(
    app: AppViewModel,
    screen: Screen.GameInfo,
    info: InfoData,
    badges: List<GameBadgeItem>,
    memory: FocusMemory,
    onView: (Viewer) -> Unit,
) {
    Row(Modifier.fillMaxSize(), horizontalArrangement = Arrangement.spacedBy(24.dp)) {
        InfoText(app, screen, info, badges, Modifier.fillMaxWidth(TEXT_SHARE).fillMaxHeight(), memory)
        InfoMedia(info.details.video, info.details.screenshots, onView, Modifier.fillMaxWidth())
    }
}

/** The launcher's frame around the content of a tab that is not the Info one: a themed panel. */
@Composable
private fun FramedTab(content: @Composable () -> Unit) {
    ZPanel(Modifier.fillMaxSize(), RoundedCornerShape(FRAME_RADIUS)) {
        Box(Modifier.fillMaxSize().padding(FRAME_PADDING)) { content() }
    }
}

/** Focus key of the tab at [index]. */
internal fun tabKey(index: Int): String = "info:tab:" + index

/** After the open tab changed with LB or RB, the focus goes to its pill: the content it was in is gone. */
@Composable
private fun FollowTab(memory: FocusMemory, selected: Int) {
    var seen by remember { mutableIntStateOf(selected) }
    LaunchedEffect(selected) {
        if (selected == seen) return@LaunchedEffect
        seen = selected
        memory.requesterOf(tabKey(selected)).requestSoon()
    }
}

/** The row of tabs. Entering it from the content lands on the open tab, so moving up never changes tab by itself. */
@OptIn(ExperimentalComposeUiApi::class)
@Composable
private fun InfoTabRow(app: AppViewModel, memory: FocusMemory, podTabs: List<GameTabItem>) {
    val tabs = app.infoTabs
    val labels = listOf(stringResource(R.string.info_tab_info), stringResource(R.string.info_tab_activity)) + podTabs.map { it.label }
    val row = Modifier.focusProperties { enter = { memory.requesterOf(tabKey(tabs.selected)) } }.focusGroup()
    TabStrip(labels, tabs.selected, tabs::select, row, tabModifier = { memory.track(tabKey(it)) })
}

/** The content a Pod draws in its tab, on a host over the app; the Pod counts as open for the crash check (§10). */
@Composable
private fun PodTabContent(app: AppViewModel, item: GameTabItem, game: GameEntry) {
    val entry = app.zl.pods.findEntry(item.podId) ?: return
    val theme = LocalZTheme.current
    val host = remember(theme, item.podId) { AppPodHost(item.podId, entry.permissions, app, theme, entry.i18nDir) }
    PodAlive(app, host, item.podId)
    item.content(host, game)
}

/** Digits of a year at the start of a release date. */
private const val YEAR_DIGITS = 4

/** Focus key of the Play button. */
private const val PLAY_KEY = "info:play"

/** The short texts the Pods put on the sheet (RF-79), as small rounded badges. */
@Composable
private fun BadgeRow(badges: List<GameBadgeItem>) {
    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        for (badge in badges) {
            Text(
                badge.text,
                color = Color.White,
                fontSize = 13.sp,
                fontWeight = FontWeight.SemiBold,
                maxLines = 1,
                modifier = Modifier.background(Color.White.copy(alpha = BADGE_ALPHA), RoundedCornerShape(50)).padding(horizontal = 10.dp, vertical = 3.dp),
            )
        }
    }
}

/** The text column: title, facts, play time, description and the actions. */
@Composable
private fun InfoText(
    app: AppViewModel,
    screen: Screen.GameInfo,
    info: InfoData,
    badges: List<GameBadgeItem>,
    modifier: Modifier,
    memory: FocusMemory,
) {
    val context = LocalContext.current
    val d = info.details
    var currentRating by remember(d.id) { mutableStateOf(d.rating) }
    val displayDetails = remember(d, currentRating) { d.copy(rating = currentRating) }
    Column(modifier, verticalArrangement = Arrangement.spacedBy(10.dp)) {
        if (d.logo != null) {
            ArtImage(d.logo, Modifier.fillMaxWidth().height(110.dp), ContentScale.Fit)
        } else {
            Text(d.displayTitle, color = Color.White, fontSize = 30.sp, fontWeight = FontWeight.Bold, maxLines = 2)
        }
        factsLine(displayDetails)?.let { Text(it, color = Color.White, fontSize = 17.sp, fontWeight = FontWeight.SemiBold) }
        Text(playTimeLine(context, info.stats), color = Color.White.copy(alpha = 0.8f), fontSize = 15.sp)
        if (badges.isNotEmpty()) BadgeRow(badges)
        Column(Modifier.weight(1f).verticalScroll(rememberScrollState())) {
            Text(
                d.description?.trim()?.ifEmpty { null } ?: stringResource(R.string.info_no_description),
                color = Color.White.copy(alpha = 0.85f),
                fontSize = 15.sp,
                overflow = TextOverflow.Ellipsis,
            )
        }
        Actions(app, screen, currentRating, { currentRating = it }, memory)
    }
}

/** Play, favourite, rating and (for games run by an emulator) the emulator of this game. */
@Composable
private fun Actions(
    app: AppViewModel,
    screen: Screen.GameInfo,
    rating: Float?,
    onRatingChange: (Float?) -> Unit,
    memory: FocusMemory,
) {
    val game = screen.game
    var favorite by remember(game.id) { mutableStateOf(game.favorite) }
    Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
        PadButton(stringResource(R.string.menu_launch), { app.launchGame(game) }, memory.track(PLAY_KEY))
        PadButton(stringResource(if (favorite) R.string.menu_favorite_remove else R.string.menu_favorite_add), {
            favorite = !favorite
            app.library.setFavorite(game, favorite)
        }, memory.track("info:favorite"))
        val ratingText = rating?.let { "★ %.0f".format(it) } ?: stringResource(R.string.menu_rating)
        PadButton(ratingText, {
            val next = when (rating?.toInt()) {
                5 -> 4.0f
                4 -> 3.0f
                3 -> 2.0f
                2 -> 1.0f
                1 -> null
                else -> 5.0f
            }
            onRatingChange(next)
            app.library.setRating(game.id, next)
        }, memory.track("info:rating"))
        if (game.appPackage == null) {
            PadButton(stringResource(R.string.info_emulator_game), { app.chooseGameEmulator(game) }, memory.track("info:emulator"))
        }
    }
}

/** "2011 · HAL Laboratory · Nintendo · Platformer · ★ 4.5" from the fields that are known. */
private fun factsLine(d: GameDetails): String? {
    val year = d.releaseDate?.take(YEAR_DIGITS)?.takeIf { it.length == YEAR_DIGITS && it.all(Char::isDigit) }
    val rating = d.rating?.let { "★ %.1f".format(it) }
    val parts = listOfNotNull(year, d.developer, d.publisher?.takeIf { it != d.developer }, d.genres, rating)
        .map { it.trim() }.filter { it.isNotEmpty() }
    return parts.takeIf { it.isNotEmpty() }?.joinToString(" · ")
}
