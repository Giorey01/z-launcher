package dev.zlauncher.ui.common

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.zlauncher.ui.theme.LocalZTheme

/** Height of the round-ended pills of the headers (back pill, sort pill). */
val PillHeight = 31.dp

/** A small round chip with a controller button name ("B", "LB"): the hint of what the button does. */
@Composable
fun ButtonChip(label: String, modifier: Modifier = Modifier, size: Dp = 22.dp) {
    val theme = LocalZTheme.current
    Box(modifier.size(size).background(theme.textPrimary.copy(alpha = 0.18f), CircleShape), contentAlignment = Alignment.Center) {
        Text(label, color = theme.textPrimary.copy(alpha = 0.6f), fontSize = (size.value * 0.42f).sp, fontWeight = FontWeight.SemiBold)
    }
}

/** A faint pill (translucent, no border): the container of header hints. */
@Composable
fun FaintPill(modifier: Modifier = Modifier, content: @Composable () -> Unit) {
    val theme = LocalZTheme.current
    Row(
        modifier.height(PillHeight).background(theme.textPrimary.copy(alpha = 0.10f), RoundedCornerShape(PillHeight)).padding(horizontal = 8.dp),
        horizontalArrangement = Arrangement.spacedBy(6.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) { content() }
}

/** The "B ←" pill at the left of a header: tells that B goes back. */
@Composable
fun BackPill(modifier: Modifier = Modifier, onClick: (() -> Unit)? = null) {
    val clickModifier = if (onClick != null) modifier.clickable { onClick() } else modifier
    FaintPill(clickModifier) {
        ButtonChip("B")
        Icon(Icons.AutoMirrored.Rounded.ArrowBack, null, Modifier.size(20.dp), tint = LocalZTheme.current.textPrimary.copy(alpha = 0.55f))
    }
}

/** A header title: Poppins semibold, large, in the primary text colour at reduced opacity as on a glass bar. */
@Composable
fun HeaderTitle(text: String, modifier: Modifier = Modifier, color: Color = LocalZTheme.current.textPrimary.copy(alpha = 0.85f)) {
    Text(text, modifier, color = color, fontSize = 20.sp, fontWeight = FontWeight.SemiBold, maxLines = 1)
}

/** Fills the height of the parent with a thin divider line in the theme's divider colour. */
@Composable
fun DividerLine(modifier: Modifier = Modifier) {
    Box(modifier.fillMaxHeight().size(1.dp).background(LocalZTheme.current.divider))
}
