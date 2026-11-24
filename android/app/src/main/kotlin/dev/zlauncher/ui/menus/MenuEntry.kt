package dev.zlauncher.ui.menus

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowRight
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.graphics.Shape
import dev.zlauncher.ui.pad.FocusCursor
import dev.zlauncher.ui.pad.PadTile
import dev.zlauncher.ui.pad.TileSurface
import dev.zlauncher.ui.pad.requestSoon
import dev.zlauncher.ui.theme.LocalZTheme
import dev.zlauncher.ui.theme.ZPanel

import androidx.compose.foundation.shape.CircleShape
import androidx.compose.ui.graphics.Color

/** One entry of a menu: its icon, its text, and whether it opens a submenu. */
data class MenuEntry(
    /** Icon at the left, drawn in the accent colour. */
    val icon: ImageVector,
    /** The text. */
    val label: String,
    /** Shows a right arrow: the entry opens a submenu. */
    val submenu: Boolean = false,
    /** Optional gamepad button hint, e.g. "Y", "R1", "X". */
    val badge: String? = null,
    /** What A does. */
    val onSelect: () -> Unit,
)

/** Thickness of the separator between rows. */
private val SEPARATOR = 1.5.dp

/**
 * A menu: a themed panel of [shape] with the entries one under the other, dark separators between them, and the
 * cursor brackets around the focused row. The cursor is drawn outside the panel's clip so it may overhang the
 * edge, as it does in the reference. The first row takes the focus when the menu appears or when [resetKey]
 * changes (a submenu replaced the entries). [rowHeight] is the height of a row.
 */
@Composable
fun MenuPanel(
    entries: List<MenuEntry>,
    rowHeight: Dp,
    resetKey: Any,
    shape: Shape,
    modifier: Modifier = Modifier,
    elevation: Dp = 12.dp
) {
    val first = remember { FocusRequester() }
    var focused by remember { mutableIntStateOf(0) }
    LaunchedEffect(resetKey) {
        focused = 0
        first.requestSoon()
    }
    Box(modifier) {
        ZPanel(Modifier.fillMaxWidth(), shape, elevation) {
            Column {
                entries.forEachIndexed { index, entry ->
                    if (index > 0) Box(Modifier.fillMaxWidth().height(SEPARATOR).background(LocalZTheme.current.innerDark))
                    MenuRow(entry, rowHeight, if (index == 0) Modifier.focusRequester(first) else Modifier) { focused = index }
                }
            }
        }
        if (focused in entries.indices) {
            Box(Modifier.offset(y = (rowHeight + SEPARATOR) * focused).fillMaxWidth().height(rowHeight)) {
                FocusCursor(Modifier.matchParentSize(), 4.dp, rowHeight / 2 + 4.dp)
            }
        }
    }
}

/** A row: icon, text, optional badge, optional arrow. The panel draws the cursor. */
@Composable
private fun MenuRow(entry: MenuEntry, height: Dp, modifier: Modifier, onFocused: () -> Unit) {
    val theme = LocalZTheme.current
    PadTile(
        entry.onSelect,
        modifier.fillMaxWidth().height(height),
        surface = TileSurface.NONE,
        showCursor = false,
        onFocused = onFocused,
    ) { focused ->
        Row(Modifier.fillMaxSize().padding(horizontal = 14.dp), verticalAlignment = Alignment.CenterVertically) {
            Icon(entry.icon, null, Modifier.size(height * 0.5f), tint = theme.accentStart)
            Text(
                entry.label,
                Modifier.weight(1f).padding(start = 16.dp),
                color = theme.textPrimary.copy(alpha = if (focused) 0.95f else 0.75f),
                fontSize = 16.sp,
                fontWeight = FontWeight.SemiBold,
            )
            entry.badge?.let { MenuBadge(it) }
            if (entry.submenu) {
                Icon(Icons.AutoMirrored.Rounded.ArrowRight, null, Modifier.size(24.dp), tint = theme.textPrimary.copy(alpha = 0.5f))
            }
        }
    }
}

/** A small circle badge showing the button shortcut of an entry. */
@Composable
private fun MenuBadge(badge: String) {
    val theme = LocalZTheme.current
    Box(
        Modifier.size(24.dp).background(Color.Black.copy(alpha = 0.28f), CircleShape),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            badge,
            color = theme.textPrimary.copy(alpha = 0.55f),
            fontSize = 11.sp,
            fontWeight = FontWeight.Bold,
        )
    }
}
