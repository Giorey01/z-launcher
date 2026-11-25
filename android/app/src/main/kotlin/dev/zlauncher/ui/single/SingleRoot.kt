package dev.zlauncher.ui.single

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dev.zlauncher.core.nativecore.UserSettings
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.Overlay
import dev.zlauncher.nav.Screen
import dev.zlauncher.ui.GridContent
import dev.zlauncher.ui.MessageLine
import dev.zlauncher.ui.hero.HeroArea
import dev.zlauncher.ui.hero.StatusPill
import dev.zlauncher.ui.home.GridSpec
import dev.zlauncher.ui.home.LocalGridSpec
import dev.zlauncher.ui.menus.AppMenu
import dev.zlauncher.ui.menus.GameMenu
import dev.zlauncher.ui.menus.StartMenu
import dev.zlauncher.ui.theme.LocalZTheme
import dev.zlauncher.ui.theme.WallpaperScreen
import kotlin.math.floor

/** Height the dock takes at the top of the grid part. */
private val DOCK_HEIGHT = 64.dp

/** Smallest number of columns the single-screen grid uses. */
private const val MIN_COLUMNS = 3

/**
 * The single-screen mode: the hero area on the top part (`hero_split` of the height), the grid below with
 * `single_screen_rows` rows (one row becomes a horizontal row in the chosen style). While the settings,
 * drawer or picker are open they take the whole screen. Overlays, panels and messages sit at full-screen scope.
 */
@Composable
fun SingleRoot(app: AppViewModel) {
    val theme = LocalZTheme.current
    val settings = theme.settings
    WallpaperScreen(theme.wallpaperMain) {
        val fullScreen = app.top == Screen.Settings || app.top == Screen.Drawer || app.top == Screen.Picker
        if (fullScreen || app.top is Screen.ConsolePicker || app.top is Screen.GameInfo) {
            GridContent(app, single = true)
        } else {
            SingleContent(app, settings)
        }
        when (val menu = app.overlay) {
            null -> Unit
            is Overlay.GameMenu -> GameMenu(app, menu)
            Overlay.StartMenu -> StartMenu(app)
            is Overlay.AppMenu -> AppMenu(app, menu)
        }
        app.message?.let { MessageLine(it, { app.message = null }, Modifier.align(Alignment.BottomCenter)) }
    }
}

/** The single-screen layout: hero area on the top part, grid below. */
@Composable
private fun SingleContent(app: AppViewModel, settings: UserSettings) {
    BoxWithConstraints(Modifier.fillMaxSize()) {
        val totalWidth = maxWidth
        val totalHeight = maxHeight
        val heroHeight = totalHeight * settings.heroSplit.coerceIn(0f, 1f)
        Column(Modifier.fillMaxSize()) {
            Box(Modifier.fillMaxWidth().height(heroHeight)) {
                HeroArea(app.hero)
                StatusPill(Modifier.align(Alignment.TopEnd).padding(top = 6.dp, end = 10.dp))
            }
            val gridHeight = totalHeight - heroHeight - DOCK_HEIGHT
            val rows = settings.singleScreenRows.toInt().coerceIn(1, 5)
            val cell = gridHeight / rows
            val columns = floor(totalWidth / cell).toInt().coerceAtLeast(MIN_COLUMNS)
            val spec = GridSpec(columns, columns, if (rows == 1) 1 else rows, settings.singleRowStyle, cell)
            CompositionLocalProvider(LocalGridSpec provides spec) { GridContent(app, single = true) }
        }
    }
}
