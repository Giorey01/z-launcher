package dev.zlauncher.ui.info

import android.content.Context
import dev.zlauncher.R
import dev.zlauncher.core.nativecore.PlayStats

/** Seconds in a minute. */
private const val MINUTE = 60L

/** Minutes in an hour. */
private const val HOUR_MINUTES = 60L

/** A play time as "12 h 30 min", "45 min", or "less than a minute". */
fun formatDuration(context: Context, seconds: Long): String {
    val minutes = seconds / MINUTE
    val hours = minutes / HOUR_MINUTES
    return when {
        minutes < 1 -> context.getString(R.string.dur_under_minute)
        hours == 0L -> context.getString(R.string.dur_m, minutes)
        minutes % HOUR_MINUTES == 0L -> context.getString(R.string.dur_h, hours)
        else -> context.getString(R.string.dur_hm, hours, minutes % HOUR_MINUTES)
    }
}

/** "3 h 10 min on this device · 40 h on Steam", from the parts that are known; "Never played" when none is. */
fun playTimeLine(context: Context, stats: PlayStats): String {
    val parts = buildList {
        if (stats.deviceSeconds > 0) add(context.getString(R.string.info_time_device, formatDuration(context, stats.deviceSeconds)))
        if (stats.steamSeconds > 0) add(context.getString(R.string.info_time_steam, formatDuration(context, stats.steamSeconds)))
    }
    return if (parts.isEmpty()) context.getString(R.string.info_never_played) else parts.joinToString(" · ")
}
