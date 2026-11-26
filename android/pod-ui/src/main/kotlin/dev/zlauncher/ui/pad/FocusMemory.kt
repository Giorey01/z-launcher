package dev.zlauncher.ui.pad

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import kotlinx.coroutines.android.awaitFrame

/** How many frames [FocusMemory.restoreSoon] waits for the remembered element to appear. */
private const val RESTORE_ATTEMPTS = 6

/**
 * Remembers which element of a screen had the focus, so the focus can be put back after the screen is
 * left and re-entered, after a launched game returns to the launcher, or after a dialog closes (RF-35:
 * the focus is never lost). One instance per screen, kept by the view model so it outlives the composition.
 */
class FocusMemory {
    /** Key of the element that had the focus last, `null` before any. */
    var lastKey: String? by mutableStateOf(null)

    /**
     * One requester per element, attached for good: adding a requester to an element just as it
     * took the focus changed its modifiers, and the focus could be dropped on the way (from the dock
     * down into the grid, the cursor vanished until A was pressed).
     */
    private val requesters = HashMap<String, FocusRequester>()

    /** The requester of the element with [key] (attached once the element is composed with [track]). */
    fun requesterOf(key: String): FocusRequester = requesters.getOrPut(key) { FocusRequester() }

    /** The last focused key that [accept]s, `null` when none was focused yet. */
    fun lastWhere(accept: (String) -> Boolean): String? = history.lastOrNull(accept)

    /** The keys in the order they last had the focus, the latest at the end. */
    private val history = LinkedHashSet<String>()

    /** Marks an element as the one to focus when nothing was focused before. */
    fun defaultTo(key: String) {
        if (lastKey == null) lastKey = key
    }

    /** Asks the remembered element for the focus; `false` when it is not on screen (not composed yet). */
    fun restore(): Boolean = try {
        val key = lastKey
        if (key == null) {
            false
        } else {
            requesterOf(key).requestFocus()
            true
        }
    } catch (_: IllegalStateException) {
        false
    }

    /** Like [restore] but waits for the element to be laid out: retries over the next frames; `false` if it never appeared. */
    suspend fun restoreSoon(): Boolean {
        repeat(RESTORE_ATTEMPTS) {
            if (restore()) return true
            awaitFrame()
        }
        return false
    }

    /** Wires an element in: it always carries its own requester and reports when it gets the focus. */
    fun track(key: String): Modifier = Modifier
        .focusRequester(requesterOf(key))
        .onFocusChanged {
            if (it.isFocused) {
                lastKey = key
                history.remove(key)
                history.add(key)
            }
        }
}

/** Asks this requester for the focus, retrying over the next frames until its element is laid out. */
suspend fun FocusRequester.requestSoon() {
    repeat(RESTORE_ATTEMPTS) {
        try {
            requestFocus()
            return
        } catch (_: IllegalStateException) {
            // The element is not attached yet; try again after the next frame.
        }
        awaitFrame()
    }
}
