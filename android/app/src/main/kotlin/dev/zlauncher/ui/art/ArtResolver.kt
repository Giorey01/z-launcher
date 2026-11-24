package dev.zlauncher.ui.art

import dev.zlauncher.core.CoreProvider
import dev.zlauncher.core.nativecore.FolderArt
import dev.zlauncher.core.nativecore.OverlayArt
import java.util.concurrent.ConcurrentHashMap

/**
 * Asks the core for the theme art of a platform or special folder and remembers the answers until the
 * theme changes ([reset]). The core resolves a path through user replacements, the chosen packs and
 * the theme; Kotlin only draws what comes back.
 */
class ArtResolver(private val core: CoreProvider) {
    /** Platform folder art by platform id. */
    private val platforms = ConcurrentHashMap<String, FolderArt>()

    /** Special folder art by folder type. */
    private val folders = ConcurrentHashMap<String, FolderArt>()

    /** Cover mask and overlay by platform id. */
    private val overlays = ConcurrentHashMap<String, OverlayArt>()

    /** Forgets every answer (the theme, a pack or the user's art changed). */
    fun reset() {
        platforms.clear()
        folders.clear()
        overlays.clear()
    }

    /** Icon, hero and logo of the folder of [platformId]. */
    suspend fun platform(platformId: String): FolderArt =
        platforms[platformId] ?: core.call { it.platformArt(platformId) }.also { platforms[platformId] = it }

    /** Icon, hero and logo of a special folder (`favorites`, `emulators`, ...). */
    suspend fun folder(type: String): FolderArt =
        folders[type] ?: core.call { it.smartFolderArt(type) }.also { folders[type] = it }

    /** Mask and overlay for the covers of [platformId]. */
    suspend fun overlay(platformId: String): OverlayArt =
        overlays[platformId] ?: core.call { it.iconOverlay(platformId) }.also { overlays[platformId] = it }
}
