package dev.zlauncher.ui.info

/** Smallest height of a bar with some time in it, as a share of the chart: a short week must still show. */
private const val MIN_BAR = 0.06f

/** Seconds in an hour. */
private const val HOUR = 3_600L

/** Seconds in a day. */
private const val DAY = 86_400L

/** Hour of the day (local time) at which a session added by hand for a past day ends. */
private const val EVENING_HOUR = 22L

/** Shortest session that can be typed in by hand, in seconds (one minute). */
const val MIN_MANUAL_SECONDS = 60L

/** Longest session that can be typed in by hand: a day, as in the core's `MAX_MANUAL_SESSION_SECONDS`. */
const val MAX_MANUAL_SECONDS = 24 * HOUR

/** How far back a session can be added by hand, in days. */
const val MAX_DAYS_AGO = 365

/** The height of each bar as a share of the tallest ([0, 1]); a week with time never falls under [MIN_BAR]. */
fun barFractions(seconds: List<Long>): List<Float> {
    val peak = seconds.maxOrNull() ?: 0L
    return seconds.map { if (it <= 0 || peak <= 0) 0f else maxOf(it.toFloat() / peak, MIN_BAR) }
}

/**
 * When a session of [durationSeconds] added by hand for [daysAgo] days ago started (Unix seconds). For today it ended
 * just now; for an earlier day it ended at 22:00 of that day in the time zone [utcOffsetSeconds] east of UTC, so that
 * it lands in the week the user means and never in the future.
 */
fun manualSessionStart(nowSeconds: Long, utcOffsetSeconds: Long, daysAgo: Int, durationSeconds: Long): Long {
    val end = if (daysAgo <= 0) {
        nowSeconds
    } else {
        val today = Math.floorDiv(nowSeconds + utcOffsetSeconds, DAY) * DAY
        minOf(today - daysAgo * DAY + EVENING_HOUR * HOUR - utcOffsetSeconds, nowSeconds)
    }
    return end - durationSeconds
}

/** [seconds] moved by [delta] and kept within the range a hand-typed session may have. */
fun adjustDuration(seconds: Long, delta: Long): Long = (seconds + delta).coerceIn(MIN_MANUAL_SECONDS, MAX_MANUAL_SECONDS)
