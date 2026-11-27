package dev.zlauncher.ui.pods

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.Screen
import dev.zlauncher.pods.AppPodHost
import dev.zlauncher.ui.theme.LocalZTheme
import java.io.File

/** The screen of a Pod: its content, with the launcher's theme and a host over the app's state (§7, §10). */
@Composable
fun PodScreen(app: AppViewModel, screen: Screen.Pod) {
    val entry = app.zl.pods.findEntry(screen.id) ?: return
    val theme = LocalZTheme.current
    val host = remember(theme, screen.id) {
        AppPodHost(screen.id, entry.permissions, app, theme, entry.i18nDir)
    }
    PodAlive(app, host, screen.id)

    Box(Modifier.fillMaxSize()) {
        entry.pod.Content(host, screen.route)
    }
}

/**
 * While this is in the composition the Pod [podId] has something of its own on screen: its host gets the launcher's
 * events, and a marker file names the Pod, so that if the process dies while it is drawn the next start counts it as a
 * crash of that Pod (§10). Composition cannot be wrapped in a `try`, so this is how a Pod's screen or tab is isolated.
 */
@Composable
fun PodAlive(app: AppViewModel, host: AppPodHost, podId: String) {
    DisposableEffect(host) {
        val sentinel = File(app.zl.filesDir, "active_pod")
        runCatching { sentinel.writeText(podId) }
        app.podHosts.add(host)
        onDispose {
            app.podHosts.remove(host)
            runCatching { sentinel.delete() }
        }
    }
}
