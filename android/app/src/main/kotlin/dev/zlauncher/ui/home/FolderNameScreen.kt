package dev.zlauncher.ui.home

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Folder
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.zlauncher.R
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.Screen
import dev.zlauncher.ui.common.BackPill
import dev.zlauncher.ui.common.HeaderTitle
import dev.zlauncher.ui.input.GamepadKeyboard
import dev.zlauncher.ui.theme.LocalZTheme
import dev.zlauncher.ui.theme.ZPanel

/** Longest name typed: the core cuts it to its own limit, this only stops the field from running away. */
private const val MAX_TYPED = 60

/** The shape of the box that shows the name. */
private val NAME_BOX = RoundedCornerShape(10.dp)

/**
 * The name of a manual folder, typed with the gamepad keyboard: for a new folder (from the Start menu, or from a
 * menu that puts games in a folder that does not exist yet) or to rename one. Confirming on the keyboard saves,
 * closing it or B goes back without saving.
 */
@Composable
fun FolderNameScreen(app: AppViewModel, screen: Screen.FolderName) {
    val theme = LocalZTheme.current
    var name by remember(screen) { mutableStateOf(screen.initial) }
    val save = { app.organize.saveFolderName(screen.renameId, name, screen.thenAdd, screen.thenMoveFrom) }
    Column(
        Modifier.fillMaxSize().background(theme.backgroundBrush()).padding(horizontal = 24.dp, vertical = 20.dp),
        verticalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
            BackPill(onClick = { app.back() })
            HeaderTitle(stringResource(R.string.folder_name_title), Modifier.padding(start = 12.dp))
        }
        ZPanel(Modifier.fillMaxWidth().height(52.dp), NAME_BOX, elevation = 2.dp) {
            Row(Modifier.fillMaxSize().padding(horizontal = 14.dp), verticalAlignment = Alignment.CenterVertically) {
                Icon(Icons.Rounded.Folder, null, Modifier.padding(end = 10.dp), tint = theme.accentStart)
                Text(
                    name.ifEmpty { stringResource(R.string.folder_name_hint) },
                    color = if (name.isEmpty()) theme.textSecondary else theme.textPrimary,
                    fontSize = 16.sp,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }
        Box(Modifier.fillMaxWidth().weight(1f), contentAlignment = Alignment.Center) {
            GamepadKeyboard(
                query = name,
                onQueryChange = { name = it.take(MAX_TYPED) },
                onConfirm = { save() },
                onClose = { app.back() },
            )
        }
    }
}
