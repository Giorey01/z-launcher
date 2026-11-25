package dev.zlauncher.ui.theme

import android.content.Context
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalLifecycleOwner
import androidx.compose.ui.viewinterop.AndroidView
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.media3.common.C
import androidx.media3.common.MediaItem
import androidx.media3.common.Player
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.ui.AspectRatioFrameLayout
import androidx.media3.ui.PlayerView
import coil.compose.AsyncImage
import coil.decode.BitmapFactoryDecoder
import coil.request.ImageRequest
import dev.zlauncher.core.nativecore.Wallpaper
import dev.zlauncher.core.nativecore.WallpaperKind
import java.io.File

/** The still wallpaper behind the panels of a screen, which glass panels blur; `null` when there is none or it is a video. */
val LocalBackdrop = compositionLocalOf<Wallpaper?> { null }

/**
 * A screen's background: the [wallpaper] (image, animated GIF/WebP or looping muted video), or the
 * theme's background gradient when there is none. [content] is drawn on top and may use glass panels,
 * which blur this wallpaper.
 */
@Composable
fun WallpaperScreen(wallpaper: Wallpaper?, modifier: Modifier = Modifier, content: @Composable BoxScope.() -> Unit) {
    val theme = LocalZTheme.current
    val still = wallpaper?.takeIf { it.kind != WallpaperKind.VIDEO }
    Box(modifier.fillMaxSize().background(theme.backgroundBrush())) {
        when (wallpaper?.kind) {
            null -> Unit
            WallpaperKind.VIDEO -> VideoWallpaper(wallpaper.path)
            else -> WallpaperImage(wallpaper.path, animated = true)
        }
        CompositionLocalProvider(LocalBackdrop provides still) { this@Box.content() }
    }
}

/** A wallpaper picture; [animated] false decodes only the first frame of a GIF (used for the blurred copy). */
@Composable
fun WallpaperImage(path: String, animated: Boolean, modifier: Modifier = Modifier.fillMaxSize()) {
    val context = LocalContext.current
    val request = remember(path, animated) {
        ImageRequest.Builder(context).data(File(path)).apply { if (!animated) decoderFactory(BitmapFactoryDecoder.Factory()) }.build()
    }
    AsyncImage(request, null, modifier, contentScale = ContentScale.Crop)
}

/** A looping, muted video that plays only while the screen is started (low power: paused when hidden). */
@Composable
private fun VideoWallpaper(path: String) {
    val context = LocalContext.current
    val lifecycle = LocalLifecycleOwner.current.lifecycle
    var player by remember { mutableStateOf<ExoPlayer?>(null) }
    DisposableEffect(path) {
        val exo = newMutedPlayer(context, path)
        player = exo
        val observer = LifecycleEventObserver { _, event ->
            val isStarted = lifecycle.currentState.isAtLeast(Lifecycle.State.STARTED)
            exo.playWhenReady = isStarted && event != Lifecycle.Event.ON_STOP
        }
        lifecycle.addObserver(observer)
        onDispose {
            lifecycle.removeObserver(observer)
            exo.release()
            player = null
        }
    }
    AndroidView(
        factory = { PlayerView(it).apply { useController = false; resizeMode = AspectRatioFrameLayout.RESIZE_MODE_ZOOM } },
        modifier = Modifier.fillMaxSize(),
        update = { it.player = player },
    )
}

/** An ExoPlayer that loops [path] without sound. */
private fun newMutedPlayer(context: Context, path: String): ExoPlayer = ExoPlayer.Builder(context).build().apply {
    setMediaItem(MediaItem.fromUri(android.net.Uri.fromFile(File(path))))
    repeatMode = Player.REPEAT_MODE_ALL
    volume = 0f
    videoScalingMode = C.VIDEO_SCALING_MODE_SCALE_TO_FIT_WITH_CROPPING
    prepare()
    playWhenReady = true
}
