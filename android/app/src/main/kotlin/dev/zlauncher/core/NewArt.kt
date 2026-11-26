package dev.zlauncher.core

import dev.zlauncher.ZLauncherApp
import dev.zlauncher.ui.settings.ArtDownload
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/**
 * The art of the games a scan added: when the user set a SteamGridDB key, their missing cover,
 * hero and logo are downloaded in the background, slowly as the service asks. Only the games of
 * that scan are looked up (the core keeps when each game was added), so a game the service does
 * not know is not asked again at every scan. Without a key nothing happens.
 */
object NewArt {
    /** Now, in the Unix seconds the core keeps the time a game was added in: take it before the scan. */
    fun now(): Long = System.currentTimeMillis() / MILLIS_PER_SECOND

    /** After a scan started at [since] that added [added] games; [changed] runs on the main thread when art arrived. */
    fun afterScan(app: ZLauncherApp, since: Long, added: UInt, changed: () -> Unit = {}) {
        if (added == 0u) return
        CoroutineScope(Dispatchers.IO).launch {
            val report = runCatching { app.core.call { it.scrapeNewImages(since, ArtDownload()) } }.getOrNull()
            if ((report?.images ?: 0u) > 0u) withContext(Dispatchers.Main) { changed() }
        }
    }

    /** Milliseconds in a second. */
    private const val MILLIS_PER_SECOND = 1000L
}
