package dev.zlauncher.ui.nowplaying

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.ColorMatrix
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.NowPlaying
import dev.zlauncher.ui.art.ArtImage
import dev.zlauncher.ui.art.CoverArt
import dev.zlauncher.ui.home.rememberOverlay
import dev.zlauncher.ui.theme.LocalZTheme
import dev.zlauncher.ui.theme.WallpaperScreen
import kotlinx.coroutines.delay

/** Side of the cover on the now playing screen. */
private val COVER_SIZE = 300.dp

/** Opacity of the game's hero image behind the cover. */
private const val HERO_ALPHA = 0.35f

/** Duration of the fade into and out of the dimmed state, in milliseconds. */
private const val FADE_MS = 600

/** Milliseconds in a second. */
private const val MS_PER_SECOND = 1000L

/**
 * What a display shows while a game runs on another one: the game's cover and title over its hero
 * image (or the theme wallpaper). After the idle delay of the settings it darkens by the "idle screen
 * darkening" amount and loses colour down to the "idle colour saturation"; any touch brings it back.
 */
@Composable
fun NowPlayingScreen(app: AppViewModel, playing: NowPlaying) {
    val settings = LocalZTheme.current.settings
    var touches by remember { mutableIntStateOf(0) }
    LaunchedEffect(playing, touches, settings.nowPlayingIdleSeconds) {
        app.nowPlayingDimmed = false
        delay(settings.nowPlayingIdleSeconds.toLong() * MS_PER_SECOND)
        app.nowPlayingDimmed = true
    }
    val dimmed = app.nowPlayingDimmed
    val shade by animateFloatAsState(if (dimmed) settings.nowPlayingDimming else 0f, tween(FADE_MS), label = "shade")
    val saturation by animateFloatAsState(if (dimmed) settings.nowPlayingSaturation else 1f, tween(FADE_MS), label = "sat")
    val filter = remember(saturation) { ColorFilter.colorMatrix(ColorMatrix().apply { setToSaturation(saturation) }) }
    Box(Modifier.fillMaxSize().pointerInput(Unit) { wakeOnTouch { touches++ } }) {
        WallpaperScreen(LocalZTheme.current.wallpaperMain) {
            ArtImage(playing.game.hero, Modifier.fillMaxSize().alpha(HERO_ALPHA), ContentScale.Crop, filter)
            Cover(playing, filter, Modifier.align(Alignment.Center))
        }
        Box(Modifier.fillMaxSize().background(Color.Black.copy(alpha = shade)))
    }
}

/** Calls [onTouch] on every finger that goes down, without consuming it. */
private suspend fun androidx.compose.ui.input.pointer.PointerInputScope.wakeOnTouch(onTouch: () -> Unit) {
    awaitEachGesture {
        awaitFirstDown(requireUnconsumed = false, pass = PointerEventPass.Initial)
        onTouch()
    }
}

/** The framed cover with the title and the platform under it. */
@Composable
private fun Cover(playing: NowPlaying, filter: ColorFilter, modifier: Modifier) {
    val theme = LocalZTheme.current
    val game = playing.game
    Column(modifier.padding(24.dp), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(16.dp)) {
        CoverArt(game.icon, rememberOverlay(game.platformId, theme), theme.settings.iconRoundness, Modifier.size(COVER_SIZE), filter) {
            Box(Modifier.fillMaxSize().background(theme.cardBrush()))
        }
        Text(
            game.displayTitle,
            Modifier.fillMaxWidth(),
            color = theme.textPrimary,
            fontSize = 26.sp,
            fontWeight = FontWeight.SemiBold,
            textAlign = TextAlign.Center,
            maxLines = 2,
        )
        Text(game.platformName, color = theme.textSecondary, fontSize = 16.sp, textAlign = TextAlign.Center)
    }
}
