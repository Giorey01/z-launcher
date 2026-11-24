package dev.zlauncher.ui.pad

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect

/**
 * Puts the focus back on the remembered element of a screen whenever the screen appears, [resumeTick]
 * changes (the launcher came back to the foreground) or [ready] turns true (the content loaded).
 * [defaultKey] is used when nothing was focused before; [scrollTo] brings the element into view first
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
        memory.restoreSoon()
    }
}
