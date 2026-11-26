package dev.zlauncher.ui.theme

import dev.zlauncher.audio.MusicPlayer
import dev.zlauncher.audio.SoundFx
import dev.zlauncher.core.CoreProvider
import dev.zlauncher.core.nativecore.CoreException
import dev.zlauncher.ui.art.ArtResolver
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

/**
 * Holds the [ZTheme] every screen on every display draws with. [reload] asks the core for the active
 * theme again (after the user changed a setting, applied a theme or imported one), drops the art caches
 * and refreshes the sounds and the music.
 */
class ThemeStore(private val core: CoreProvider, private val art: ArtResolver, private val sound: SoundFx,
    private val music: MusicPlayer,
) {
    /** Backing state; `null` until the first load. */
    private val state = MutableStateFlow<ZTheme?>(null)

    /** The current theme, `null` until the first load. */
    val theme: StateFlow<ZTheme?> = state.asStateFlow()

    /** Counter stamped on every theme, so keyed caches notice a change. */
    private var revision = 0

    /** Reads the active theme; a core failure leaves the previous theme in place and is rethrown. */
    @Throws(CoreException::class)
    suspend fun reload() {
        val resolved = core.call { it.activeTheme() }
        art.reset()
        val next = resolved.toZTheme(++revision)
        sound.apply(next)
        music.apply(resolved.music, next.settings.musicVolume)
        state.value = next
    }
}
