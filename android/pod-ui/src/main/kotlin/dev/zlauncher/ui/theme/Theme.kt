package dev.zlauncher.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
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

/** The launcher's Material theme with Poppins; with a [theme] the colours come from it and [LocalZTheme] is provided. */
@Composable
fun ZLauncherTheme(theme: ZTheme? = null, content: @Composable () -> Unit) {
    // The launcher has one look; the system setting is read only so that it is an explicit decision.
    @Suppress("UNUSED_VARIABLE") val system = isSystemInDarkTheme()
    val colors = darkColorScheme(
        primary = theme?.accentStart ?: AccentA,
        background = theme?.backgroundEnd ?: Backdrop,
        surface = theme?.cardEnd ?: Surface,
        onSurface = theme?.textPrimary ?: TextPrimary,
    )
    MaterialTheme(colorScheme = colors, typography = ZTypography) {
        if (theme == null) content() else CompositionLocalProvider(LocalZTheme provides theme, content = content)
    }
}
