package dev.zlauncher.ui.theme

import androidx.compose.runtime.Immutable
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import dev.zlauncher.core.nativecore.ResolvedTheme
import dev.zlauncher.core.nativecore.SoundEntry
import dev.zlauncher.core.nativecore.UserSettings
import dev.zlauncher.core.nativecore.Wallpaper

/**
 * The look of the launcher as the core resolved it (active theme plus the user's settings): colours
 * as Compose [Color]s, the numeric and enum settings, the wallpapers and the sound files.
 * [revision] grows with every reload, so caches keyed on it are dropped when the theme changes.
 */
@Immutable
data class ZTheme(
    /** Name of the theme in use. */
    val name: String,
    /** Start of the screen background gradient (top). */
    val backgroundStart: Color,
    /** End of the screen background gradient (bottom). */
    val backgroundEnd: Color,
    /** Start (top) of the gradient of panels and cards. */
    val cardStart: Color,
    /** End (bottom) of the gradient of panels and cards. */
    val cardEnd: Color,
    /** Main text colour. */
    val textPrimary: Color,
    /** Secondary text colour. */
    val textSecondary: Color,
    /** Tint of icons drawn in a single colour. */
    val iconTint: Color,
    /** Fill of tiles. */
    val tileBackground: Color,
    /** Border of tiles. */
    val tileBorder: Color,
    /** Start of the gradient of a switch or control that is off. */
    val toggleOffStart: Color,
    /** End of the gradient of a switch or control that is off. */
    val toggleOffEnd: Color,
    /** Start of the gradient of a slider thumb or switch knob. */
    val thumbStart: Color,
    /** End of the gradient of a slider thumb or switch knob. */
    val thumbEnd: Color,
    /** Colour of the drop shadows. */
    val dropShadow: Color,
    /** Light inner edge of raised surfaces. */
    val innerLight: Color,
    /** Dark inner edge of raised surfaces. */
    val innerDark: Color,
    /** Separator lines. */
    val divider: Color,
    /** Start of the accent gradient. */
    val accentStart: Color,
    /** End of the accent gradient. */
    val accentEnd: Color,
    /** Glow around accent elements. */
    val accentGlow: Color,
    /** Start of the selection cursor gradient. */
    val selectionStart: Color,
    /** End of the selection cursor gradient. */
    val selectionEnd: Color,
    /** Every look setting (cursor, surface, icon style, screens...). */
    val settings: UserSettings,
    /** Wallpaper of the grid screen. */
    val wallpaperMain: Wallpaper?,
    /** Wallpaper of the hero (external) screen. */
    val wallpaperExternal: Wallpaper?,
    /** Sound file for each action that has one. */
    val sounds: List<SoundEntry>,
    /** Counter of reloads, to key caches. */
    val revision: Int,
) {
    /** The accent as a horizontal-ish gradient brush for fills. */
    fun accentBrush(): Brush = Brush.linearGradient(listOf(accentStart, accentEnd))

    /** The selection cursor gradient. */
    fun selectionBrush(): Brush = Brush.linearGradient(listOf(selectionStart, selectionEnd))

    /** The vertical gradient of panels and cards, scaled to [alpha]. */
    fun cardBrush(alpha: Float = 1f): Brush =
        Brush.verticalGradient(listOf(cardStart.copy(alpha = cardStart.alpha * alpha), cardEnd.copy(alpha = cardEnd.alpha * alpha)))

    /** The vertical gradient of the screen background. */
    fun backgroundBrush(): Brush = Brush.verticalGradient(listOf(backgroundStart, backgroundEnd))
}

/** An ARGB integer from the core as a Compose colour. */
private fun UInt.argb(): Color = Color(toInt())

/** Converts what the core resolved into a [ZTheme] stamped with [revision]. */
fun ResolvedTheme.toZTheme(revision: Int): ZTheme = ZTheme(
    name = name,
    backgroundStart = colors.backgroundGradientStart.argb(),
    backgroundEnd = colors.backgroundGradientEnd.argb(),
    cardStart = colors.cardGradientStart.argb(),
    cardEnd = colors.cardGradientEnd.argb(),
    textPrimary = colors.textPrimary.argb(),
    textSecondary = colors.textSecondary.argb(),
    iconTint = colors.iconTint.argb(),
    tileBackground = colors.tileBackground.argb(),
    tileBorder = colors.tileBorder.argb(),
    toggleOffStart = colors.toggleOffGradientStart.argb(),
    toggleOffEnd = colors.toggleOffGradientEnd.argb(),
    thumbStart = colors.toggleThumbGradientStart.argb(),
    thumbEnd = colors.toggleThumbGradientEnd.argb(),
    dropShadow = colors.dropShadow.argb(),
    innerLight = colors.innerShadowLight.argb(),
    innerDark = colors.innerShadowDark.argb(),
    divider = colors.divider.argb(),
    accentStart = colors.accentGradientStart.argb(),
    accentEnd = colors.accentGradientEnd.argb(),
    accentGlow = colors.accentGlow.argb(),
    selectionStart = colors.selectionGradientStart.argb(),
    selectionEnd = colors.selectionGradientEnd.argb(),
    settings = settings,
    wallpaperMain = wallpaperMain,
    wallpaperExternal = wallpaperExternal,
    sounds = sounds,
    revision = revision,
)

/** The theme of the composition; screens read colours and settings from here, never hard-code them. */
val LocalZTheme = compositionLocalOf<ZTheme> { error("ZLauncherTheme is missing") }
