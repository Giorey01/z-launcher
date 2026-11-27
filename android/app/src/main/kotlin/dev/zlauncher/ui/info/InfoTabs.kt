package dev.zlauncher.ui.info

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import dev.zlauncher.core.nativecore.SessionRow

/** A dialog of the Activity tab (RF-41). */
sealed interface ActivityDialog {
    /** Correcting the duration of [session]. */
    data class Edit(
        /** The session. */
        val session: SessionRow,
    ) : ActivityDialog

    /** Asking whether to delete [session]. */
    data class Delete(
        /** The session. */
        val session: SessionRow,
    ) : ActivityDialog

    /** Adding a session by hand. */
    data object Add : ActivityDialog
}

/**
 * The state of a game's sheet that outlives its composition: which tab is open, how many there are (the launcher's
 * own plus the Pods'), and the dialog of the Activity tab. It lives in the view model so that LB and RB, which the
 * activity reads, and B, which closes a dialog before leaving the screen, reach it.
 */
class InfoTabs {
    /** Index of the open tab: 0 is Info, 1 Activity, then those of the Pods. */
    var selected by mutableIntStateOf(0)
        private set

    /** Number of tabs of the sheet on screen. */
    var count by mutableIntStateOf(FIXED_TABS)
        private set

    /** The dialog open over the Activity tab, if any. */
    var dialog by mutableStateOf<ActivityDialog?>(null)

    /** Bumped when the sessions changed, so the Activity tab reads them again. */
    var sessionsVersion by mutableIntStateOf(0)
        private set

    /** A new sheet opens: first tab, no dialog, [tabs] tabs. */
    fun open(tabs: Int) {
        selected = 0
        dialog = null
        count = tabs.coerceAtLeast(1)
    }

    /** The sheet has [tabs] tabs now (a Pod was turned off); the open one stays valid. */
    fun resize(tabs: Int) {
        count = tabs.coerceAtLeast(1)
        selected = selected.coerceIn(0, count - 1)
    }

    /** Opens the tab at [index], if there is one. */
    fun select(index: Int) {
        if (index in 0 until count) selected = index
    }

    /** Moves to the previous ([delta] < 0) or next tab, stopping at the ends (LB and RB); `false` at an end. */
    fun step(delta: Int): Boolean {
        val next = selected + delta
        if (next !in 0 until count || dialog != null) return false
        selected = next
        return true
    }

    /** Closes the open dialog; `false` when there was none (B then leaves the screen). */
    fun dismissDialog(): Boolean {
        if (dialog == null) return false
        dialog = null
        return true
    }

    /** The sessions changed: the Activity tab reads them again. */
    fun sessionsChanged() {
        sessionsVersion++
    }

    /** Where the tabs count starts. */
    companion object {
        /** The tabs of the launcher itself: Info and Activity. */
        const val FIXED_TABS = 2
    }
}
