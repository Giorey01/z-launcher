package dev.zlauncher

import android.content.Intent
import android.hardware.display.DisplayManager
import dev.zlauncher.display.DisplayRoles
import dev.zlauncher.display.Role

/**
 * The launcher's Home window: the one the system starts, which can also be the device Home. In the
 * dual-screen mode it starts [SecondaryActivity] on the other display and each window shows its part.
 */
class MainActivity : ScreenActivity() {
    /** In the dual-screen mode makes sure the companion window is running on the other display. */
    override fun onRoleApplied(role: Role) {
        if (role != Role.SINGLE) {
            val settings = app.theme.value?.settings
            if (settings != null) {
                val dm = getSystemService(DisplayManager::class.java)
                val displays = DisplayRoles.candidates(dm, settings.externalDisplayDocking)
                val other = DisplayRoles.otherDisplay(displays, display?.displayId ?: 0)
                if (other != null && SecondaryActivity.alive?.get() == null) {
                    startCompanion(this, other.displayId, SecondaryActivity::class.java)
                }
            }
        }
    }

    /** The system Home button while the launcher is already open: back to the Home screen. */
    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        if (intent.action == Intent.ACTION_MAIN) app.goHome()
    }
}
