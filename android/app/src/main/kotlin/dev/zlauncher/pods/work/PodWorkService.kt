package dev.zlauncher.pods.work

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import androidx.core.app.NotificationCompat
import dev.zlauncher.R

/** Id of the one notification of the Pods' tasks. */
private const val NOTIFICATION_ID = 7301

/** Channel of the Pods' tasks. */
private const val CHANNEL_ID = "pod_work"

/**
 * Keeps the process in the foreground while a Pod's task runs ([PodWorkRunner]), with one
 * notification that shows the first task and how many run. It stops when the last task ends.
 */
class PodWorkService : Service() {
    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        val notification = notification(this) ?: return START_NOT_STICKY.also { stopSelf() }
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            startForeground(NOTIFICATION_ID, notification, ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC)
        } else {
            startForeground(NOTIFICATION_ID, notification)
        }
        return START_NOT_STICKY
    }

    /** Updating and stopping the service from the runner. */
    companion object {
        /** Shows what runs now, or stops the service when nothing does. */
        fun refresh(context: Context) {
            val notification = notification(context)
            if (notification == null) {
                context.stopService(Intent(context, PodWorkService::class.java))
            } else {
                context.getSystemService(NotificationManager::class.java)?.notify(NOTIFICATION_ID, notification)
            }
        }

        /** The notification of the running tasks, null when none runs. */
        private fun notification(context: Context): Notification? {
            val summary = PodWorkRunner.book.summary() ?: return null
            ensureChannel(context)
            val more = if (summary.count > 1) context.getString(R.string.pod_work_more, summary.count - 1) else null
            return NotificationCompat.Builder(context, CHANNEL_ID)
                .setSmallIcon(android.R.drawable.stat_sys_download)
                .setContentTitle(summary.title)
                .setContentText(listOfNotNull(summary.text, more).joinToString(" · ").ifEmpty { null })
                .setProgress(PodWorkBook.MAX_PERCENT, summary.percent ?: 0, summary.percent == null)
                .setOngoing(true)
                .setOnlyAlertOnce(true)
                .setSilent(true)
                .build()
        }

        /** The channel of the Pods' tasks (low importance: no sound). */
        private fun ensureChannel(context: Context) {
            val manager = context.getSystemService(NotificationManager::class.java) ?: return
            if (manager.getNotificationChannel(CHANNEL_ID) != null) return
            val channel = NotificationChannel(CHANNEL_ID, context.getString(R.string.pod_work_channel), NotificationManager.IMPORTANCE_LOW)
            manager.createNotificationChannel(channel)
        }
    }
}
