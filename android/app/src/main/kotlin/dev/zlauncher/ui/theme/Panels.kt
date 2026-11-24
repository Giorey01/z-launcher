package dev.zlauncher.ui.theme

import android.os.Build
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.requiredSize
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.BlurEffect
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.TileMode
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.positionInRoot
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import dev.zlauncher.core.nativecore.SurfaceMaterial

/** Blur radius, in dp, of one step of the `glass_blur` setting (0 to 10). */
private const val BLUR_DP_PER_STEP = 5f

/** Alpha of a glass panel's tint at the strongest `glass_tint`. */
private const val GLASS_TINT_ALPHA = 0.9f

/** Alpha of the tint when the content behind cannot be blurred (older Android, video wallpaper). */
private const val TRANSLUCENT_ALPHA = 0.88f

/** Shape of the panels of the lists and menus. */
val PanelShape = RoundedCornerShape(24.dp)

/**
 * A panel in the theme's surface material. SOLID is the card gradient with a soft shadow. GLASS blurs the
 * wallpaper behind it (Android 12+; older versions and video wallpapers use a translucent card), tints it with
 * the card colours and adds a rim highlight, all driven by the `glass_*` settings.
 */
@Composable
fun ZPanel(
    modifier: Modifier = Modifier,
    shape: Shape = PanelShape,
    elevation: Dp = 6.dp,
    content: @Composable BoxScope.() -> Unit,
) {
    val theme = LocalZTheme.current
    val glass = theme.settings.surfaceMaterial == SurfaceMaterial.GLASS
    var position by remember { mutableStateOf(Offset.Zero) }
    Box(modifier.onGloballyPositioned { position = it.positionInRoot() }.then(if (glass) Modifier else Modifier.shadow(elevation, shape)).clip(shape)) {
        if (glass) GlassLayers(theme, position) else Box(Modifier.matchParentSize().background(theme.cardBrush()))
        Box(Modifier.matchParentSize().border(1.dp, rim(theme), shape))
        content()
    }
}

/** The rim highlight of a surface: a light edge at the top fading to nothing at the bottom. */
private fun rim(theme: ZTheme): Brush {
    val strength = theme.settings.glassRimStrength.coerceIn(0f, 1f)
    return Brush.verticalGradient(listOf(Color.White.copy(alpha = 0.35f * strength), Color.White.copy(alpha = 0.04f * strength)))
}

/** The glass body: the blurred wallpaper (when possible) and the tint over it. */
@Composable
private fun BoxScope.GlassLayers(theme: ZTheme, position: Offset) {
    val backdrop = LocalBackdrop.current
    val canBlur = backdrop != null && Build.VERSION.SDK_INT >= Build.VERSION_CODES.S && theme.settings.glassBlur > 0f
    if (canBlur && backdrop != null) {
        val configuration = LocalConfiguration.current
        val density = LocalDensity.current
        val radius = theme.settings.glassBlur * BLUR_DP_PER_STEP * density.density
        Box(Modifier.matchParentSize().graphicsLayer { renderEffect = BlurEffect(radius, radius, TileMode.Clamp) }) {
            WallpaperImage(
                backdrop.path,
                animated = false,
                Modifier.requiredSize(configuration.screenWidthDp.dp, configuration.screenHeightDp.dp)
                    .offset { IntOffset(-position.x.toInt(), -position.y.toInt()) },
            )
        }
    }
    val alpha = if (canBlur) theme.settings.glassTint.coerceIn(0f, 1f) * GLASS_TINT_ALPHA + 0.25f else TRANSLUCENT_ALPHA
    Box(Modifier.matchParentSize().background(theme.cardBrush(alpha.coerceAtMost(1f))))
}
