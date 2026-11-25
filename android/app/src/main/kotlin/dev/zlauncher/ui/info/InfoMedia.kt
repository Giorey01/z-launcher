package dev.zlauncher.ui.info

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.focusable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.zlauncher.R
import dev.zlauncher.ui.art.ArtImage
import dev.zlauncher.ui.pad.PadTile
import dev.zlauncher.ui.pad.TileSurface
import dev.zlauncher.ui.pad.requestSoon

/** Width over height of a trailer and of a screenshot. */
private const val WIDE = 16f / 9f

/** Height of a screenshot thumbnail. */
private val THUMB_HEIGHT = 96.dp

/** What is open over the game info, full screen. */
sealed interface Viewer {
    /** The screenshot at [index]. */
    data class Shot(
        /** Position in the list. */
        val index: Int,
    ) : Viewer

    /** The trailer, with sound and controls. */
    data object Trailer : Viewer
}

/**
 * The media of a game: the trailer playing muted (A opens it full screen with sound) and the screenshots in a row
 * (A opens one full screen). Nothing is drawn when the game has neither.
 */
@Composable
fun InfoMedia(video: String?, screenshots: List<String>, onOpen: (Viewer) -> Unit, modifier: Modifier = Modifier) {
    if (video == null && screenshots.isEmpty()) return
    Column(modifier, verticalArrangement = Arrangement.spacedBy(12.dp)) {
        if (video != null) {
            PadTile({ onOpen(Viewer.Trailer) }, Modifier.fillMaxWidth().aspectRatio(WIDE), surface = TileSurface.NONE) {
                TrailerPlayer(video, sound = false, controls = false, modifier = Modifier.fillMaxSize().clip(RoundedCornerShape(14.dp)))
                Text(
                    stringResource(R.string.info_trailer_hint),
                    Modifier.align(Alignment.BottomStart).padding(10.dp).background(Color.Black.copy(alpha = 0.6f), RoundedCornerShape(8.dp))
                        .padding(horizontal = 10.dp, vertical = 4.dp),
                    color = Color.White,
                    fontSize = 13.sp,
                )
            }
        }
        LazyRow(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            itemsIndexed(screenshots) { index, path ->
                PadTile({ onOpen(Viewer.Shot(index)) }, Modifier.height(THUMB_HEIGHT).aspectRatio(WIDE), surface = TileSurface.NONE) {
                    ArtImage(path, Modifier.fillMaxSize().clip(RoundedCornerShape(10.dp)), ContentScale.Crop)
                }
            }
        }
    }
}

/** The open [viewer] over everything: the trailer with sound, or a screenshot that left and right change. B closes it. */
@Composable
fun FullScreenViewer(viewer: Viewer, video: String?, screenshots: List<String>, onClose: () -> Unit) {
    BackHandler(onBack = onClose)
    Box(Modifier.fillMaxSize().background(Color.Black)) {
        when (viewer) {
            Viewer.Trailer -> video?.let { TrailerPlayer(it, sound = true, controls = true, modifier = Modifier.fillMaxSize()) }
            is Viewer.Shot -> ShotPager(screenshots, viewer.index)
        }
    }
}

/** One screenshot full screen; left and right show the previous and the next one. */
@Composable
private fun ShotPager(screenshots: List<String>, start: Int) {
    if (screenshots.isEmpty()) return
    var index by remember(start) { mutableIntStateOf(start.coerceIn(0, screenshots.lastIndex)) }
    val focus = remember { FocusRequester() }
    LaunchedEffect(Unit) { focus.requestSoon() }
    Box(
        Modifier.fillMaxSize().focusRequester(focus).focusable().onKeyEvent { event ->
            if (event.type != KeyEventType.KeyDown) return@onKeyEvent false
            when (event.key) {
                Key.DirectionLeft -> true.also { index = (index - 1).mod(screenshots.size) }
                Key.DirectionRight -> true.also { index = (index + 1).mod(screenshots.size) }
                else -> false
            }
        },
    ) {
        ArtImage(screenshots[index], Modifier.fillMaxSize(), ContentScale.Fit)
        Text("${index + 1} / ${screenshots.size}", Modifier.align(Alignment.BottomCenter).padding(16.dp), color = Color.White)
    }
}
