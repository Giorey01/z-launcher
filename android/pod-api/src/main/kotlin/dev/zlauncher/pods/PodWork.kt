package dev.zlauncher.pods

import kotlinx.coroutines.flow.StateFlow

/**
 * Long tasks of a Pod (permission `work`, §8): a download, an import. The launcher runs each in
 * its own process with a foreground service and a notification, so a task goes on when the Pod's
 * screen is closed. A Worker in the Pod's dex cannot be used for this: WorkManager creates
 * workers by class name with the launcher's class loader.
 *
 * A task that throws counts as a crash of the Pod (§10). Tasks do not survive the end of the
 * process: a Pod that needs it keeps its own queue and starts the task again when it is opened.
 */
interface PodWork {
    /** Ids of this Pod's tasks that are running. */
    val running: StateFlow<Set<String>>

    /**
     * Starts [task] under [id], shown in the notification as [title]; `false` (and nothing
     * starts) when a task with that id is already running.
     */
    fun start(id: String, title: String, task: suspend (PodWorkProgress) -> Unit): Boolean

    /** Cancels the task [id] (its coroutine is cancelled); nothing when it is not running. */
    fun cancel(id: String)
}

/** What a running task tells the notification. */
fun interface PodWorkProgress {
    /** The line under the title and the progress, 0–100 (`null`: indeterminate). */
    fun update(text: String?, percent: Int?)
}
