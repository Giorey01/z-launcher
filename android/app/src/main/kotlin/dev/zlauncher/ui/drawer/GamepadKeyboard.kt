package dev.zlauncher.ui.drawer

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.Backspace
import androidx.compose.material.icons.rounded.ArrowUpward
import androidx.compose.material.icons.rounded.Check
import androidx.compose.material.icons.rounded.ContentPaste
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.zlauncher.core.nativecore.SoundAction
import dev.zlauncher.ui.pad.LocalPlay
import dev.zlauncher.ui.pad.requestSoon
import dev.zlauncher.ui.theme.LocalZTheme
import dev.zlauncher.ui.theme.ZPanel

/** Specification of one keyboard key. */
private data class KeySpec(
    val label: String,
    val hint: String? = null,
    val badge: String? = null,
    val icon: ImageVector? = null,
    val weight: Float = 1f,
    val action: KeyAction = KeyAction.Char(label),
)

/** What pressing a key does. */
private sealed interface KeyAction {
    data class Char(val text: String) : KeyAction
    data object Backspace : KeyAction
    data object Shift : KeyAction
    data object Space : KeyAction
    data object Confirm : KeyAction
}

/** The keys that press a keyboard key: gamepad A, the D-pad centre and Enter. */
private val CONFIRM_KEYS = setOf(Key.ButtonA, Key.DirectionCenter, Key.Enter, Key.NumPadEnter)

/** Standard QWERTY layout rows matching Cocoon reference 12-search-gamepad-keyboard.png. */
private val ROW_1 = listOf(
    KeySpec("q"), KeySpec("w"), KeySpec("e", hint = "é"), KeySpec("r", hint = "ř"),
    KeySpec("t", hint = "ť"), KeySpec("y", hint = "ý"), KeySpec("u", hint = "ú"),
    KeySpec("i", hint = "í"), KeySpec("o", hint = "ó"), KeySpec("p")
)

private val ROW_2 = listOf(
    KeySpec("a", hint = "à"), KeySpec("s", hint = "ś"), KeySpec("d", hint = "ď"),
    KeySpec("f"), KeySpec("g", hint = "ğ"), KeySpec("h"), KeySpec("j"),
    KeySpec("k"), KeySpec("l", hint = "ł"),
    KeySpec("", badge = "B", icon = Icons.AutoMirrored.Rounded.Backspace, weight = 1.2f, action = KeyAction.Backspace)
)

private val ROW_3 = listOf(
    KeySpec("", badge = "L1", icon = Icons.Rounded.ArrowUpward, weight = 1.1f, action = KeyAction.Shift),
    KeySpec("z", hint = "ź"), KeySpec("x"), KeySpec("c", hint = "ç"),
    KeySpec("v"), KeySpec("b"), KeySpec("n", hint = "ñ"), KeySpec("m"),
    KeySpec(","), KeySpec(".")
)

/** The custom gamepad QWERTY keyboard matching Cocoon 12-search-gamepad-keyboard.png. */
@Composable
fun GamepadKeyboard(
    query: String,
    onQueryChange: (String) -> Unit,
    onConfirm: () -> Unit,
    onClose: () -> Unit,
    modifier: Modifier = Modifier,
) {
    var caps by remember { mutableStateOf(false) }
    // The keyboard opens with the focus on its first key: with a controller there is no other way in.
    val firstKey = remember { FocusRequester() }
    LaunchedEffect(Unit) { firstKey.requestSoon() }
    val play = LocalPlay.current
    val handleAction: (KeyAction) -> Unit = { action ->
        when (action) {
            is KeyAction.Char -> {
                val ch = if (caps) action.text.uppercase() else action.text
                onQueryChange(query + ch)
                play(SoundAction.NAVIGATE)
            }
            KeyAction.Backspace -> {
                if (query.isNotEmpty()) onQueryChange(query.dropLast(1)) else onClose()
                play(SoundAction.BACK)
            }
            KeyAction.Shift -> {
                caps = !caps
                play(SoundAction.SELECT)
            }
            KeyAction.Space -> {
                onQueryChange("$query ")
                play(SoundAction.NAVIGATE)
            }
            KeyAction.Confirm -> {
                onConfirm()
                play(SoundAction.SELECT)
            }
        }
    }
    KeyboardPanel(caps, firstKey, handleAction, modifier)
}

/** The rounded keyboard panel: the action bar, the three letter rows and the bottom row. */
@Composable
private fun KeyboardPanel(caps: Boolean, firstKey: FocusRequester, onAction: (KeyAction) -> Unit, modifier: Modifier) {
    Box(
        modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(topStart = 24.dp, topEnd = 24.dp))
            .background(Color(0xF52A243D))
            .padding(horizontal = 8.dp, vertical = 10.dp)
    ) {
        Column(Modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(6.dp)) {
            KeyboardActionBar()
            KeyRow(ROW_1, caps, onAction, firstKey)
            KeyRow(ROW_2, caps, onAction)
            KeyRow(ROW_3, caps, onAction)
            BottomControlRow(onAction)
        }
    }
}

/** Top bar above the keyboard: Paste and Accents buttons. */
@Composable
private fun KeyboardActionBar() {
    Row(
        Modifier.fillMaxWidth().padding(horizontal = 4.dp, vertical = 2.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        ActionChip(label = "Paste", icon = Icons.Rounded.ContentPaste)
        ActionChip(label = "Accents", badge = "R1")
    }
}

/** Action chip button in keyboard header. */
@Composable
private fun ActionChip(label: String, icon: ImageVector? = null, badge: String? = null) {
    Row(
        Modifier
            .clip(RoundedCornerShape(12.dp))
            .background(Color.White.copy(alpha = 0.12f))
            .padding(horizontal = 10.dp, vertical = 5.dp),
        horizontalArrangement = Arrangement.spacedBy(6.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        badge?.let {
            Box(Modifier.size(16.dp).background(Color.Black.copy(alpha = 0.35f), CircleShape), contentAlignment = Alignment.Center) {
                Text(it, color = Color.White.copy(alpha = 0.8f), fontSize = 8.sp, fontWeight = FontWeight.Bold)
            }
        }
        icon?.let { Icon(it, null, Modifier.size(14.dp), tint = Color.White.copy(alpha = 0.8f)) }
        Text(label, color = Color.White.copy(alpha = 0.8f), fontSize = 12.sp, fontWeight = FontWeight.Medium)
    }
}

/** One row of keys in the keyboard grid. */
@Composable
private fun KeyRow(keys: List<KeySpec>, caps: Boolean, onAction: (KeyAction) -> Unit, firstKey: FocusRequester? = null) {
    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(4.dp)) {
        for ((index, key) in keys.withIndex()) {
            val focusFirst = if (index == 0 && firstKey != null) Modifier.focusRequester(firstKey) else Modifier
            KeyTile(key, caps, Modifier.weight(key.weight).then(focusFirst), onAction)
        }
    }
}

/** Bottom row: symbols, space bar with Y badge, and confirm with checkmark. */
@Composable
private fun BottomControlRow(onAction: (KeyAction) -> Unit) {
    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(4.dp)) {
        KeyTile(KeySpec("?1", badge = "L2", weight = 1.3f, action = KeyAction.Char("?")), false, Modifier.weight(1.3f), onAction)
        KeyTile(KeySpec("", badge = "Y", weight = 4.5f, action = KeyAction.Space), false, Modifier.weight(4.5f), onAction)
        KeyTile(KeySpec("", icon = Icons.Rounded.Check, weight = 1.5f, action = KeyAction.Confirm), false, Modifier.weight(1.5f), onAction)
    }
}

/** One key button with gamepad focus and controller shortcut badge. */
@Composable
private fun KeyTile(spec: KeySpec, caps: Boolean, modifier: Modifier, onAction: (KeyAction) -> Unit) {
    val theme = LocalZTheme.current
    var focused by remember { mutableStateOf(false) }
    val displayLabel = if (caps) spec.label.uppercase() else spec.label
    val activeBorder = if (focused) Modifier.border(2.dp, theme.accentStart, RoundedCornerShape(10.dp)) else Modifier
    val keyBg = if (focused) Color(0xFF5A4F76) else Color(0xFF423A5B)
    Box(
        modifier
            .height(44.dp)
            .clip(RoundedCornerShape(10.dp))
            .background(keyBg)
            .then(activeBorder)
            // No separate focusable(): clickable is the one focus target, so the requester of the first key,
            // the highlight below and the D-pad all talk about the same node.
            .onFocusChanged { focused = it.isFocused }
            .onKeyEvent { event ->
                // Only the first press: holding A must not type the letter over and over (as PadTile does).
                val first = event.type == KeyEventType.KeyDown && event.nativeKeyEvent.repeatCount == 0
                if (first && event.key in CONFIRM_KEYS) {
                    onAction(spec.action)
                    true
                } else false
            }
            .clickable { onAction(spec.action) },
        contentAlignment = Alignment.Center
    ) {
        KeyContent(spec, displayLabel, focused)
    }
}

/** Content of a key: label or icon, superscript hint, and shortcut badge. */
@Composable
private fun androidx.compose.foundation.layout.BoxScope.KeyContent(spec: KeySpec, displayLabel: String, focused: Boolean) {
    val theme = LocalZTheme.current
    if (displayLabel.isNotEmpty()) {
        Text(
            displayLabel,
            color = if (focused) theme.accentStart else theme.textPrimary,
            fontSize = 17.sp,
            fontWeight = FontWeight.SemiBold
        )
    }
    spec.icon?.let { Icon(it, null, Modifier.size(18.dp), tint = if (focused) theme.accentStart else Color.White) }
    spec.hint?.let {
        Text(
            it,
            Modifier.align(Alignment.TopEnd).padding(top = 2.dp, end = 3.dp),
            color = theme.textPrimary.copy(alpha = 0.35f),
            fontSize = 9.sp
        )
    }
    spec.badge?.let { KeyBadge(it, Modifier.align(Alignment.TopEnd)) }
}

/** Small controller badge chip on top right of a key. */
@Composable
private fun KeyBadge(badge: String, modifier: Modifier = Modifier) {
    Box(
        modifier
            .padding(top = 3.dp, end = 3.dp)
            .size(14.dp)
            .background(Color.Black.copy(alpha = 0.35f), CircleShape),
        contentAlignment = Alignment.Center
    ) {
        Text(badge, color = Color.White.copy(alpha = 0.75f), fontSize = 7.sp, fontWeight = FontWeight.Bold)
    }
}
