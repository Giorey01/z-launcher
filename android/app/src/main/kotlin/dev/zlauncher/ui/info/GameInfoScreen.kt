package dev.zlauncher.ui.info

import androidx.compose.foundation.background
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
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
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
import dev.zlauncher.core.nativecore.PlayStats
import dev.zlauncher.launch.toMessage
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.Screen
import dev.zlauncher.ui.art.ArtImage
import dev.zlauncher.ui.pad.FocusRestorer
import dev.zlauncher.ui.pad.PadButton

/** Opacity of the dark layer over the hero image, so the text reads over any picture. */
private const val SCRIM_ALPHA = 0.72f

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
 * The Info tab of a game (RF-30): the hero image behind, at the left the logo or title, the facts, the play time,
 * the description and the actions (Play, favourite, the emulator for this game); at the right the trailer, streamed
 * and muted, and the screenshots. A on the trailer or a screenshot opens it full screen; B goes back.
 */
@Composable
fun GameInfoScreen(app: AppViewModel, screen: Screen.GameInfo) {
    val game = screen.game
    val data by produceState<InfoData?>(null, game.id, app.resumeTick) {
        value = try {
            app.zl.core.call { InfoData(it.gameDetails(game.id), it.playStats(game.id)) }
        } catch (e: CoreException) {
            app.message = e.toMessage()
            null
        }
    }
    var viewer by remember { mutableStateOf<Viewer?>(null) }
    val memory = app.memory("info:${game.id}")
    Box(Modifier.fillMaxSize().background(Color.Black)) {
        val info = data ?: return@Box
        info.details.hero?.let { ArtImage(it, Modifier.fillMaxSize(), ContentScale.Crop) }
        Box(Modifier.fillMaxSize().background(Color.Black.copy(alpha = SCRIM_ALPHA)))
        Row(Modifier.fillMaxSize().systemBarsPadding().padding(24.dp), horizontalArrangement = Arrangement.spacedBy(24.dp)) {
            InfoText(app, screen, info, Modifier.fillMaxWidth(TEXT_SHARE).fillMaxHeight(), memory)
            InfoMedia(info.details.video, info.details.screenshots, { viewer = it }, Modifier.fillMaxWidth())
        }
        viewer?.let { FullScreenViewer(it, info.details.video, info.details.screenshots) { viewer = null } }
    }
    FocusRestorer(memory, app.resumeTick, data != null && viewer == null, PLAY_KEY)
}

/** Digits of a year at the start of a release date. */
private const val YEAR_DIGITS = 4

/** Focus key of the Play button. */
private const val PLAY_KEY = "info:play"

/** The text column: title, facts, play time, description and the actions. */
@Composable
private fun InfoText(app: AppViewModel, screen: Screen.GameInfo, info: InfoData, modifier: Modifier, memory: dev.zlauncher.ui.pad.FocusMemory) {
    val context = LocalContext.current
    val d = info.details
    Column(modifier, verticalArrangement = Arrangement.spacedBy(10.dp)) {
        if (d.logo != null) {
            ArtImage(d.logo, Modifier.fillMaxWidth().height(110.dp), ContentScale.Fit)
        } else {
            Text(d.displayTitle, color = Color.White, fontSize = 30.sp, fontWeight = FontWeight.Bold, maxLines = 2)
        }
        factsLine(d)?.let { Text(it, color = Color.White, fontSize = 17.sp, fontWeight = FontWeight.SemiBold) }
        Text(playTimeLine(context, info.stats), color = Color.White.copy(alpha = 0.8f), fontSize = 15.sp)
        Column(Modifier.weight(1f).verticalScroll(rememberScrollState())) {
            Text(
                d.description?.trim()?.ifEmpty { null } ?: stringResource(R.string.info_no_description),
                color = Color.White.copy(alpha = 0.85f),
                fontSize = 15.sp,
                overflow = TextOverflow.Ellipsis,
            )
        }
        Actions(app, screen, memory)
    }
}

/** Play, favourite and (for games run by an emulator) the emulator of this game. */
@Composable
private fun Actions(app: AppViewModel, screen: Screen.GameInfo, memory: dev.zlauncher.ui.pad.FocusMemory) {
    val game = screen.game
    var favorite by remember(game.id) { mutableStateOf(game.favorite) }
    Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
        PadButton(stringResource(R.string.menu_launch), { app.launchGame(game) }, memory.track(PLAY_KEY))
        PadButton(stringResource(if (favorite) R.string.menu_favorite_remove else R.string.menu_favorite_add), {
            favorite = !favorite
            app.library.setFavorite(game, favorite)
        }, memory.track("info:favorite"))
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
