package dev.zlauncher.ui.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import dev.zlauncher.R
import dev.zlauncher.core.nativecore.CoreException
import dev.zlauncher.core.nativecore.KeyCheck
import dev.zlauncher.launch.toMessage
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.UiMessage
import dev.zlauncher.ui.theme.LocalZTheme
import kotlinx.coroutines.launch

/** Width of the key editor. */
private val DIALOG_WIDTH = 520.dp

/**
 * The SteamGridDB key editor: the user pastes the key (it is long, so Paste is the main way in), it is checked with
 * SteamGridDB and stored only when it is accepted, or when SteamGridDB cannot be reached. The stored key is never
 * shown back.
 */
@Composable
fun SgdbKeyDialog(app: AppViewModel, onClose: () -> Unit) {
    val theme = LocalZTheme.current
    val clipboard = LocalClipboardManager.current
    val scope = rememberCoroutineScope()
    var key by remember { mutableStateOf("") }
    var status by remember { mutableStateOf<Int?>(null) }
    var checking by remember { mutableStateOf(false) }
    val hasKey = app.settings.data.hasSgdbKey
    val save: () -> Unit = {
        checking = true
        scope.launch {
            status = saveKey(app, key.trim())
            checking = false
            if (status == null) onClose()
        }
    }
    Dialog(onDismissRequest = onClose) {
        Column(
            Modifier.widthIn(max = DIALOG_WIDTH).background(theme.cardEnd, RoundedCornerShape(20.dp)).padding(24.dp),
            verticalArrangement = Arrangement.spacedBy(14.dp),
        ) {
            Text(stringResource(R.string.st_steamgrid), color = theme.textPrimary, fontSize = 22.sp)
            Text(stringResource(R.string.st_sgdb_key_help), color = theme.textSecondary, fontSize = 14.sp)
            OutlinedTextField(
                key,
                { key = it; status = null },
                Modifier.fillMaxWidth(),
                singleLine = true,
                label = { Text(stringResource(R.string.st_sgdb_key_label)) },
                visualTransformation = PasswordVisualTransformation(),
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password),
            )
            status?.let { Text(stringResource(it), color = theme.accentStart, fontSize = 14.sp) }
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
                TextButton({ clipboard.getText()?.text?.let { key = it.trim(); status = null } }) {
                    Text(stringResource(R.string.st_sgdb_paste))
                }
                if (hasKey) {
                    TextButton({ scope.launch { removeKey(app); onClose() } }) { Text(stringResource(R.string.st_sgdb_remove)) }
                }
                TextButton(onClose) { Text(stringResource(R.string.cancel)) }
                TextButton(save, enabled = key.isNotBlank() && !checking) {
                    Text(stringResource(if (checking) R.string.st_sgdb_checking else R.string.st_sgdb_save))
                }
            }
        }
    }
}

/**
 * Checks [key] and stores it when SteamGridDB accepts it or cannot be reached; returns the text to show under the
 * field, or `null` when the key was stored.
 */
private suspend fun saveKey(app: AppViewModel, key: String): Int? {
    val check = app.zl.core.call { it.checkSteamgriddbKey(key) }
    if (check == KeyCheck.REFUSED) return R.string.msg_sgdb_key_refused
    return try {
        app.zl.core.call { it.setSteamgriddbKey(key) }
        app.message = UiMessage(if (check == KeyCheck.VALID) R.string.msg_sgdb_key_saved else R.string.msg_sgdb_key_saved_unchecked)
        app.settings.reload()
        null
    } catch (_: CoreException) {
        R.string.msg_sgdb_key_malformed
    }
}

/** Forgets the stored key. */
private suspend fun removeKey(app: AppViewModel) {
    try {
        app.zl.core.call { it.setSteamgriddbKey(null) }
        app.settings.reload()
    } catch (e: CoreException) {
        app.message = e.toMessage()
    }
}
