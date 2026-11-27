package dev.zlauncher.core

import dev.zlauncher.ZLauncherApp
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

/**
 * The checksums (CRC32, MD5, SHA1) of the ROM files, computed in the background by the core: after every scan the
 * files that are new or changed are read once, zip and 7z entries included, and CHD images are identified from
 * their header. The core runs the work on a thread of its own and ignores the call when a run is already going, so
 * calling this after each scan is cheap; with nothing to do it ends at once.
 */
object RomHashing {
    /** Starts the hashing of what the last scan left without checksums. */
    fun afterScan(app: ZLauncherApp) {
        CoroutineScope(Dispatchers.IO).launch {
            runCatching { app.core.call { it.startHashing(null) } }
        }
    }
}
