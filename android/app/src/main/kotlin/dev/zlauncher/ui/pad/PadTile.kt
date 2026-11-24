package dev.zlauncher.ui.pad

import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.focusable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.unit.dp
import dev.zlauncher.ui.theme.Surface
import dev.zlauncher.ui.theme.SurfaceRaised

/** The keys that confirm: gamepad A, the D-pad centre and Enter (RF-35). */
private val CONFIRM_KEYS = setOf(Key.ButtonA, Key.DirectionCenter, Key.Enter, Key.NumPadEnter)

/**
 * A focusable tile. A (or D-pad centre, Enter, a tap) runs [onClick]; Y runs [onSecondary] when given.
 * The focused tile is raised and wears the [FocusCursor]. [modifier] carries the size and the focus tracking.
 */
@Composable
fun PadTile(
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    onSecondary: (() -> Unit)? = null,
    content: @Composable BoxScope.() -> Unit,
) {
    var focused by remember { mutableStateOf(false) }
    val click by rememberUpdatedState(onClick)
    val secondary by rememberUpdatedState(onSecondary)
    Box(
        modifier
            .onFocusChanged { focused = it.isFocused }
            .onKeyEvent { event ->
                val first = event.type == KeyEventType.KeyDown && event.nativeKeyEvent.repeatCount == 0
                when {
                    !first -> false
                    event.key in CONFIRM_KEYS -> true.also { click() }
                    event.key == Key.ButtonY && secondary != null -> true.also { secondary?.invoke() }
                    else -> false
                }
            }
            .focusable()
            .pointerInput(Unit) { detectTapGestures { click() } }
            .clip(RoundedCornerShape(14.dp))
            .background(if (focused) SurfaceRaised else Surface),
    ) {
        content()
        if (focused) FocusCursor(Modifier.matchParentSize())
    }
}
