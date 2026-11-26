package dev.zlauncher.audio

import android.app.Activity
import android.app.Application
import android.os.Bundle
import dev.zlauncher.MainActivity

/**
 * Tells the [MusicPlayer] whether a launcher screen is visible: it counts the started activities
 * of the app (the home and the screens on the other display), so the music stops when a game or
 * another app covers them all.
 */
class MusicVisibility(private val music: MusicPlayer) : Application.ActivityLifecycleCallbacks {
    /** Activities of the app started and not yet stopped. */
    private var started = 0

    /** One more screen is visible. */
    override fun onActivityStarted(activity: Activity) {
        started++
        music.setVisible(true)
    }

    /** One screen fewer; with none left the music pauses. */
    override fun onActivityStopped(activity: Activity) {
        started = (started - 1).coerceAtLeast(0)
        if (started == 0) music.setVisible(false)
    }

    /** Not needed. */
    override fun onActivityCreated(activity: Activity, savedInstanceState: Bundle?) = Unit

    /** The home in front again (a game or an app was closed): the music may come back. */
    override fun onActivityResumed(activity: Activity) {
        if (activity is MainActivity) music.back()
    }

    /** Not needed. */
    override fun onActivityPaused(activity: Activity) = Unit

    /** Not needed. */
    override fun onActivitySaveInstanceState(activity: Activity, outState: Bundle) = Unit

    /** Not needed. */
    override fun onActivityDestroyed(activity: Activity) = Unit
}
