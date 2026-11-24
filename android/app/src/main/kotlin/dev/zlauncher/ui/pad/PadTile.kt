package dev.zlauncher.ui.pad

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.focusable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import dev.zlauncher.core.nativecore.SoundAction
import dev.zlauncher.ui.theme.LocalZTheme

/** The keys that confirm: gamepad A, the D-pad centre and Enter (RF-35). */
private val CONFIRM_KEYS = setOf(Key.ButtonA, Key.DirectionCenter, Key.Enter, Key.NumPadEnter)

/** Plays an interface sound; provided by the screen root so every tile can make its click. */
val LocalPlay = compositionLocalOf<(SoundAction) -> Unit> { {} }

/** Provides [play] to the tiles under [content]. */
@Composable
fun ProvidePlay(play: (SoundAction) -> Unit, content: @Composable () -> Unit) {
    CompositionLocalProvider(LocalPlay provides play, content = content)
}

/** How a [PadTile] looks when it is not just an invisible hit area. */
enum class TileSurface {
    /** A card in the theme's card gradient with a border. */
    CARD,

    /** No background: the content (art) is the tile. */
    NONE,
}

/**
 * A focusable element. A (or D-pad centre, Enter, a tap) runs [onClick]; Y runs [onSecondary] when given.
 * The focused tile scales to the theme's `hover_scale`, wears the [FocusCursor] ([cursorOutset] past its
 * bounds) and makes the navigation sound; [onFocused] runs when it gets the focus. [modifier] carries the
 * size and the focus tracking. [showCursor] false leaves the drawing of the cursor to a parent that is not clipped.
 */
@Composable
fun PadTile(
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    onSecondary: (() -> Unit)? = null,
    shape: Shape = RoundedCornerShape(14.dp),
    surface: TileSurface = TileSurface.CARD,
    cursorOutset: Dp = 0.dp,
    showCursor: Boolean = true,
    onFocused: (() -> Unit)? = null,
    content: @Composable BoxScope.(focused: Boolean) -> Unit,
) {
    val theme = LocalZTheme.current
    val play = LocalPlay.current
    var focused by remember { mutableStateOf(false) }
    val click by rememberUpdatedState(onClick)
    val secondary by rememberUpdatedState(onSecondary)
    val focusedCallback by rememberUpdatedState(onFocused)
    val scale by animateFloatAsState(if (focused) theme.settings.hoverScale else 1f, label = "hover")
    Box(
        modifier
            .onFocusChanged {
                if (it.isFocused && !focused) {
                    play(SoundAction.NAVIGATE)
                    focusedCallback?.invoke()
                }
                focused = it.isFocused
            }
            .onKeyEvent { event ->
                val first = event.type == KeyEventType.KeyDown && event.nativeKeyEvent.repeatCount == 0
                when {
                    !first -> false
                    event.key in CONFIRM_KEYS -> true.also { play(SoundAction.SELECT); click() }
                    event.key == Key.ButtonY && secondary != null -> true.also { play(SoundAction.OPEN_EDIT); secondary?.invoke() }
                    else -> false
                }
            }
            .focusable()
            .pointerInput(Unit) { detectTapGestures { click() } },
    ) {
        Box(Modifier.matchParentSize().graphicsLayer { scaleX = scale; scaleY = scale }.then(cardLook(surface, shape))) {
            content(focused)
        }
        if (focused && showCursor) FocusCursor(Modifier.matchParentSize(), cursorOutset)
    }
}

/** The card background of a tile with a surface (gradient and a faint border), or nothing. */
@Composable
private fun cardLook(surface: TileSurface, shape: Shape): Modifier {
    if (surface == TileSurface.NONE) return Modifier
    val theme = LocalZTheme.current
    return Modifier.clip(shape).background(theme.cardBrush()).border(1.dp, theme.tileBorder, shape)
}
