package dev.zlauncher.ui.pods

import androidx.compose.foundation.background
import androidx.compose.foundation.focusGroup
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.focusProperties
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.zlauncher.R
import dev.zlauncher.core.nativecore.InstalledPodView
import dev.zlauncher.core.nativecore.PodPreview
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.Screen
import dev.zlauncher.pods.PodManager
import dev.zlauncher.pods.PodRoute
import dev.zlauncher.pods.PodSlot
import dev.zlauncher.pods.formatFileSize
import dev.zlauncher.pods.formatSignerFingerprint
import dev.zlauncher.ui.drawer.GamepadKeyboard
import dev.zlauncher.ui.pad.PadButton
import dev.zlauncher.ui.theme.LocalZTheme
import dev.zlauncher.ui.theme.ZPanel

/** Opacity of the dim layer behind a dialog. */
private const val SCRIM_ALPHA = 0.45f

/** All the dialogs of the Pod screen, at most one at a time. */
@Composable
internal fun PodDialogs(app: AppViewModel, manager: PodManager) {
    val state = manager.state
    when {
        state.showExternalPrompt -> ExternalPromptDialog(manager)
        state.pendingPreview != null -> InspectConfirmDialog(state.pendingPreview, manager)
        state.showUninstallConfirm != null -> UninstallDialog(state.showUninstallConfirm, manager)
        state.actionsFor != null -> PodActionsDialog(app, state.actionsFor, manager)
        state.showUrlDialog -> UrlInputDialog(manager)
    }
}

/**
 * A dialog card with a title over a dim layer, drawn in the screen's own window: a system dialog
 * window on a secondary display does not get the gamepad keys. The focus cannot leave the card,
 * B closes it (see [PodManager.dismissDialog]) and a tap outside does too.
 */
@OptIn(ExperimentalComposeUiApi::class)
@Composable
private fun PodDialog(title: String, onDismiss: () -> Unit, maxWidth: Dp = 480.dp, content: @Composable ColumnScope.() -> Unit) {
    val theme = LocalZTheme.current
    Box(
        Modifier.fillMaxSize().background(Color.Black.copy(alpha = SCRIM_ALPHA)).pointerInput(Unit) { detectTapGestures { onDismiss() } },
        contentAlignment = Alignment.Center,
    ) {
        ZPanel(
            Modifier.padding(16.dp).widthIn(max = maxWidth).pointerInput(Unit) { detectTapGestures { } },
            RoundedCornerShape(22.dp),
            elevation = 8.dp,
        ) {
            Column(
                Modifier.focusProperties { exit = { FocusRequester.Cancel } }.focusGroup()
                    .padding(24.dp).verticalScroll(rememberScrollState()),
                verticalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                Text(title, color = theme.textPrimary, fontSize = 21.sp, fontWeight = FontWeight.Bold)
                content()
            }
        }
    }
}

/** Cancel and a confirm button on the right; the confirm one takes the focus. */
@Composable
private fun DialogButtons(confirm: String, onConfirm: () -> Unit, onCancel: () -> Unit) {
    val focus = remember { FocusRequester() }
    LaunchedEffect(Unit) { runCatching { focus.requestFocus() } }
    Row(Modifier.fillMaxWidth().padding(top = 6.dp), horizontalArrangement = Arrangement.End) {
        PadButton(stringResource(R.string.cancel), onClick = onCancel)
        Spacer(Modifier.width(10.dp))
        PadButton(confirm, onClick = onConfirm, modifier = Modifier.focusRequester(focus))
    }
}

/** Asks to turn on external Pods (§6). */
@Composable
private fun ExternalPromptDialog(manager: PodManager) {
    PodDialog(stringResource(R.string.pod_external_prompt_title), manager::onExternalPromptDismissed) {
        Text(stringResource(R.string.pod_external_prompt_msg), color = LocalZTheme.current.textSecondary, fontSize = 15.sp)
        DialogButtons(stringResource(R.string.pod_external_enable_action), manager::onExternalPromptConfirmed, manager::onExternalPromptDismissed)
    }
}

/** Before installing: name, version, author, fingerprint, permissions, size and the warning (§11). */
@Composable
private fun InspectConfirmDialog(preview: PodPreview, manager: PodManager) {
    val theme = LocalZTheme.current
    PodDialog(preview.name, manager::cancelPendingInstall, maxWidth = 560.dp) {
        Text(stringResource(R.string.pod_version_author, preview.version, preview.authorName), color = theme.textSecondary, fontSize = 15.sp)
        Text(stringResource(R.string.pod_size, formatFileSize(preview.uncompressedSize.toLong())), color = theme.textSecondary, fontSize = 14.sp)
        Text(stringResource(R.string.pod_fingerprint, formatSignerFingerprint(preview.signer)), color = theme.textSecondary, fontSize = 14.sp)
        Text(stringResource(R.string.pod_details_permissions), color = theme.textPrimary, fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
        if (preview.permissions.isEmpty()) Text(stringResource(R.string.pod_perm_none), color = theme.textSecondary, fontSize = 14.sp)
        for (p in preview.permissions) Text(permissionText(p), color = theme.textSecondary, fontSize = 14.sp)
        SecurityWarning()
        DialogButtons(stringResource(R.string.pod_install), manager::confirmPendingInstall, manager::cancelPendingInstall)
    }
}

/** Confirms the uninstall of [pod]. */
@Composable
private fun UninstallDialog(pod: InstalledPodView, manager: PodManager) {
    PodDialog(stringResource(R.string.pod_uninstall), { manager.confirmUninstall(null) }) {
        Text(stringResource(R.string.pod_uninstall_confirm, pod.name), color = LocalZTheme.current.textSecondary, fontSize = 15.sp)
        DialogButtons(stringResource(R.string.pod_uninstall), { manager.uninstall(pod.id) }, { manager.confirmUninstall(null) })
    }
}

/** What can be done with an installed Pod: open, turn on or off, reactivate after crashes, uninstall. */
@Composable
private fun PodActionsDialog(app: AppViewModel, pod: InstalledPodView, manager: PodManager) {
    val close = { manager.showActions(null) }
    val loaded = app.zl.pods.findEntry(pod.id)
    val first = remember { FocusRequester() }
    LaunchedEffect(Unit) { runCatching { first.requestFocus() } }
    PodDialog(pod.name, close, maxWidth = 420.dp) {
        val wide = Modifier.fillMaxWidth()
        if (pod.state == "active" && loaded != null) {
            val route = loaded.pod.slots().filterIsInstance<PodSlot.StartMenu>().firstOrNull()?.route ?: PodRoute("main")
            PadButton(stringResource(R.string.pod_open), { close(); app.open(Screen.Pod(pod.id, route)) }, wide.focusRequester(first))
        }
        val firstOr = if (pod.state == "active" && loaded != null) wide else wide.focusRequester(first)
        when (pod.state) {
            "active" -> PadButton(stringResource(R.string.pod_disable), { close(); manager.setPodEnabled(pod.id, false) }, firstOr)
            "disabled_by_user" -> PadButton(stringResource(R.string.pod_enable), { close(); manager.setPodEnabled(pod.id, true) }, firstOr)
            "disabled_by_crashes" -> PadButton(stringResource(R.string.pod_reset_health), { close(); manager.resetHealth(pod.id) }, firstOr)
            else -> Box(firstOr)
        }
        PadButton(stringResource(R.string.pod_uninstall), { close(); manager.confirmUninstall(pod) }, wide)
        PadButton(stringResource(R.string.cancel), close, wide)
    }
}

/** A link typed with the on-screen keyboard, shown in a field above it. */
@Composable
private fun UrlInputDialog(manager: PodManager) {
    val theme = LocalZTheme.current
    var url by remember { mutableStateOf("") }
    val cancel = { manager.showUrlInput(false) }
    PodDialog(stringResource(R.string.pod_direct_url), cancel, maxWidth = 680.dp) {
        val shown = url.ifEmpty { stringResource(R.string.pod_url_hint) }
        Box(Modifier.fillMaxWidth().background(theme.textPrimary.copy(alpha = 0.08f), RoundedCornerShape(12.dp)).padding(14.dp)) {
            Text(shown, color = if (url.isEmpty()) theme.textSecondary else theme.textPrimary, fontSize = 16.sp, maxLines = 2)
        }
        GamepadKeyboard(
            query = url,
            onQueryChange = { url = it },
            onConfirm = { manager.showUrlInput(false); manager.downloadAndInspect(url.trim()) },
            onClose = cancel,
        )
    }
}
