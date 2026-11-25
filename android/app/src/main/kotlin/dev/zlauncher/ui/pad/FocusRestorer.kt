package dev.zlauncher.ui.pad

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect

/**
 * Puts the focus back on the remembered element of a screen whenever the screen appears, [resumeTick]
 * changes (the launcher came back to the foreground) or [ready] turns true (the content loaded).
 * [defaultKey] is used when nothing was focused before, or when the remembered element is no longer there; [scrollTo] brings the element into view first
 * when the screen scrolls (it receives the remembered key).
 */
@Composable
fun FocusRestorer(
    memory: FocusMemory,
    resumeTick: Int,
    ready: Boolean,
    defaultKey: String?,
    scrollTo: suspend (String) -> Unit = {},
) {
    LaunchedEffect(resumeTick, ready, defaultKey) {
        if (!ready) return@LaunchedEffect
        defaultKey?.let(memory::defaultTo)
        memory.lastKey?.let { scrollTo(it) }
        // The remembered element may be gone (an app uninstalled, a game hidden, a filter applied): the focus
        // must not be lost (RF-35), so it falls back to the default element.
        if (!memory.restoreSoon() && defaultKey != null && memory.lastKey != defaultKey) {
            memory.lastKey = defaultKey
            scrollTo(defaultKey)
            memory.restoreSoon()
        }
    }
}
