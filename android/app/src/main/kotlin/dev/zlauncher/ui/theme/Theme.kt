package dev.zlauncher.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

/** Background of every screen. */
val Backdrop = Color(0xFF0D0F16)

/** Background of tiles and panels. */
val Surface = Color(0xFF1A1E2B)

/** Brighter surface, used under the focused tile. */
val SurfaceRaised = Color(0xFF262C3E)

/** Main accent: the start of the focus cursor gradient. */
val AccentA = Color(0xFF6EE7F9)

/** Second accent: the end of the focus cursor gradient. */
val AccentB = Color(0xFFA78BFA)

/** Text on dark surfaces. */
val TextPrimary = Color(0xFFF2F4FA)

/** Secondary text. */
val TextSecondary = Color(0xFF9AA3BC)

/** Colour of error messages. */
val ErrorColor = Color(0xFFFF8A8A)

/** The launcher is always dark: it sits on top of wallpapers and game art. */
@Composable
fun ZLauncherTheme(content: @Composable () -> Unit) {
    // The launcher has one look; the system setting is read only so that it is an explicit decision.
    @Suppress("UNUSED_VARIABLE") val system = isSystemInDarkTheme()
    val colors = darkColorScheme(primary = AccentA, background = Backdrop, surface = Surface, onSurface = TextPrimary)
    MaterialTheme(colorScheme = colors, content = content)
}
