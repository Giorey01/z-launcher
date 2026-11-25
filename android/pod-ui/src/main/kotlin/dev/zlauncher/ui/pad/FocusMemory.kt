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

    /** The requester attached to the element whose key is [lastKey]. */
    val requester = FocusRequester()

    /** Marks an element as the one to focus when nothing was focused before. */
    fun defaultTo(key: String) {
        if (lastKey == null) lastKey = key
    }

    /** Asks the remembered element for the focus; `false` when it is not on screen (not composed yet). */
    fun restore(): Boolean = try {
        requester.requestFocus()
        true
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

    /** Wires an element in: it carries the requester while it is the remembered one and reports when it gets the focus. */
    fun track(key: String): Modifier = Modifier
        .then(if (lastKey == key) Modifier.focusRequester(requester) else Modifier)
        .onFocusChanged { if (it.isFocused) lastKey = key }
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
