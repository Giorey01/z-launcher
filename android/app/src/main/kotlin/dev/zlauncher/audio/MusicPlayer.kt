package dev.zlauncher.audio

import android.content.Context
import android.media.AudioAttributes
import android.media.AudioFocusRequest
import android.media.AudioManager
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
 * visible ([setVisible]) and no game or app started from the launcher is in front ([leave], until a
 * home screen is resumed: on a dual-screen device the other screen stays visible under the game).
 * It also yields the audio focus: an app that takes it (an emulator) silences the music.
 */
class MusicPlayer(context: Context) {
    /** The system's audio, for the focus. */
    private val audio = context.getSystemService(AudioManager::class.java)

    /** The focus request of the music; losing the focus pauses it. */
    private val focus = AudioFocusRequest.Builder(AudioManager.AUDIOFOCUS_GAIN)
        .setAudioAttributes(attributes())
        .setOnAudioFocusChangeListener { change -> onFocus(change) }
        .build()

    /** A game or an app started from the launcher is in front. */
    private var away = false

    /** Another app holds the audio focus. */
    private var focusLost = false

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
        refresh()
    }

    /** A game or an app is being started from the launcher: silence until [back]. */
    @Synchronized
    fun leave() {
        away = true
        refresh()
    }

    /** A home screen of the launcher is in front again. */
    @Synchronized
    fun back() {
        if (!away) return
        away = false
        focusLost = false
        refresh()
    }

    /** Whether the music may play now. */
    private fun audible() = visible && !away && !focusLost

    /** Plays or pauses as [audible] says; the scheduled track may have changed while silent. */
    private fun refresh() {
        if (!audible()) {
            player?.takeIf { it.isPlaying }?.pause()
            audio.abandonAudioFocusRequest(focus)
            return
        }
        val fresh = queueNow()
        if (fresh != queue || player == null) {
            queue = fresh
            index = 0
            restart()
        } else {
            requestFocus()
            player?.start()
        }
    }

    /** The audio focus changed: another app took it (pause) or gave it back (play again). */
    @Synchronized
    private fun onFocus(change: Int) {
        focusLost = change != AudioManager.AUDIOFOCUS_GAIN
        refresh()
    }

    /** Asks the audio focus; refused, the music stays silent. */
    private fun requestFocus(): Boolean = audio.requestAudioFocus(focus) == AudioManager.AUDIOFOCUS_REQUEST_GRANTED

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
        if (!audible() || volume == 0f || !requestFocus()) return
        player = runCatching { open(path) }.onFailure { Log.w(TAG, "cannot play $path", it) }.getOrNull()
    }

    /** A player on [path], started; at its end the next track follows. */
    private fun open(path: String): MediaPlayer = MediaPlayer().apply {
        setAudioAttributes(attributes())
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

/** How the music plays: on the media volume, as music. */
private fun attributes(): AudioAttributes =
    AudioAttributes.Builder().setUsage(AudioAttributes.USAGE_GAME).setContentType(AudioAttributes.CONTENT_TYPE_MUSIC).build()

/** Minutes in an hour, for the minute of the day. */
private const val MINUTES_PER_HOUR = 60
