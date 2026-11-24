package dev.zlauncher

import android.os.Bundle
import dev.zlauncher.display.Role
import java.lang.ref.WeakReference

/**
 * The launcher's window on the second display (the hero screen, or the grid when the roles are swapped).
 * It is started by [MainActivity] and closes itself when the launcher goes back to a single screen.
 */
class SecondaryActivity : ScreenActivity() {
    /** Registers this window as the running companion. */
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        alive = WeakReference(this)
    }

    /** One screen only: nothing to show here, so the window closes. */
    override fun onRoleApplied(role: Role) {
        if (role == Role.SINGLE) finishAndRemoveTask()
    }

    /** Forgets this window. */
    override fun onDestroy() {
        if (alive?.get() === this) alive = null
        super.onDestroy()
    }

    /** The running window. */
    companion object {
        /** The running companion window, `null` when there is none. */
        var alive: WeakReference<SecondaryActivity>? = null
    }
}
