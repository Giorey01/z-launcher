package dev.zlauncher.ui.pods

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.zlauncher.R
import dev.zlauncher.ui.pad.PadTile
import dev.zlauncher.ui.pad.TileSurface
import dev.zlauncher.ui.theme.LocalZTheme

/** Height of a row of the Pod list. */
internal val POD_ROW_HEIGHT = 66.dp

/** Colour of the states that need attention (crashed, incompatible). */
internal val POD_WARNING = Color(0xFFE5484D)

/**
 * A row of the Pod list, drawn like the rows of the settings: accent icon, title, a muted subtitle
 * and something on the right. The whole row is one pad target; [onFocused] tells the details panel.
 */
@Composable
internal fun PodListRow(
    icon: ImageVector,
    title: String,
    subtitle: String?,
    onClick: () -> Unit,
    onFocused: () -> Unit,
    modifier: Modifier = Modifier,
    trailing: @Composable () -> Unit = {},
) {
    val theme = LocalZTheme.current
    PadTile(onClick, modifier.fillMaxWidth().height(POD_ROW_HEIGHT), surface = TileSurface.NONE, onFocused = onFocused) {
        Row(Modifier.fillMaxSize().padding(horizontal = 16.dp), verticalAlignment = Alignment.CenterVertically) {
            Icon(icon, null, Modifier.size(28.dp), tint = theme.accentStart)
            Column(Modifier.weight(1f).padding(start = 16.dp, end = 10.dp)) {
                Text(title, color = theme.textPrimary, fontSize = 17.sp, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                if (subtitle != null) {
                    Text(
                        subtitle,
                        color = theme.textPrimary.copy(alpha = 0.55f),
                        fontSize = 13.sp,
                        fontWeight = FontWeight.Medium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            }
            trailing()
        }
    }
}

/** A small coloured pill on the right of a row: a state or a switch value. */
@Composable
internal fun PodChip(text: String, color: Color, filled: Boolean = false) {
    val shape = RoundedCornerShape(50)
    val background = if (filled) color else color.copy(alpha = 0.14f)
    val textColor = if (filled) Color.White else color
    Box(Modifier.background(background, shape).padding(horizontal = 12.dp, vertical = 5.dp)) {
        Text(text, color = textColor, fontSize = 13.sp, fontWeight = FontWeight.SemiBold, maxLines = 1)
    }
}

/** The chip of a Pod state from the core (`active`, `disabled_by_user`, `disabled_by_crashes`, `incompatible`). */
@Composable
internal fun PodStateChip(state: String) {
    val theme = LocalZTheme.current
    when (state) {
        "active" -> PodChip(stringResource(R.string.pod_status_active), theme.accentStart)
        "disabled_by_user" -> PodChip(stringResource(R.string.pod_status_disabled_user), theme.textSecondary)
        "disabled_by_crashes" -> PodChip(stringResource(R.string.pod_status_crashed_short), POD_WARNING)
        else -> PodChip(stringResource(R.string.pod_status_incompatible_short), POD_WARNING)
    }
}

/** The text of one declared permission (§8), or the raw name for one this build does not know. */
@Composable
internal fun permissionText(permission: String): String = when (permission) {
    "library.read" -> stringResource(R.string.pod_perm_library_read)
    "library.rescan" -> stringResource(R.string.pod_perm_library_rescan)
    "launch" -> stringResource(R.string.pod_perm_launch)
    "secrets" -> stringResource(R.string.pod_perm_secrets)
    "http" -> stringResource(R.string.pod_perm_http)
    "archive" -> stringResource(R.string.pod_perm_archive)
    "work" -> stringResource(R.string.pod_perm_work)
    "notifications" -> stringResource(R.string.pod_perm_notifications)
    "folders" -> stringResource(R.string.pod_perm_folders)
    "audio" -> stringResource(R.string.pod_perm_audio)
    else -> "• $permission"
}
