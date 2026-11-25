package dev.zlauncher.ui.info

import android.content.Context
import android.net.Uri
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.viewinterop.AndroidView
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.compose.ui.platform.LocalLifecycleOwner
import androidx.media3.common.MediaItem
import androidx.media3.common.Player
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.ui.AspectRatioFrameLayout
import androidx.media3.ui.PlayerView
import java.io.File

/** A local file path or a web address as the URI the player opens. */
private fun uriOf(path: String): Uri =
    if (path.startsWith("https://") || path.startsWith("http://")) Uri.parse(path) else Uri.fromFile(File(path))

/** A player for [path] that loops; [sound] off plays it muted. */
private fun newPlayer(context: Context, path: String, sound: Boolean): ExoPlayer = ExoPlayer.Builder(context).build().apply {
    setMediaItem(MediaItem.fromUri(uriOf(path)))
    repeatMode = Player.REPEAT_MODE_ALL
    volume = if (sound) 1f else 0f
    prepare()
    playWhenReady = true
}

/**
 * A trailer: a local video or one streamed from the store (MP4 or HLS). The preview plays muted without controls;
 * [sound] and [controls] are for the full-screen view. It pauses while the screen is stopped and is released when
 * it leaves the composition.
 */
@Composable
fun TrailerPlayer(path: String, sound: Boolean, controls: Boolean, modifier: Modifier = Modifier) {
    val context = LocalContext.current
    val lifecycle = LocalLifecycleOwner.current.lifecycle
    var player by remember { mutableStateOf<ExoPlayer?>(null) }
    DisposableEffect(path, sound) {
        val exo = newPlayer(context, path, sound)
        player = exo
        val observer = LifecycleEventObserver { _, _ ->
            exo.playWhenReady = lifecycle.currentState.isAtLeast(Lifecycle.State.STARTED)
        }
        lifecycle.addObserver(observer)
        onDispose {
            lifecycle.removeObserver(observer)
            exo.release()
            player = null
        }
    }
    AndroidView(
        factory = { PlayerView(it).apply { resizeMode = AspectRatioFrameLayout.RESIZE_MODE_FIT } },
        modifier = modifier,
        update = {
            it.useController = controls
            it.player = player
        },
    )
}
