package dev.zlauncher.apps

import android.content.Context

/** Name of the preferences file. */
private const val PREFS = "recent_apps"

/** Key of the list. */
private const val KEY = "packages"

/** How many recent apps are kept. */
private const val KEEP = 5

/**
 * The apps launched last, newest first, kept in a small preferences file. This is a view of the user's own
 * activity inside the launcher (not system usage statistics), so no permission is needed.
 */
class RecentApps(context: Context) {
    /** The preferences. */
    private val prefs = context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)

    /** The recent packages, newest first. */
    fun list(): List<String> = prefs.getString(KEY, "").orEmpty().split(' ').filter { it.isNotEmpty() }

    /** Records that [packageName] was launched now. */
    fun push(packageName: String) {
        val next = (listOf(packageName) + list().filter { it != packageName }).take(KEEP)
        prefs.edit().putString(KEY, next.joinToString(" ")).apply()
    }
}
