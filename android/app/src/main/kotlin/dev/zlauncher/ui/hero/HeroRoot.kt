package dev.zlauncher.ui.hero

import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.Screen
import dev.zlauncher.ui.settings.SettingsMain
import dev.zlauncher.ui.theme.LocalZTheme
import dev.zlauncher.ui.theme.WallpaperScreen

/**
 * The big screen of the dual-screen mode: external wallpaper with the hero of the focused item
 * and the status pill in the top-right corner.
 */
@Composable
fun HeroRoot(app: AppViewModel) {
    val theme = LocalZTheme.current
    WallpaperScreen(theme.wallpaperExternal) {
        if (app.top == Screen.Settings) {
            SettingsMain(app)
        } else {
            HeroArea(app.hero)
            StatusPill(Modifier.align(Alignment.TopEnd).padding(top = 10.dp, end = 16.dp))
        }
    }
}
