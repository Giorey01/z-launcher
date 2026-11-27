package dev.zlauncher.nav

import android.view.KeyEvent

/**
 * The D-pad while a game is picked up in a manual folder (RF-26): left and right move it one place, up and down one
 * row, instead of moving the focus. A puts it down and B puts it back (both handled where they always are).
 */
object OrganizeKeys {
    /**
     * Applies a D-pad key to the picked up game; `true` when the key was used, press and release alike, so nothing
     * else moves the focus.
     */
    fun handle(app: AppViewModel, event: KeyEvent): Boolean {
        val columns = app.organize.columns.coerceAtLeast(1)
        val delta = when (event.keyCode) {
            KeyEvent.KEYCODE_DPAD_LEFT -> -1
            KeyEvent.KEYCODE_DPAD_RIGHT -> 1
            KeyEvent.KEYCODE_DPAD_UP -> -columns
            KeyEvent.KEYCODE_DPAD_DOWN -> columns
            else -> return false
        }
        if (event.action == KeyEvent.ACTION_DOWN) app.organize.moveGrabbed(delta)
        return true
    }
}
