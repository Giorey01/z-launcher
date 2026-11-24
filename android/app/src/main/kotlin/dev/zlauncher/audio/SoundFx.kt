package dev.zlauncher.audio

import android.media.AudioAttributes
import android.media.SoundPool
import dev.zlauncher.core.nativecore.SoundAction
import dev.zlauncher.ui.theme.ZTheme

/** How many sounds may play at once (navigation clicks can come fast). */
private const val MAX_STREAMS = 4

/** Priority of every sound in the pool. */
private const val PRIORITY = 1

/**
 * The interface sounds of the active theme, preloaded in a [SoundPool]. Only actions the theme (or its
 * sound pack) has a file for make a sound; with no file, or with effects off, [play] does nothing.
 */
class SoundFx {
    /** The pool the files are decoded into. */
    private val pool = SoundPool.Builder()
        .setMaxStreams(MAX_STREAMS)
        .setAudioAttributes(
            AudioAttributes.Builder()
                .setUsage(AudioAttributes.USAGE_ASSISTANCE_SONIFICATION)
                .setContentType(AudioAttributes.CONTENT_TYPE_SONIFICATION)
                .build()
        )
        .build()

    /** Pool sample id of each loaded action. */
    private val samples = HashMap<SoundAction, Int>()

    /** Source path each loaded sample came from, to skip reloading unchanged files. */
    private val sources = HashMap<SoundAction, String>()

    /** Whether effects are enabled. */
    private var enabled = true

    /** Playback volume, 0 to 1. */
    private var volume = 1f

    /** Applies the settings of [theme] and loads the files it lists, unloading those no longer used. */
    fun apply(theme: ZTheme) {
        enabled = theme.settings.sfxEnabled
        volume = theme.settings.sfxVolume.coerceIn(0f, 1f)
        val wanted = theme.sounds.mapNotNull { entry -> entry.path?.let { entry.action to it } }.toMap()
        for (action in samples.keys.toList()) if (wanted[action] != sources[action]) unload(action)
        for ((action, path) in wanted) if (action !in samples) load(action, path)
    }

    /** Plays the sound of [action] if there is one. */
    fun play(action: SoundAction) {
        if (!enabled) return
        samples[action]?.let { pool.play(it, volume, volume, PRIORITY, 0, 1f) }
    }

    /** Frees the pool. */
    fun release() = pool.release()

    /** Loads one file into the pool. */
    private fun load(action: SoundAction, path: String) {
        samples[action] = pool.load(path, PRIORITY)
        sources[action] = path
    }

    /** Removes one sample from the pool. */
    private fun unload(action: SoundAction) {
        samples.remove(action)?.let(pool::unload)
        sources.remove(action)
    }
}
