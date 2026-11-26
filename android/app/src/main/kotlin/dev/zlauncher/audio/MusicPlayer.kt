package dev.zlauncher.audio

import android.media.AudioAttributes
import android.media.MediaPlayer
import android.util.Log
import dev.zlauncher.core.nativecore.MusicSetup
import dev.zlauncher.core.nativecore.musicQueue
import java.time.LocalTime

/** Log tag of the background music. */
private const val TAG = "ZMusic"

/**
 * The background music of the active theme. The core says which tracks play now
 * ([musicQueue]: the playlist, or the scheduled track); this plays them one after the other and
 * starts again, at the music volume of the settings. It only plays while a launcher screen is
 * visible ([setVisible]): a game started from the launcher is not covered by the music.
 */
class MusicPlayer {
    /** The setup of the theme, with the user's mode. */
    private var setup: MusicSetup? = null

    /** Playback volume, 0 to 1. */
    private var volume = 0f

    /** Whether a launcher screen is visible. */
    private var visible = false

    /** The tracks being played, and the index of the current one. */
    private var queue: List<String> = emptyList()
    private var index = 0

    /** The player of the current track, `null` while silent. */
    private var player: MediaPlayer? = null

    /** Seed of the shuffle, one per process so a session keeps its order. */
    private val seed = System.nanoTime().toULong()

    /** Applies the theme's music and volume; the queue starts again only if it changed. */
    @Synchronized
    fun apply(music: MusicSetup, musicVolume: Float) {
        setup = music
        volume = musicVolume.coerceIn(0f, 1f)
        player?.setVolume(volume, volume)
        val next = queueNow()
        if (next != queue) {
            queue = next
            index = 0
            restart()
        }
    }

    /** A launcher screen became visible (true) or none is (false). */
    @Synchronized
    fun setVisible(now: Boolean) {
        if (now == visible) return
        visible = now
        if (now) {
            // The scheduled track may have changed while away.
            queueNow().takeIf { it != queue }?.let { queue = it; index = 0; restart() } ?: player?.start() ?: restart()
        } else {
            player?.takeIf { it.isPlaying }?.pause()
        }
    }

    /** The tracks the core says to play at this minute. */
    private fun queueNow(): List<String> {
        val music = setup ?: return emptyList()
        val now = LocalTime.now()
        return musicQueue(music, (now.hour * MINUTES_PER_HOUR + now.minute).toUInt(), seed)
    }

    /** Stops the current track and, when there is something to play and a screen to play it for, plays the current one. */
    private fun restart() {
        player?.release()
        player = null
        val path = queue.getOrNull(index) ?: return
        if (!visible || volume == 0f) return
        player = runCatching { open(path) }.onFailure { Log.w(TAG, "cannot play $path", it) }.getOrNull()
    }

    /** A player on [path], started; at its end the next track follows. */
    private fun open(path: String): MediaPlayer = MediaPlayer().apply {
        setAudioAttributes(
            AudioAttributes.Builder().setUsage(AudioAttributes.USAGE_GAME).setContentType(AudioAttributes.CONTENT_TYPE_MUSIC).build(),
        )
        setDataSource(path)
        // A single track loops; a list goes to the next one.
        isLooping = queue.size == 1
        setVolume(volume, volume)
        setOnCompletionListener { next() }
        prepare()
        start()
    }

    /** Goes to the next track (in time mode the core may now give another one). */
    @Synchronized
    private fun next() {
        val fresh = queueNow()
        if (fresh != queue) {
            queue = fresh
            index = 0
        } else {
            index = (index + 1) % queue.size.coerceAtLeast(1)
        }
        restart()
    }
}

/** Minutes in an hour, for the minute of the day. */
private const val MINUTES_PER_HOUR = 60
