package dev.zlauncher.pods.work

import android.content.Context
import android.content.Intent
import androidx.core.content.ContextCompat
import dev.zlauncher.pods.PodWork
import dev.zlauncher.pods.PodWorkProgress
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import java.util.concurrent.ConcurrentHashMap

/** Least time between two notification updates of the same tasks, so a chatty task does not flood the system. */
private const val NOTIFY_EVERY_MS = 500L

/**
 * Runs the long tasks of the Pods for the whole process: one coroutine per task, the foreground
 * service ([PodWorkService]) while any runs, its notification kept up to date. A task that throws
 * is reported to [onCrash] (it counts as a crash of its Pod, §10).
 */
object PodWorkRunner {
    /** The tasks' coroutines: one failing does not stop the others. */
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)

    /** What runs and what it reported. */
    val book = PodWorkBook()

    /** The coroutine of each running task, by key. */
    private val jobs = ConcurrentHashMap<String, Job>()

    /** When the notification was last updated. */
    @Volatile
    private var notifiedAt = 0L

    /** The work service of each Pod, made once: every screen of the Pod shares it. */
    private val services = ConcurrentHashMap<String, PodWork>()

    /** The work service of Pod [podId]; [onCrash] gets a task's failure. */
    fun forPod(context: Context, podId: String, onCrash: (Throwable) -> Unit): PodWork =
        services.getOrPut(podId) { serviceOf(context, podId, onCrash) }

    /** A new work service of Pod [podId]. */
    private fun serviceOf(context: Context, podId: String, onCrash: (Throwable) -> Unit): PodWork = object : PodWork {
        override val running: StateFlow<Set<String>> =
            book.running.map { book.runningOf(podId, it) }.stateIn(scope, SharingStarted.Eagerly, book.runningOf(podId, book.running.value))

        override fun start(id: String, title: String, task: suspend (PodWorkProgress) -> Unit): Boolean =
            launch(context.applicationContext, PodWorkBook.keyOf(podId, id), title, onCrash, task)

        override fun cancel(id: String) {
            jobs[PodWorkBook.keyOf(podId, id)]?.cancel()
        }
    }

    /** Starts the task [key] with the service; `false` when it already runs. */
    private fun launch(context: Context, key: String, title: String, onCrash: (Throwable) -> Unit, task: suspend (PodWorkProgress) -> Unit): Boolean {
        if (!book.add(key, title)) return false
        ContextCompat.startForegroundService(context, Intent(context, PodWorkService::class.java))
        val progress = PodWorkProgress { text, percent ->
            book.progress(key, text, percent)
            notifySoon(context)
        }
        jobs[key] = scope.launch {
            try {
                task(progress)
            } catch (e: CancellationException) {
                throw e
            } catch (@Suppress("TooGenericExceptionCaught") e: Exception) {
                onCrash(e)
            } finally {
                jobs.remove(key)
                book.remove(key)
                PodWorkService.refresh(context)
            }
        }
        return true
    }

    /** Updates the notification, at most every [NOTIFY_EVERY_MS]. */
    private fun notifySoon(context: Context) {
        val now = System.currentTimeMillis()
        if (now - notifiedAt < NOTIFY_EVERY_MS) return
        notifiedAt = now
        PodWorkService.refresh(context)
    }
}
