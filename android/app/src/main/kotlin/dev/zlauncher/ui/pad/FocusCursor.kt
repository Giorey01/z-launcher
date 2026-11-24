package dev.zlauncher.ui.pad

import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.unit.dp
import dev.zlauncher.ui.theme.AccentA
import dev.zlauncher.ui.theme.AccentB

/** Length of each bracket arm, in dp. */
private const val ARM_DP = 18f

/** Thickness of the bracket stroke, in dp. */
private const val STROKE_DP = 3.5f

/** Distance of the brackets from the edge of the tile when at rest, in dp (inside it, so the tile's clip keeps them). */
private const val INSET_DP = 3f

/** How far the brackets breathe inwards, in dp. */
private const val BREATH_DP = 3f

/** Duration of one breath, in milliseconds. */
private const val BREATH_MS = 650

/** The focus cursor: four animated corner brackets drawn over the focused element. */
@Composable
fun FocusCursor(modifier: Modifier = Modifier) {
    val breath = rememberInfiniteTransition(label = "cursor").animateFloat(
        initialValue = 0f,
        targetValue = 1f,
        animationSpec = infiniteRepeatable(tween(BREATH_MS, easing = FastOutSlowInEasing), RepeatMode.Reverse),
        label = "breath",
    )
    Canvas(modifier) { drawBrackets((INSET_DP + breath.value * BREATH_DP).dp.toPx()) }
}

/** One corner: where it sits and which way its arms point (1 or -1 on each axis). */
private data class Corner(val x: Float, val y: Float, val dx: Float, val dy: Float)

/** Draws the four corners, [inset] pixels inside the edges. */
private fun DrawScope.drawBrackets(inset: Float) {
    val brush = Brush.linearGradient(listOf(AccentA, AccentB), Offset.Zero, Offset(size.width, size.height))
    val width = STROKE_DP.dp.toPx()
    val arm = ARM_DP.dp.toPx()
    val left = inset
    val top = inset
    val right = size.width - inset
    val bottom = size.height - inset
    val corners = listOf(Corner(left, top, 1f, 1f), Corner(right, top, -1f, 1f), Corner(left, bottom, 1f, -1f), Corner(right, bottom, -1f, -1f))
    for (c in corners) {
        drawLine(brush, Offset(c.x, c.y), Offset(c.x + c.dx * arm, c.y), width, StrokeCap.Round)
        drawLine(brush, Offset(c.x, c.y), Offset(c.x, c.y + c.dy * arm), width, StrokeCap.Round)
    }
}
