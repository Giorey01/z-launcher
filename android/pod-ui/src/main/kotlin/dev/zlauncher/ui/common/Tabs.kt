package dev.zlauncher.ui.common

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.zlauncher.ui.pad.PadTile
import dev.zlauncher.ui.pad.TileSurface
import dev.zlauncher.ui.theme.LocalZTheme

/** Height of a tab pill. */
private val TAB_HEIGHT = 38.dp

/**
 * One tab: the selected one is a card with the title in the accent colour, the others plain text. A runs
 * [onClick]; [onFocused] runs when the pad moves onto it (a row of tabs can follow the cursor with it).
 */
@Composable
fun TabPill(label: String, selected: Boolean, modifier: Modifier = Modifier, onFocused: (() -> Unit)? = null, onClick: () -> Unit) {
    val theme = LocalZTheme.current
    val shape = RoundedCornerShape(50)
    PadTile(
        onClick,
        modifier.height(TAB_HEIGHT),
        shape = shape,
        surface = if (selected) TileSurface.CARD else TileSurface.NONE,
        onFocused = onFocused,
    ) {
        Text(
            label,
            Modifier.align(Alignment.Center).padding(horizontal = 18.dp),
            color = if (selected) theme.accentStart else theme.textPrimary.copy(alpha = 0.6f),
            fontSize = 15.sp,
            fontWeight = if (selected) FontWeight.Bold else FontWeight.SemiBold,
            maxLines = 1,
        )
    }
}

/**
 * A row of tabs for the pad: left and right move over them and the tab under the cursor becomes the selected one
 * ([onSelect] gets its index). [tabModifier] gives each tab its focus tracking.
 */
@Composable
fun TabStrip(
    labels: List<String>,
    selected: Int,
    onSelect: (Int) -> Unit,
    modifier: Modifier = Modifier,
    tabModifier: (index: Int) -> Modifier = { Modifier },
) {
    Row(modifier, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        labels.forEachIndexed { index, label ->
            TabPill(label, index == selected, tabModifier(index), onFocused = { onSelect(index) }) { onSelect(index) }
        }
    }
}
