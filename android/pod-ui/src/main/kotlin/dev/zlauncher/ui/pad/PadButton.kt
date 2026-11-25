package dev.zlauncher.ui.pad

import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.zlauncher.ui.theme.LocalZTheme

/** A button for the controller: a [PadTile] with a label. */
@Composable
fun PadButton(label: String, onClick: () -> Unit, modifier: Modifier = Modifier) {
    PadTile(onClick, modifier) {
        Text(
            label,
            color = LocalZTheme.current.textPrimary,
            fontSize = 18.sp,
            fontWeight = FontWeight.SemiBold,
            modifier = Modifier.padding(horizontal = 24.dp, vertical = 14.dp),
        )
    }
}
