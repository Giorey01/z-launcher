package dev.zlauncher.ui

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.Boot
import dev.zlauncher.nav.Screen
import dev.zlauncher.nav.UiMessage
import dev.zlauncher.ui.drawer.DrawerScreen
import dev.zlauncher.ui.home.HomeScreen
import dev.zlauncher.ui.home.PlatformScreen
import dev.zlauncher.ui.onboarding.OnboardingScreen
import dev.zlauncher.ui.picker.EmulatorPickerScreen
import dev.zlauncher.ui.theme.Backdrop
import dev.zlauncher.ui.theme.SurfaceRaised
import dev.zlauncher.ui.theme.TextPrimary
import kotlinx.coroutines.delay

/** How long a message stays on screen, in milliseconds. */
private const val MESSAGE_MS = 6000L

/** The whole UI: the setup, or the Home with the screens above it, and the message line. */
@Composable
fun LauncherRoot(app: AppViewModel) {
    // BACK pops a screen; on the Home it does nothing, so the launcher is never left by mistake.
    BackHandler(enabled = true) { app.back() }
    Surface(Modifier.fillMaxSize(), color = Backdrop, contentColor = TextPrimary) {
        Screens(app)
    }
}

/** The setup or the Home with its screens, and the message line over them. */
@Composable
private fun Screens(app: AppViewModel) {
    Box(Modifier.fillMaxSize()) {
        when (app.boot) {
            Boot.Loading -> Unit
            Boot.Onboarding -> OnboardingScreen(app)
            Boot.Ready -> ReadyScreens(app)
        }
        app.message?.let { MessageLine(it, { app.message = null }, Modifier.align(Alignment.BottomCenter)) }
    }
}

/** The Home or the screen on top of it. */
@Composable
private fun ReadyScreens(app: AppViewModel) {
    when (val screen = app.top) {
        null -> HomeScreen(app)
        is Screen.Platform -> PlatformScreen(app, screen)
        Screen.Drawer -> DrawerScreen(app)
        Screen.Picker -> EmulatorPickerScreen(app)
    }
}

/** A translated message at the bottom, gone after a few seconds. It never takes the focus. */
@Composable
private fun MessageLine(message: UiMessage, onGone: () -> Unit, modifier: Modifier) {
    LaunchedEffect(message) {
        delay(MESSAGE_MS)
        onGone()
    }
    Box(modifier.navigationBarsPadding().padding(16.dp).fillMaxWidth().background(SurfaceRaised, RoundedCornerShape(12.dp)).padding(16.dp)) {
        Text(message.text(), color = TextPrimary)
    }
}
