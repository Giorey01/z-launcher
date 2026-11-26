package dev.zlauncher.pods.work

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

/** One running task: its title and the last progress it reported. */
data class WorkTask(
    /** Shown as the notification's title. */
    val title: String,
    /** The line under the title, null until the task says one. */
    val text: String? = null,
    /** 0–100, null for indeterminate. */
    val percent: Int? = null,
)

/** What the notification shows of the running tasks. */
data class WorkSummary(
    /** The title: the first task's. */
    val title: String,
    /** The line: the first task's text. */
    val text: String?,
    /** The first task's progress. */
    val percent: Int?,
    /** How many tasks run. */
    val count: Int,
)

/**
 * The running tasks of every Pod, by key (`<pod id>:<task id>`): which run, what they last
 * reported, and what the notification shows. No Android here; [PodWorkRunner] runs the tasks and
 * the service around this book.
 */
class PodWorkBook {
    /** Tasks in the order they started. */
    private val tasks = LinkedHashMap<String, WorkTask>()

    /** Keys of the running tasks. */
    private val _running = MutableStateFlow<Set<String>>(emptySet())

    /** Keys of the running tasks, as they change. */
    val running: StateFlow<Set<String>> = _running.asStateFlow()

    /** Adds a task; `false` when one with [key] already runs. */
    @Synchronized
    fun add(key: String, title: String): Boolean {
        if (key in tasks) return false
        tasks[key] = WorkTask(title)
        publish()
        return true
    }

    /** Records what the task [key] reports; ignored when it is not running. */
    @Synchronized
    fun progress(key: String, text: String?, percent: Int?) {
        val task = tasks[key] ?: return
        tasks[key] = task.copy(text = text, percent = percent?.coerceIn(0, MAX_PERCENT))
    }

    /** The task [key] ended (done, failed or cancelled). */
    @Synchronized
    fun remove(key: String) {
        if (tasks.remove(key) != null) publish()
    }

    /** What the notification shows, null when nothing runs. */
    @Synchronized
    fun summary(): WorkSummary? {
        val first = tasks.values.firstOrNull() ?: return null
        return WorkSummary(first.title, first.text, first.percent, tasks.size)
    }

    /** The keys of one Pod's running tasks, without the Pod's prefix. */
    fun runningOf(podId: String, keys: Set<String>): Set<String> {
        val prefix = keyOf(podId, "")
        return keys.filter { it.startsWith(prefix) }.map { it.removePrefix(prefix) }.toSet()
    }

    /** Publishes the keys to [running]. */
    private fun publish() {
        _running.value = tasks.keys.toSet()
    }

    /** The key of a Pod's task and the progress bound. */
    companion object {
        /** Top of the progress scale. */
        const val MAX_PERCENT = 100

        /** The key of task [id] of Pod [podId]: tasks of different Pods never meet. */
        fun keyOf(podId: String, id: String): String = "$podId:$id"
    }
}
