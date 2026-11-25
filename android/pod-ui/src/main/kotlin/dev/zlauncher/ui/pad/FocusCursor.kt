package dev.zlauncher.ui.pad

import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import dev.zlauncher.core.nativecore.CursorStyle
import dev.zlauncher.ui.theme.LocalZTheme
import dev.zlauncher.ui.theme.ZTheme

/** Stroke width as a fraction of the cursor box side, at full highlight thickness. */
private const val STROKE_FRACTION = 0.042f

/** Smallest stroke, in pixels per 100 pixels of density-independent size (keeps small cursors visible). */
private const val MIN_STROKE_DP = 2.2f

/** Corner radius as a fraction of the shorter side of the cursor box. */
private const val RADIUS_FRACTION = 0.19f

/** Length of each bracket arm as a fraction of the shorter side. */
private const val ARM_FRACTION = 0.30f

/** Arm length of the alternative styles. */
private const val ARM_FRACTION_ALT = 0.42f

/** How much the animated styles grow and shrink, as a fraction of the box. */
private const val BREATH_FRACTION = 0.018f

/** Duration of one breath, in milliseconds. */
private const val BREATH_MS = 900

/** Alpha and stroke-multiplier of the multi-tier halo bloom behind the main bracket stroke. */
private val HALO_ALPHAS = listOf(0.08f to 3.2f, 0.18f to 2.2f, 0.35f to 1.4f)

/**
 * The selection cursor, drawn over the focused element: four rounded corner brackets in the selection
 * gradient with a soft glow. Style and thickness come from the theme settings (ANIMATED breathes,
 * OUTLINED is a full rounded outline, the ALTERNATIVE styles use longer arms). [outset] grows the box
 * past the element's bounds (the cursor sits around the icon, not on it); [cornerRadius] fixes the corner size (menus use half the row height).
 */
@Composable
fun FocusCursor(modifier: Modifier = Modifier, outset: Dp = 0.dp, cornerRadius: Dp? = null) {
    val theme = LocalZTheme.current
    val style = theme.settings.cursorStyle
    val animated = style == CursorStyle.ANIMATED || style == CursorStyle.ALTERNATIVE_ANIMATED
    val breath by rememberInfiniteTransition(label = "cursor").animateFloat(
        initialValue = 0f,
        targetValue = 1f,
        animationSpec = infiniteRepeatable(tween(BREATH_MS, easing = FastOutSlowInEasing), RepeatMode.Reverse),
        label = "breath",
    )
    Canvas(modifier) {
        val grow = outset.toPx() + if (animated) breath * BREATH_FRACTION * size.minDimension else 0f
        val box = Rect(Offset(-grow, -grow), Size(size.width + 2 * grow, size.height + 2 * grow))
        drawCursor(theme, box, style, cornerRadius?.toPx())
    }
}

/** Draws the halo, then the bracket (or outline) stroke, inside [box]. */
private fun DrawScope.drawCursor(theme: ZTheme, box: Rect, style: CursorStyle, fixedRadius: Float?) {
    val side = minOf(box.width, box.height)
    val stroke = maxOf(MIN_STROKE_DP.dp.toPx(), side * STROKE_FRACTION * theme.settings.highlightThickness)
    val radius = (fixedRadius ?: (side * RADIUS_FRACTION)).coerceAtMost(side / 2)
    val alt = style == CursorStyle.ALTERNATIVE || style == CursorStyle.ALTERNATIVE_ANIMATED
    val path = if (style == CursorStyle.OUTLINED) outlinePath(box, radius) else bracketPath(box, radius, side * if (alt) ARM_FRACTION_ALT else ARM_FRACTION)
    val brush = Brush.linearGradient(listOf(theme.selectionStart, theme.selectionEnd), box.topLeft, box.bottomRight)
    for ((alpha, factor) in HALO_ALPHAS) {
        drawPath(path, theme.selectionStart.copy(alpha = alpha), style = roundStroke(stroke * factor))
    }
    drawPath(path, brush, style = roundStroke(stroke))
}

/** A round-capped, round-joined stroke of [width] pixels. */
private fun roundStroke(width: Float) = Stroke(width, cap = StrokeCap.Round, join = StrokeJoin.Round)

/** A full rounded rectangle outline. */
private fun outlinePath(box: Rect, radius: Float): Path = Path().apply {
    addRoundRect(androidx.compose.ui.geometry.RoundRect(box, androidx.compose.ui.geometry.CornerRadius(radius)))
}

/** Four separate corner brackets: each is an arm, a quarter circle and an arm. */
private fun bracketPath(box: Rect, radius: Float, arm: Float): Path {
    val path = Path()
    val a = maxOf(arm, radius)
    for ((cx, dx) in listOf(box.left to 1f, box.right to -1f)) {
        for ((cy, dy) in listOf(box.top to 1f, box.bottom to -1f)) {
            path.moveTo(cx + dx * a, cy)
            path.lineTo(cx + dx * radius, cy)
            path.quadraticTo(cx, cy, cx, cy + dy * radius)
            path.lineTo(cx, cy + dy * a)
        }
    }
    return path
}

/** A transparent colour constant for callers that need "no tint". */
val NoTint: Color = Color.Transparent
