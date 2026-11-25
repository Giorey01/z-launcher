package dev.zlauncher.ui.settings

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import dev.zlauncher.R
import dev.zlauncher.core.nativecore.ImageScrapeReport
import dev.zlauncher.core.nativecore.ScrapeListener
import dev.zlauncher.core.nativecore.ScrapeStop
import dev.zlauncher.nav.UiMessage
import java.util.concurrent.atomic.AtomicBoolean

/**
 * The state of one SteamGridDB art download: the games done so far and the cancel request. The core calls
 * [onGame] and [isCancelled] from its worker thread; Compose state may be written from there.
 */
class ArtDownload : ScrapeListener {
    /** Games finished so far, `null` before the first one. */
    var done by mutableStateOf<Pair<Int, Int>?>(null)
        private set

    /** Set when the user asked to stop. */
    private val cancel = AtomicBoolean(false)

    /** Asks the run to stop after the current game. */
    fun requestCancel() = cancel.set(true)

    /** Progress from the core. */
    override fun onGame(done: UInt, total: UInt) {
        this.done = done.toInt() to total.toInt()
    }

    /** Whether the run should stop. */
    override fun isCancelled(): Boolean = cancel.get()
}

/** The message that sums up a finished download: how it ended, then the numbers. */
fun ImageScrapeReport.toMessage(): UiMessage = when (stopped) {
    ScrapeStop.MISSING_KEY -> UiMessage(R.string.msg_sgdb_need_key)
    ScrapeStop.KEY_REFUSED -> UiMessage(R.string.msg_sgdb_key_refused)
    ScrapeStop.OFFLINE -> UiMessage(R.string.msg_sgdb_offline)
    ScrapeStop.CANCELLED -> UiMessage(R.string.msg_sgdb_cancelled, listOf(images.toString()))
    null -> UiMessage(
        R.string.msg_sgdb_result,
        listOf(images.toString(), updated.toString(), notFound.toString()),
    )
}

/** The message that sums up a finished run of Steam store details: how it ended, then the numbers. */
fun ImageScrapeReport.toSteamDetailsMessage(): UiMessage = when (stopped) {
    ScrapeStop.OFFLINE -> UiMessage(R.string.msg_steam_store_offline, listOf(updated.toString()))
    ScrapeStop.CANCELLED -> UiMessage(R.string.msg_steam_details_cancelled, listOf(updated.toString()))
    ScrapeStop.MISSING_KEY, ScrapeStop.KEY_REFUSED, null -> UiMessage(
        R.string.msg_steam_details_result,
        listOf(updated.toString(), images.toString(), notFound.toString()),
    )
}
