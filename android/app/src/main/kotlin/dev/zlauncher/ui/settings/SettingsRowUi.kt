package dev.zlauncher.ui.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowRight
import androidx.compose.material.icons.rounded.AddCircle
import androidx.compose.material.icons.rounded.ArrowDropDown
import androidx.compose.material.icons.rounded.Check
import androidx.compose.material.icons.rounded.Remove
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.zlauncher.ui.art.ArtImage
import dev.zlauncher.ui.theme.LocalZTheme

/** Width of the control column at the right of a row with a dropdown. */
private val DROPDOWN_WIDTH = 180.dp

/** Height of a dropdown pill. */
private val DROPDOWN_HEIGHT = 40.dp

/** Smallest height of a row by its control: pills, sliders and swatches make a row taller. */
private fun minHeightOf(control: Control) = when (control) {
    is Control.Dropdown -> 74.dp
    is Control.Slider -> 87.dp
    is Control.Swatches -> 96.dp
    else -> 56.dp
}

/** One row of a list: icon, title, subtitle and control. The cursor is drawn by the list, over the row. */
@Composable
fun SettingRowItem(row: SettingRow, dropdownIndex: Int) {
    val theme = LocalZTheme.current
    Column {
        Box(Modifier.fillMaxWidth()) {
            Row(
                Modifier
                    .fillMaxWidth()
                    .heightIn(min = minHeightOf(row.control))
                    .padding(start = 15.dp, end = 12.dp, top = 6.dp, bottom = 6.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                RowIcon(row, Modifier.width(46.dp))
                Column(Modifier.weight(1f).padding(end = 8.dp)) {
                    Text(
                        row.title,
                        color = theme.textPrimary.copy(alpha = 0.9f),
                        fontSize = 16.sp,
                        lineHeight = 22.sp,
                        fontWeight = FontWeight.SemiBold,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                    if (row.subtitle != null && row.control !is Control.Slider && row.control !is Control.Swatches) {
                        Text(
                            row.subtitle,
                            color = theme.textPrimary.copy(alpha = 0.5f),
                            fontSize = 12.sp,
                            lineHeight = 17.sp,
                            fontWeight = FontWeight.Medium,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis
                        )
                    }
                    InlineControl(row)
                }
                SideControl(row.control)
            }
        }
        if (dropdownIndex >= 0) (row.control as? Control.Dropdown)?.let { OptionList(it, dropdownIndex) }
        Box(Modifier.fillMaxWidth().size(1.5.dp).background(theme.innerDark))
    }
}

/** The icon of a row, or its preview picture when it has one. */
@Composable
private fun RowIcon(row: SettingRow, modifier: Modifier) {
    val theme = LocalZTheme.current
    Box(modifier, contentAlignment = Alignment.CenterStart) {
        if (row.preview != null) {
            ArtImage(row.preview, Modifier.size(width = 44.dp, height = 28.dp).clip(RoundedCornerShape(4.dp)), ContentScale.Crop)
        } else {
            Icon(row.icon, null, Modifier.size(27.dp), tint = theme.iconTint.copy(alpha = 0.55f))
        }
    }
}

/** Controls that sit under the title (slider, swatches). */
@Composable
private fun InlineControl(row: SettingRow) {
    when (val control = row.control) {
        is Control.Slider -> SliderControl(control)
        is Control.Swatches -> SwatchesControl(control)
        else -> Unit
    }
}

/** Controls that sit at the right of the title (chevron, dropdown, switch, value). */
@Composable
private fun SideControl(control: Control) {
    val theme = LocalZTheme.current
    when (control) {
        Control.Chevron -> Icon(Icons.AutoMirrored.Rounded.ArrowRight, null, Modifier.size(26.dp), tint = theme.textPrimary.copy(alpha = 0.45f))
        is Control.Dropdown -> DropdownPill(control.options.getOrElse(control.selected) { "" })
        is Control.Switch -> SwitchControl(control.on)
        is Control.Value -> Text(control.text, color = theme.accentStart, fontSize = 18.sp, fontWeight = FontWeight.Bold)
        else -> Unit
    }
}

/** The closed dropdown: a rounded pill with the chosen text and a caret. */
@Composable
private fun DropdownPill(text: String) {
    val theme = LocalZTheme.current
    Row(
        Modifier
            .width(DROPDOWN_WIDTH)
            .size(DROPDOWN_WIDTH, DROPDOWN_HEIGHT)
            .background(theme.cardBrush(), CircleShape)
            .border(1.dp, theme.tileBorder, CircleShape)
            .padding(start = 14.dp, end = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text,
            Modifier.weight(1f),
            color = theme.textPrimary.copy(alpha = 0.9f),
            fontSize = 15.sp,
            fontWeight = FontWeight.SemiBold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis
        )
        Icon(Icons.Rounded.ArrowDropDown, null, Modifier.size(24.dp), tint = theme.textPrimary.copy(alpha = 0.6f))
    }
}

/** The open dropdown: its options one under the other, the highlighted one marked. */
@Composable
private fun OptionList(dropdown: Control.Dropdown, highlighted: Int) {
    val theme = LocalZTheme.current
    Column(Modifier.fillMaxWidth().padding(start = 61.dp, end = 12.dp, bottom = 8.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
        dropdown.options.forEachIndexed { index, option ->
            val on = index == highlighted
            val bg = if (on) theme.accentStart.copy(alpha = 0.28f) else Color.Transparent
            Row(
                Modifier
                    .fillMaxWidth()
                    .background(bg, RoundedCornerShape(10.dp))
                    .padding(horizontal = 12.dp, vertical = 8.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    option,
                    Modifier.weight(1f),
                    color = theme.textPrimary.copy(alpha = if (on) 1f else 0.7f),
                    fontSize = 15.sp,
                    fontWeight = FontWeight.SemiBold
                )
                if (index == dropdown.selected) Icon(Icons.Rounded.Check, null, Modifier.size(18.dp), tint = theme.accentStart)
            }
        }
    }
}

/** The switch: a pill track and a round knob, in the accent gradient when on. */
@Composable
private fun SwitchControl(on: Boolean) {
    val theme = LocalZTheme.current
    val track = if (on) theme.accentBrush() else Brush.linearGradient(listOf(theme.toggleOffStart, theme.toggleOffEnd))
    Box(Modifier.size(54.dp, 30.dp).background(track, CircleShape).padding(3.dp), contentAlignment = if (on) Alignment.CenterEnd else Alignment.CenterStart) {
        Box(Modifier.size(24.dp).background(Brush.verticalGradient(listOf(theme.thumbStart, theme.thumbEnd)), CircleShape))
    }
}

/** The slider: the value, a minus, the track with a thumb and a plus. */
@Composable
private fun SliderControl(slider: Control.Slider) {
    val theme = LocalZTheme.current
    val fraction = ((slider.value - slider.min) / (slider.max - slider.min)).coerceIn(0f, 1f)
    Row(Modifier.fillMaxWidth().padding(top = 4.dp), verticalAlignment = Alignment.CenterVertically) {
        Text(slider.label, Modifier.width(44.dp), color = theme.textPrimary.copy(alpha = 0.6f), fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
        Icon(Icons.Rounded.Remove, null, Modifier.size(20.dp), tint = theme.textPrimary.copy(alpha = 0.4f))
        Box(Modifier.weight(1f).padding(horizontal = 8.dp).size(width = 200.dp, height = 24.dp), contentAlignment = Alignment.CenterStart) {
            Box(Modifier.fillMaxWidth().size(4.dp).background(theme.innerDark, CircleShape))
            Row(Modifier.fillMaxWidth()) {
                if (fraction > 0f) Box(Modifier.weight(fraction).size(1.dp))
                Box(Modifier.size(24.dp).background(theme.accentBrush(), CircleShape))
                if (fraction < 1f) Box(Modifier.weight(1f - fraction).size(1.dp))
            }
        }
        Icon(Icons.Rounded.AddCircle, null, Modifier.size(20.dp), tint = theme.textPrimary.copy(alpha = 0.4f))
    }
}

/** The accent swatches: the theme's own colour first, then the fixed colours, the chosen one with a check. */
@Composable
private fun SwatchesControl(swatches: Control.Swatches) {
    val theme = LocalZTheme.current
    Row(Modifier.fillMaxWidth().padding(top = 6.dp), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
        swatches.colors.forEachIndexed { index, color ->
            val fill = color?.let { Brush.verticalGradient(listOf(it.copy(alpha = 0.9f), it)) } ?: theme.accentBrush()
            val borderColor = Color.White.copy(alpha = if (index == swatches.selected) 0.9f else 0.18f)
            Box(
                Modifier
                    .size(36.dp)
                    .background(fill, CircleShape)
                    .border(2.dp, borderColor, CircleShape),
                contentAlignment = Alignment.Center
            ) {
                if (index == swatches.selected) Icon(Icons.Rounded.Check, null, Modifier.size(18.dp), tint = Color.White)
            }
        }
    }
}
