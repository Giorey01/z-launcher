package dev.zlauncher.ui.pods

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.Screen
import dev.zlauncher.pods.AppPodHost
import dev.zlauncher.ui.theme.LocalZTheme

/** The screen of a Pod: its content, with the launcher's theme and a host over the app's state. */
@Composable
fun PodScreen(app: AppViewModel, screen: Screen.Pod) {
    val pod = app.zl.pods.find(screen.id) ?: return
    val theme = LocalZTheme.current
    val host = remember(theme) { AppPodHost(app, theme) }
    Box(Modifier.fillMaxSize()) { pod.Content(host) }
}
