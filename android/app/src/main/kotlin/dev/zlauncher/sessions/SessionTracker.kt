package dev.zlauncher.sessions

import android.app.AppOpsManager
import android.app.usage.UsageEvents
import android.app.usage.UsageStatsManager
import android.content.Context
import android.os.Process
import dev.zlauncher.core.CoreProvider
import dev.zlauncher.core.nativecore.SessionEnd
import dev.zlauncher.core.nativecore.UsageEvent
import dev.zlauncher.core.nativecore.UsageKind
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

/** How far before the launch the usage events are read, so a resume logged a moment early is not missed. */
private const val EVENTS_MARGIN_MS = 60_000L

/**
 * Play time without a service running during the game (RF-40): the launch is noted in the core, and when the launcher
 * is in front again the usage events of the game's package are read from [UsageStatsManager] and handed to the core,
 * which counts the foreground time. Without usage access the core counts the launch-to-return time instead.
 */
class SessionTracker(private val context: Context, private val core: CoreProvider) {
    /** Both displays resume the launcher; only one of them may close the session. */
    private val closing = Mutex()

    /** Whether the user gave the launcher usage access (Settings, Special app access). */
    fun hasUsageAccess(): Boolean {
        val ops = context.getSystemService(AppOpsManager::class.java)
        val mode = ops.unsafeCheckOpNoThrow(AppOpsManager.OPSTR_GET_USAGE_STATS, Process.myUid(), context.packageName)
        return mode == AppOpsManager.MODE_ALLOWED
    }

    /** Notes that [gameId] was started through [packageName] now. */
    suspend fun begin(gameId: Long, packageName: String) {
        core.call { it.beginSession(gameId, packageName, System.currentTimeMillis()) }
    }

    /** Closes the pending session, if any, from the usage events of its package. */
    suspend fun end(): SessionEnd = closing.withLock {
        val pending = core.call { it.pendingSession() } ?: return@withLock SessionEnd.NoneOpen
        val now = System.currentTimeMillis()
        val events = if (hasUsageAccess()) read(pending.`package`, pending.startedMs - EVENTS_MARGIN_MS, now) else null
        core.call { it.endSession(events, now) }
    }

    /** The resume and pause events of [packageName]'s activities between [from] and [to]. */
    private fun read(packageName: String, from: Long, to: Long): List<UsageEvent> {
        val manager = context.getSystemService(UsageStatsManager::class.java)
        val events = manager.queryEvents(from, to)
        val event = UsageEvents.Event()
        val out = mutableListOf<UsageEvent>()
        while (events.hasNextEvent()) {
            events.getNextEvent(event)
            if (event.packageName != packageName) continue
            val kind = when (event.eventType) {
                UsageEvents.Event.ACTIVITY_RESUMED -> UsageKind.RESUMED
                UsageEvents.Event.ACTIVITY_PAUSED -> UsageKind.PAUSED
                else -> null
            }
            kind?.let { out += UsageEvent(event.timeStamp, it) }
        }
        return out
    }
}
