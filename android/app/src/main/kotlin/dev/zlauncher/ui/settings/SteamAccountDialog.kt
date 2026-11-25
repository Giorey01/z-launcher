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
import dev.zlauncher.core.nativecore.SteamProblem
import dev.zlauncher.launch.toMessage
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.UiMessage
import dev.zlauncher.ui.theme.LocalZTheme
import kotlinx.coroutines.launch

/** Width of the editor. */
private val DIALOG_WIDTH = 560.dp

/** The message of a Steam problem. */
fun steamProblemMessage(problem: SteamProblem): UiMessage = UiMessage(
    when (problem) {
        SteamProblem.NOT_SET -> R.string.msg_steam_not_set
        SteamProblem.KEY_REFUSED -> R.string.msg_steam_key_refused
        SteamProblem.PRIVATE_PROFILE -> R.string.msg_steam_private
        SteamProblem.UNKNOWN_PROFILE -> R.string.msg_steam_unknown_profile
        SteamProblem.OFFLINE -> R.string.msg_steam_offline
    },
)

/**
 * The Steam account editor: the Web API key (pasted, never shown back; left empty it keeps the stored one) and the
 * profile (SteamID64, profile link or custom name). Saving stores both and checks them with Steam; the result is the
 * message under the fields, or the editor closes when the library can be read.
 */
@Composable
fun SteamAccountDialog(app: AppViewModel, onClose: () -> Unit) {
    val theme = LocalZTheme.current
    val clipboard = LocalClipboardManager.current
    val scope = rememberCoroutineScope()
    val stored = app.settings.data.steam
    var key by remember { mutableStateOf("") }
    var account by remember { mutableStateOf(stored.account.orEmpty()) }
    var status by remember { mutableStateOf<UiMessage?>(null) }
    var checking by remember { mutableStateOf(false) }
    val save: () -> Unit = {
        checking = true
        scope.launch {
            status = saveAccount(app, key.trim().ifEmpty { null }, account.trim())
            checking = false
            if (status == null) onClose()
        }
    }
    Dialog(onDismissRequest = onClose) {
        Column(
            Modifier.widthIn(max = DIALOG_WIDTH).background(theme.cardEnd, RoundedCornerShape(20.dp)).padding(24.dp),
            verticalArrangement = Arrangement.spacedBy(14.dp),
        ) {
            Text(stringResource(R.string.st_steam), color = theme.textPrimary, fontSize = 22.sp)
            Text(stringResource(R.string.st_steam_help), color = theme.textSecondary, fontSize = 14.sp)
            OutlinedTextField(
                key,
                { key = it; status = null },
                Modifier.fillMaxWidth(),
                singleLine = true,
                label = { Text(stringResource(if (stored.hasKey) R.string.st_steam_key_label_set else R.string.st_steam_key_label)) },
                visualTransformation = PasswordVisualTransformation(),
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password),
            )
            OutlinedTextField(account, { account = it; status = null }, Modifier.fillMaxWidth(), singleLine = true,
                label = { Text(stringResource(R.string.st_steam_account_label)) })
            status?.let { Text(formatUiMessage(app.zl, it), color = theme.accentStart, fontSize = 14.sp) }
            Buttons(
                onPaste = { clipboard.getText()?.text?.let { key = it.trim(); status = null } },
                onClose = onClose,
                onSave = save,
                canSave = account.isNotBlank() && (stored.hasKey || key.isNotBlank()) && !checking,
                checking = checking,
            )
        }
    }
}

/** Paste (into the key field), Cancel and Save. */
@Composable
private fun Buttons(onPaste: () -> Unit, onClose: () -> Unit, onSave: () -> Unit, canSave: Boolean, checking: Boolean) {
    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
        TextButton(onPaste) { Text(stringResource(R.string.st_sgdb_paste)) }
        TextButton(onClose) { Text(stringResource(R.string.cancel)) }
        TextButton(onSave, enabled = canSave) {
            Text(stringResource(if (checking) R.string.st_sgdb_checking else R.string.st_sgdb_save))
        }
    }
}

/**
 * Stores [key] (`null` keeps the stored one) and [account], then checks them with Steam; returns the message to
 * show under the fields, or `null` when the library can be read.
 */
private suspend fun saveAccount(app: AppViewModel, key: String?, account: String): UiMessage? = try {
    app.zl.core.call { it.setSteamAccount(key, account) }
    app.settings.reload()
    val problem = app.zl.core.call { it.checkSteamAccount() }
    if (problem == null) {
        app.message = UiMessage(R.string.msg_steam_ready)
        null
    } else {
        steamProblemMessage(problem)
    }
} catch (_: CoreException.InvalidInput) {
    UiMessage(R.string.msg_steam_malformed)
} catch (e: CoreException) {
    e.toMessage()
}
