package dev.zlauncher.ui

import androidx.activity.compose.BackHandler
import android.os.Build
import androidx.compose.foundation.background
import androidx.compose.ui.draw.blur
import androidx.compose.ui.focus.focusProperties
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
import dev.zlauncher.display.Role
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.Boot
import dev.zlauncher.nav.Screen
import dev.zlauncher.nav.UiMessage
import dev.zlauncher.ui.drawer.DrawerScreen
import dev.zlauncher.ui.hero.HeroRoot
import dev.zlauncher.ui.home.HomeScreen
import dev.zlauncher.ui.home.PlatformScreen
import dev.zlauncher.ui.menus.AppMenu
import dev.zlauncher.ui.menus.GameMenu
import dev.zlauncher.ui.nowplaying.NowPlayingScreen
import dev.zlauncher.ui.menus.StartMenu
import dev.zlauncher.nav.Overlay
import dev.zlauncher.ui.onboarding.OnboardingScreen
import dev.zlauncher.ui.pad.ProvidePlay
import dev.zlauncher.ui.info.GameInfoScreen
import dev.zlauncher.ui.picker.ConsolePickerScreen
import dev.zlauncher.ui.pods.PodScreen
import dev.zlauncher.ui.picker.EmulatorPickerScreen
import dev.zlauncher.ui.settings.SettingsAside
import dev.zlauncher.ui.settings.SettingsMain
import dev.zlauncher.ui.settings.SgdbKeyDialog
import dev.zlauncher.ui.settings.SteamAccountDialog
import dev.zlauncher.ui.single.SingleRoot
import dev.zlauncher.ui.theme.LocalZTheme
import dev.zlauncher.ui.theme.WallpaperScreen
import kotlinx.coroutines.delay

/** How long a message stays on screen, in milliseconds. */
private const val MESSAGE_MS = 6000L

/**
 * The whole UI of one window: BACK pops a screen (on the Home it does nothing, so the launcher is never left by
 * mistake) and the part drawn depends on the [role] of the display: the grid, the hero area or both. While a
 * game runs on another display, this one ([displayId]) shows the now playing screen instead.
 */
@Composable
fun ScreenRoot(app: AppViewModel, role: Role, displayId: Int) {
    BackHandler(enabled = true) { app.back() }
    val playing = app.nowPlaying?.takeIf { it.displayId != displayId }
    ProvidePlay(app::play) {
        Surface(Modifier.fillMaxSize(), color = LocalZTheme.current.backgroundEnd, contentColor = LocalZTheme.current.textPrimary) {
            if (playing != null) {
                NowPlayingScreen(app, playing)
                return@Surface
            }
            if (app.settings.wantSgdbKey && role != Role.HERO) {
                SgdbKeyDialog(app) { app.settings.wantSgdbKey = false }
            }
            if (app.settings.wantSteamAccount && role != Role.HERO) {
                SteamAccountDialog(app) { app.settings.wantSteamAccount = false }
            }
            when (role) {
                Role.GRID -> GridRoot(app)
                Role.HERO -> HeroRoot(app)
                Role.SINGLE -> SingleRoot(app)
            }
        }
    }
}

/** The grid display: main wallpaper, the setup or the Home with its screens and menus, and the message line. */
@Composable
fun GridRoot(app: AppViewModel) {
    WallpaperScreen(LocalZTheme.current.wallpaperMain) {
        GridContent(app, single = false)
        app.message?.let { MessageLine(it, { app.message = null }, Modifier.align(Alignment.BottomCenter)) }
    }
}

/** The setup or the Home with its screens; [single] is true when this display also shows the settings list. */
@Composable
fun GridContent(app: AppViewModel, single: Boolean) {
    val menuOpen = app.overlay != null
    Box(
        Modifier.fillMaxSize()
            .focusProperties { canFocus = !menuOpen }
            .then(if (menuOpen && Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) Modifier.blur(10.dp) else Modifier),
    ) {
        when (app.boot) {
            Boot.Loading -> Unit
            Boot.Onboarding -> OnboardingScreen(app)
            Boot.Ready -> ReadyScreens(app, single)
        }
    }
    if (!single) {
        when (val menu = app.overlay) {
            null -> Unit
            is Overlay.GameMenu -> GameMenu(app, menu)
            Overlay.StartMenu -> StartMenu(app)
            is Overlay.AppMenu -> AppMenu(app, menu)
        }
    }
}

/** The Home or the screen on top of it, then the open menu. */
@Composable
private fun ReadyScreens(app: AppViewModel, single: Boolean) {
    when (val screen = app.top) {
        null -> HomeScreen(app)
        is Screen.Platform -> PlatformScreen(app, screen)
        Screen.Drawer -> DrawerScreen(app)
        Screen.Picker -> EmulatorPickerScreen(app)
        is Screen.ConsolePicker -> ConsolePickerScreen(app, screen)
        is Screen.GameInfo -> GameInfoScreen(app, screen)
        is Screen.Pod -> PodScreen(app, screen)
        Screen.PodManagement -> dev.zlauncher.ui.pods.PodManagementScreen(app)
        Screen.Search -> dev.zlauncher.ui.search.SearchScreen(app)
        Screen.Settings -> if (single) SettingsMain(app) else SettingsAside(app)
    }
}

/** A translated message at the bottom, gone after a few seconds. It never takes the focus. */
@Composable
internal fun MessageLine(message: UiMessage, onGone: () -> Unit, modifier: Modifier) {
    LaunchedEffect(message) {
        delay(MESSAGE_MS)
        onGone()
    }
    val theme = LocalZTheme.current
    Box(modifier.navigationBarsPadding().padding(16.dp).fillMaxWidth().background(theme.cardEnd, RoundedCornerShape(12.dp)).padding(16.dp)) {
        Text(message.text(), color = theme.textPrimary)
    }
}
