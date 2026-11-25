package dev.zlauncher.ui.pods

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.rounded.Add
import androidx.compose.material.icons.rounded.Check
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material.icons.rounded.Download
import androidx.compose.material.icons.rounded.Extension
import androidx.compose.material.icons.rounded.Link
import androidx.compose.material.icons.rounded.Refresh
import androidx.compose.material.icons.rounded.Warning
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import dev.zlauncher.R
import dev.zlauncher.core.nativecore.IndexEntryView
import dev.zlauncher.core.nativecore.InstalledPodView
import dev.zlauncher.core.nativecore.PodPreview
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.pods.PodManager
import dev.zlauncher.pods.PodTab
import dev.zlauncher.pods.PodWithMetadata
import dev.zlauncher.pods.formatFileSize
import dev.zlauncher.pods.formatSignerFingerprint
import dev.zlauncher.ui.drawer.GamepadKeyboard
import dev.zlauncher.ui.pad.PadButton
import dev.zlauncher.ui.pad.PadTile
import dev.zlauncher.ui.pad.TileSurface
import dev.zlauncher.ui.theme.LocalZTheme
import dev.zlauncher.ui.theme.ZPanel
import dev.zlauncher.ui.theme.ZTheme

/** Main Pod management and store screen (§11). */
@Composable
fun PodManagementScreen(app: AppViewModel) {
    val theme = LocalZTheme.current
    val manager = app.podManager
    val state = manager.state

    LaunchedEffect(Unit) { manager.reload() }

    Box(Modifier.fillMaxSize().background(theme.backgroundBrush()).padding(horizontal = 24.dp, vertical = 16.dp)) {
        Column(Modifier.fillMaxSize()) {
            PodHeader(app, manager)
            Spacer(Modifier.height(16.dp))
            when (state.activeTab) {
                PodTab.INSTALLED -> InstalledPodsTab(manager, state.installedPods)
                PodTab.BUILTIN -> BuiltinPodsTab(state.builtinPods)
                PodTab.STORE -> StoreTab(manager, state.storeEntries, state.installedPods)
            }
        }

        PodDialogs(manager)
    }
}

/** Header with title, tab selector, and action buttons. */
@Composable
private fun PodHeader(app: AppViewModel, manager: PodManager) {
    val theme = LocalZTheme.current
    val state = manager.state
    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
        PadTile({ app.back() }, modifier = Modifier.size(44.dp)) {
            Icon(Icons.AutoMirrored.Rounded.ArrowBack, contentDescription = null, tint = theme.textPrimary)
        }
        Spacer(Modifier.width(12.dp))
        Text(stringResource(R.string.pod_screen_title), color = theme.textPrimary, fontSize = 24.sp, fontWeight = FontWeight.Bold)
        Spacer(Modifier.width(24.dp))
        TabPill(stringResource(R.string.pod_tab_installed), state.activeTab == PodTab.INSTALLED) { manager.selectTab(PodTab.INSTALLED) }
        TabPill(stringResource(R.string.pod_tab_builtin), state.activeTab == PodTab.BUILTIN) { manager.selectTab(PodTab.BUILTIN) }
        TabPill(stringResource(R.string.pod_tab_store), state.activeTab == PodTab.STORE) { manager.selectTab(PodTab.STORE) }
        Spacer(Modifier.weight(1f))
        val toggleText = if (state.externalPodsEnabled) stringResource(R.string.pod_status_active) else stringResource(R.string.pod_status_disabled_user)
        PadButton("${stringResource(R.string.pod_external_toggle)}: $toggleText", onClick = {
            manager.setExternalEnabled(!state.externalPodsEnabled)
        })
        Spacer(Modifier.width(8.dp))
        PadTile({ manager.selectTab(PodTab.STORE) }, modifier = Modifier.size(44.dp)) {
            Icon(Icons.Rounded.Add, contentDescription = null, tint = theme.accentStart)
        }
    }
}

/** Selectable tab pill. */
@Composable
private fun TabPill(label: String, selected: Boolean, onClick: () -> Unit) {
    val theme = LocalZTheme.current
    PadTile(onClick, modifier = Modifier.padding(horizontal = 4.dp)) {
        val bg = if (selected) theme.accentStart.copy(alpha = 0.25f) else Color.Transparent
        val border = if (selected) theme.accentStart else theme.textPrimary.copy(alpha = 0.2f)
        Box(
            Modifier.background(bg, RoundedCornerShape(16.dp)).border(1.dp, border, RoundedCornerShape(16.dp))
                .padding(horizontal = 14.dp, vertical = 8.dp)
        ) {
            Text(label, color = if (selected) theme.accentStart else theme.textSecondary, fontSize = 14.sp, fontWeight = FontWeight.Medium)
        }
    }
}

/** Tab displaying installed external Pods. */
@Composable
private fun InstalledPodsTab(manager: PodManager, pods: List<InstalledPodView>) {
    val theme = LocalZTheme.current
    if (pods.isEmpty()) {
        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            Text(stringResource(R.string.pod_empty_installed), color = theme.textSecondary, fontSize = 16.sp)
        }
    } else {
        LazyColumn(Modifier.fillMaxSize(), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            items(pods, key = { it.id }) { pod ->
                InstalledPodRow(manager, pod)
            }
        }
    }
}

/** Row for an installed Pod. */
@Composable
private fun InstalledPodRow(manager: PodManager, pod: InstalledPodView) {
    val theme = LocalZTheme.current
    ZPanel(Modifier.fillMaxWidth(), RoundedCornerShape(14.dp), elevation = 2.dp) {
        Row(Modifier.fillMaxWidth().padding(14.dp), verticalAlignment = Alignment.CenterVertically) {
            Icon(Icons.Rounded.Extension, contentDescription = null, tint = theme.accentStart, modifier = Modifier.size(36.dp))
            Spacer(Modifier.width(14.dp))
            Column(Modifier.weight(1f)) {
                Text(pod.name, color = theme.textPrimary, fontSize = 18.sp, fontWeight = FontWeight.SemiBold)
                Text(
                    stringResource(R.string.pod_version_author, pod.version, pod.authorName),
                    color = theme.textSecondary, fontSize = 13.sp, maxLines = 1, overflow = TextOverflow.Ellipsis
                )
            }
            StatusBadge(pod.state)
            Spacer(Modifier.width(12.dp))
            PodActions(manager, pod)
        }
    }
}

/** Action buttons for an installed Pod. */
@Composable
private fun PodActions(manager: PodManager, pod: InstalledPodView) {
    when (pod.state) {
        "active" -> PadButton(stringResource(R.string.pod_disable), onClick = { manager.setPodEnabled(pod.id, false) })
        "disabled_by_user" -> PadButton(stringResource(R.string.pod_enable), onClick = { manager.setPodEnabled(pod.id, true) })
        "disabled_by_crashes" -> PadButton(stringResource(R.string.pod_reset_health), onClick = { manager.resetHealth(pod.id) })
    }
    Spacer(Modifier.width(6.dp))
    PadTile({ manager.confirmUninstall(pod) }, modifier = Modifier.size(40.dp)) {
        Icon(Icons.Rounded.Delete, contentDescription = null, tint = LocalZTheme.current.textSecondary)
    }
}

/** Status badge matching Pod state. */
@Composable
private fun StatusBadge(state: String) {
    val theme = LocalZTheme.current
    val (textRes, color) = when (state) {
        "active" -> R.string.pod_status_active to theme.accentStart
        "disabled_by_user" -> R.string.pod_status_disabled_user to theme.textSecondary
        "disabled_by_crashes" -> R.string.pod_status_disabled_crashes to Color(0xFFEF5350)
        else -> R.string.pod_status_incompatible to Color(0xFFEF5350)
    }
    Box(
        Modifier.background(color.copy(alpha = 0.15f), RoundedCornerShape(8.dp))
            .border(1.dp, color.copy(alpha = 0.4f), RoundedCornerShape(8.dp)).padding(horizontal = 10.dp, vertical = 4.dp)
    ) {
        Text(stringResource(textRes), color = color, fontSize = 12.sp, fontWeight = FontWeight.Medium)
    }
}

/** Tab displaying built-in Pods. */
@Composable
private fun BuiltinPodsTab(pods: List<PodWithMetadata>) {
    val theme = LocalZTheme.current
    val context = LocalContext.current
    if (pods.isEmpty()) {
        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            Text(stringResource(R.string.pod_empty_builtin), color = theme.textSecondary, fontSize = 16.sp)
        }
    } else {
        LazyColumn(Modifier.fillMaxSize(), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            items(pods, key = { it.pod.id }) { item ->
                ZPanel(Modifier.fillMaxWidth(), RoundedCornerShape(14.dp), elevation = 2.dp) {
                    Row(Modifier.fillMaxWidth().padding(14.dp), verticalAlignment = Alignment.CenterVertically) {
                        Icon(item.pod.icon, contentDescription = null, tint = theme.accentStart, modifier = Modifier.size(36.dp))
                        Spacer(Modifier.width(14.dp))
                        Column(Modifier.weight(1f)) {
                            Text(item.pod.title(context), color = theme.textPrimary, fontSize = 18.sp, fontWeight = FontWeight.SemiBold)
                            Text("ID: ${item.pod.id}", color = theme.textSecondary, fontSize = 13.sp)
                        }
                        StatusBadge("builtin")
                    }
                }
            }
        }
    }
}

/** Store tab with remote index and direct URL action. */
@Composable
private fun StoreTab(manager: PodManager, entries: List<IndexEntryView>, installed: List<InstalledPodView>) {
    val theme = LocalZTheme.current
    Column(Modifier.fillMaxSize()) {
        Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
            PadButton(stringResource(R.string.pod_direct_url), onClick = { manager.showUrlInput(true) })
            Spacer(Modifier.width(8.dp))
            PadTile({ manager.fetchStoreIndex() }, modifier = Modifier.size(40.dp)) {
                Icon(Icons.Rounded.Refresh, contentDescription = null, tint = theme.accentStart)
            }
        }
        Spacer(Modifier.height(12.dp))
        if (manager.state.storeLoading) {
            Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                CircularProgressIndicator(color = theme.accentStart)
            }
        } else if (entries.isEmpty()) {
            Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                Text(stringResource(R.string.pod_store_empty), color = theme.textSecondary, fontSize = 16.sp)
            }
        } else {
            LazyColumn(Modifier.fillMaxSize(), verticalArrangement = Arrangement.spacedBy(10.dp)) {
                items(entries, key = { it.id }) { entry ->
                    StoreEntryRow(manager, entry, installed.firstOrNull { it.id == entry.id })
                }
            }
        }
    }
}

/** Row for a store entry. */
@Composable
private fun StoreEntryRow(manager: PodManager, entry: IndexEntryView, current: InstalledPodView?) {
    val theme = LocalZTheme.current
    ZPanel(Modifier.fillMaxWidth(), RoundedCornerShape(14.dp), elevation = 2.dp) {
        Row(Modifier.fillMaxWidth().padding(14.dp), verticalAlignment = Alignment.CenterVertically) {
            Icon(Icons.Rounded.Download, contentDescription = null, tint = theme.accentStart, modifier = Modifier.size(36.dp))
            Spacer(Modifier.width(14.dp))
            Column(Modifier.weight(1f)) {
                Text(entry.name, color = theme.textPrimary, fontSize = 18.sp, fontWeight = FontWeight.SemiBold)
                Text("v${entry.version} · ${formatFileSize(entry.size.toLong())}", color = theme.textSecondary, fontSize = 13.sp)
                entry.description?.let { desc ->
                    Text(desc, color = theme.textPrimary.copy(alpha = 0.7f), fontSize = 13.sp, maxLines = 2, overflow = TextOverflow.Ellipsis)
                }
            }
            StoreEntryAction(manager, entry, current)
        }
    }
}

/** Action button for a store entry. */
@Composable
private fun StoreEntryAction(manager: PodManager, entry: IndexEntryView, current: InstalledPodView?) {
    val theme = LocalZTheme.current
    if (entry.flag != null) {
        Text(stringResource(R.string.pod_incompatible_api), color = Color(0xFFEF5350), fontSize = 13.sp)
    } else if (current != null && current.version == entry.version) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Icon(Icons.Rounded.Check, null, tint = theme.accentStart, modifier = Modifier.size(16.dp))
            Text(stringResource(R.string.pod_installed_badge), color = theme.accentStart, fontSize = 14.sp)
        }
    } else {
        val label = if (current != null) stringResource(R.string.pod_update) else stringResource(R.string.pod_install)
        PadButton(label, onClick = { manager.downloadAndInspect(entry.url, entry.sha256, entry.signer) })
    }
}

/** All active modal dialogs. */
@Composable
private fun PodDialogs(manager: PodManager) {
    val state = manager.state
    if (state.showExternalPrompt) {
        ExternalPromptDialog(onConfirm = { manager.onExternalPromptConfirmed() }, onDismiss = { manager.onExternalPromptDismissed() })
    }
    state.pendingPreview?.let { preview ->
        InspectConfirmDialog(preview, onConfirm = { manager.confirmPendingInstall() }, onCancel = { manager.cancelPendingInstall() })
    }
    state.showUninstallConfirm?.let { pod ->
        UninstallDialog(pod, onConfirm = { manager.uninstall(pod.id) }, onCancel = { manager.confirmUninstall(null) })
    }
    if (state.showUrlDialog) {
        UrlInputDialog(onConfirm = { url -> manager.downloadAndInspect(url); manager.showUrlInput(false) }, onCancel = { manager.showUrlInput(false) })
    }
}

/** Dialog prompting the user to enable external Pods (§6). */
@Composable
private fun ExternalPromptDialog(onConfirm: () -> Unit, onDismiss: () -> Unit) {
    val theme = LocalZTheme.current
    Dialog(onDismissRequest = onDismiss) {
        ZPanel(Modifier.widthIn(max = 480.dp), RoundedCornerShape(20.dp), elevation = 8.dp) {
            Column(Modifier.padding(24.dp), verticalArrangement = Arrangement.spacedBy(14.dp)) {
                Text(stringResource(R.string.pod_external_prompt_title), color = theme.textPrimary, fontSize = 20.sp, fontWeight = FontWeight.Bold)
                Text(stringResource(R.string.pod_external_prompt_msg), color = theme.textSecondary, fontSize = 14.sp)
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
                    PadButton(stringResource(R.string.cancel), onClick = onDismiss)
                    Spacer(Modifier.width(10.dp))
                    PadButton(stringResource(R.string.pod_external_enable_action), onClick = onConfirm)
                }
            }
        }
    }
}

/** Confirmation dialog showing package preview, permissions, and security warning (§11). */
@Composable
private fun InspectConfirmDialog(preview: PodPreview, onConfirm: () -> Unit, onCancel: () -> Unit) {
    val theme = LocalZTheme.current
    Dialog(onDismissRequest = onCancel) {
        ZPanel(Modifier.widthIn(max = 520.dp), RoundedCornerShape(20.dp), elevation = 8.dp) {
            Column(Modifier.padding(24.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                Text(preview.name, color = theme.textPrimary, fontSize = 22.sp, fontWeight = FontWeight.Bold)
                Text("v${preview.version} · ${preview.authorName}", color = theme.textSecondary, fontSize = 14.sp)
                Text(stringResource(R.string.pod_size, formatFileSize(preview.uncompressedSize.toLong())), color = theme.textSecondary, fontSize = 13.sp)
                Text(stringResource(R.string.pod_fingerprint, formatSignerFingerprint(preview.signer)), color = theme.textSecondary, fontSize = 12.sp)

                Text(stringResource(R.string.pod_permissions_heading), color = theme.textPrimary, fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                PermissionsList(preview.permissions)

                SecurityWarningBox(theme)

                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
                    PadButton(stringResource(R.string.cancel), onClick = onCancel)
                    Spacer(Modifier.width(10.dp))
                    PadButton(stringResource(R.string.pod_install), onClick = onConfirm)
                }
            }
        }
    }
}

/** Formatted permissions list. */
@Composable
private fun PermissionsList(permissions: List<String>) {
    val theme = LocalZTheme.current
    if (permissions.isEmpty()) {
        Text(stringResource(R.string.pod_perm_none), color = theme.textSecondary, fontSize = 13.sp)
    } else {
        Column(verticalArrangement = Arrangement.spacedBy(3.dp)) {
            for (p in permissions) {
                val text = when (p) {
                    "library.read" -> stringResource(R.string.pod_perm_library_read)
                    "library.rescan" -> stringResource(R.string.pod_perm_library_rescan)
                    "launch" -> stringResource(R.string.pod_perm_launch)
                    "secrets" -> stringResource(R.string.pod_perm_secrets)
                    "http" -> stringResource(R.string.pod_perm_http)
                    "folders" -> stringResource(R.string.pod_perm_folders)
                    "audio" -> stringResource(R.string.pod_perm_audio)
                    else -> "• $p"
                }
                Text(text, color = theme.textSecondary, fontSize = 13.sp)
            }
        }
    }
}

/** Security warning box matching Cocoon glass styling (§11). */
@Composable
private fun SecurityWarningBox(theme: ZTheme) {
    Box(
        Modifier.fillMaxWidth().background(Color(0xFFEF5350).copy(alpha = 0.12f), RoundedCornerShape(10.dp))
            .border(1.dp, Color(0xFFEF5350).copy(alpha = 0.35f), RoundedCornerShape(10.dp)).padding(12.dp)
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Icon(Icons.Rounded.Warning, null, tint = Color(0xFFEF5350), modifier = Modifier.size(24.dp))
            Spacer(Modifier.width(10.dp))
            Text(stringResource(R.string.pod_install_warning), color = theme.textPrimary, fontSize = 13.sp, fontWeight = FontWeight.Medium)
        }
    }
}

/** Dialog confirming Pod uninstallation. */
@Composable
private fun UninstallDialog(pod: InstalledPodView, onConfirm: () -> Unit, onCancel: () -> Unit) {
    val theme = LocalZTheme.current
    Dialog(onDismissRequest = onCancel) {
        ZPanel(Modifier.widthIn(max = 460.dp), RoundedCornerShape(20.dp), elevation = 8.dp) {
            Column(Modifier.padding(24.dp), verticalArrangement = Arrangement.spacedBy(14.dp)) {
                Text(stringResource(R.string.pod_uninstall), color = theme.textPrimary, fontSize = 20.sp, fontWeight = FontWeight.Bold)
                Text(stringResource(R.string.pod_uninstall_confirm, pod.name), color = theme.textSecondary, fontSize = 14.sp)
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
                    PadButton(stringResource(R.string.cancel), onClick = onCancel)
                    Spacer(Modifier.width(10.dp))
                    PadButton(stringResource(R.string.pod_uninstall), onClick = onConfirm)
                }
            }
        }
    }
}

/** URL input dialog with custom on-screen keyboard. */
@Composable
private fun UrlInputDialog(onConfirm: (String) -> Unit, onCancel: () -> Unit) {
    val theme = LocalZTheme.current
    var url by remember { mutableStateOf("") }
    Dialog(onDismissRequest = onCancel) {
        ZPanel(Modifier.widthIn(max = 680.dp), RoundedCornerShape(22.dp), elevation = 8.dp) {
            Column(Modifier.padding(20.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                Text(stringResource(R.string.pod_direct_url), color = theme.textPrimary, fontSize = 20.sp, fontWeight = FontWeight.Bold)
                OutlinedTextField(
                    url, { url = it }, modifier = Modifier.fillMaxWidth(),
                    placeholder = { Text(stringResource(R.string.pod_url_hint)) },
                    singleLine = true, leadingIcon = { Icon(Icons.Rounded.Link, null) }
                )
                GamepadKeyboard(query = url, onQueryChange = { url = it }, onConfirm = { onConfirm(url.trim()) }, onClose = onCancel)
            }
        }
    }
}
