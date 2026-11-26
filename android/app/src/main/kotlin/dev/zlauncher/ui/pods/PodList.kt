package dev.zlauncher.ui.pods

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Extension
import androidx.compose.material.icons.rounded.FolderOpen
import androidx.compose.material.icons.rounded.Link
import androidx.compose.material.icons.rounded.PowerSettingsNew
import androidx.compose.material.icons.rounded.Refresh
import androidx.compose.material.icons.rounded.Storefront
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.zlauncher.R
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.Screen
import dev.zlauncher.pods.PodAction
import dev.zlauncher.pods.PodFocus
import dev.zlauncher.pods.PodManager
import dev.zlauncher.pods.PodRoute
import dev.zlauncher.pods.PodTab
import dev.zlauncher.pods.formatFileSize
import dev.zlauncher.ui.theme.LocalZTheme
import dev.zlauncher.ui.theme.ZPanel

/**
 * The rows of the active tab in one panel, as the settings list. The first row takes the focus
 * when the tab opens, so the pad always has somewhere to go.
 */
@Composable
internal fun PodList(app: AppViewModel, manager: PodManager, modifier: Modifier) {
    val first = remember(manager.state.activeTab) { FocusRequester() }
    LaunchedEffect(manager.state.activeTab) { runCatching { first.requestFocus() } }
    ZPanel(modifier, RoundedCornerShape(topStart = 18.dp, topEnd = 18.dp), elevation = 6.dp) {
        LazyColumn(Modifier.fillMaxSize()) {
            when (manager.state.activeTab) {
                PodTab.INSTALLED -> installedRows(manager, first)
                PodTab.BUILTIN -> builtinRows(app, manager, first)
                PodTab.STORE -> storeRows(manager, first)
            }
        }
    }
}

/** A thin line between two rows. */
@Composable
private fun RowDivider() {
    Box(Modifier.fillMaxWidth().padding(horizontal = 16.dp).height(1.dp).background(LocalZTheme.current.divider))
}

/** A centred hint in place of rows that are not there. */
@Composable
private fun EmptyHint(text: String) {
    Box(Modifier.fillMaxWidth().padding(horizontal = 24.dp, vertical = 28.dp), contentAlignment = Alignment.Center) {
        Text(text, color = LocalZTheme.current.textSecondary, fontSize = 15.sp, textAlign = TextAlign.Center)
    }
}

/** Installed tab: the external Pods switch, then one row per installed Pod. */
private fun LazyListScope.installedRows(manager: PodManager, first: FocusRequester) {
    item("external") {
        val on = manager.state.externalPodsEnabled
        val theme = LocalZTheme.current
        PodListRow(
            Icons.Rounded.PowerSettingsNew,
            stringResource(R.string.pod_external_toggle),
            stringResource(R.string.pod_external_toggle_sub),
            onClick = { manager.setExternalEnabled(!on) },
            onFocused = { manager.focus(PodFocus.Action(PodAction.EXTERNAL_TOGGLE)) },
            modifier = Modifier.focusRequester(first),
        ) {
            PodChip(stringResource(if (on) R.string.pod_switch_on else R.string.pod_switch_off), theme.accentStart, filled = on)
        }
    }
    val pods = manager.state.installedPods
    if (pods.isEmpty()) item("empty") { RowDivider(); EmptyHint(stringResource(R.string.pod_empty_installed)) }
    items(pods, key = { it.id }) { pod ->
        RowDivider()
        PodListRow(
            Icons.Rounded.Extension,
            pod.name,
            stringResource(R.string.pod_version_author, pod.version, pod.authorName),
            onClick = { manager.showActions(pod) },
            onFocused = { manager.focus(PodFocus.Installed(pod)) },
        ) { PodStateChip(pod.state) }
    }
}

/** Built-in tab: the Pods compiled into this build; A opens one. */
private fun LazyListScope.builtinRows(app: AppViewModel, manager: PodManager, first: FocusRequester) {
    val pods = manager.state.builtinPods
    if (pods.isEmpty()) item("empty") { EmptyHint(stringResource(R.string.pod_empty_builtin)) }
    items(pods, key = { it.pod.id }) { entry ->
        if (entry != pods.first()) RowDivider()
        val context = LocalContext.current
        PodListRow(
            entry.pod.icon,
            entry.pod.title(context),
            stringResource(R.string.pod_status_builtin),
            onClick = { app.open(Screen.Pod(entry.pod.id, PodRoute("main"))) },
            onFocused = { manager.focus(PodFocus.Builtin(entry)) },
            modifier = if (entry == pods.first()) Modifier.focusRequester(first) else Modifier,
        )
    }
}

/** Store tab: install from a file, from a link, reload; then the entries of the index. */
private fun LazyListScope.storeRows(manager: PodManager, first: FocusRequester) {
    item("file") {
        PodListRow(
            Icons.Rounded.FolderOpen, stringResource(R.string.pod_install_file), stringResource(R.string.pod_install_file_sub),
            onClick = { manager.requestFile() }, onFocused = { manager.focus(PodFocus.Action(PodAction.INSTALL_FILE)) },
            modifier = Modifier.focusRequester(first),
        )
    }
    item("url") {
        RowDivider()
        PodListRow(
            Icons.Rounded.Link, stringResource(R.string.pod_direct_url), stringResource(R.string.pod_install_url_sub),
            onClick = { manager.showUrlInput(true) }, onFocused = { manager.focus(PodFocus.Action(PodAction.INSTALL_URL)) },
        )
    }
    item("refresh") {
        RowDivider()
        PodListRow(
            Icons.Rounded.Refresh, stringResource(R.string.pod_refresh_store), stringResource(R.string.pod_refresh_store_sub),
            onClick = { manager.fetchStoreIndex() }, onFocused = { manager.focus(PodFocus.Action(PodAction.REFRESH_STORE)) },
        )
    }
    storeEntries(manager)
}

/** The entries of the store index, or a line that says why there are none. */
private fun LazyListScope.storeEntries(manager: PodManager) {
    val state = manager.state
    val hint = when {
        state.storeLoading -> R.string.pod_loading_index
        state.storeError -> R.string.pod_error_index
        state.storeEntries.isEmpty() -> R.string.pod_store_empty
        else -> null
    }
    if (hint != null) item("store-hint") { RowDivider(); EmptyHint(stringResource(hint)) }
    items(state.storeEntries, key = { "store-${it.id}" }) { entry ->
        RowDivider()
        val installed = state.installedPods.firstOrNull { it.id == entry.id }
        PodListRow(
            Icons.Rounded.Storefront,
            entry.name,
            "v${entry.version} · ${formatFileSize(entry.size.toLong())}",
            onClick = { if (entry.flag == null) manager.downloadAndInspect(entry.url, entry.sha256, entry.signer) },
            onFocused = { manager.focus(PodFocus.Store(entry)) },
        ) { StoreChip(entry.flag, installed?.version, entry.version) }
    }
}

/** On a store entry: incompatible, installed, update or install. */
@Composable
private fun StoreChip(flag: String?, installedVersion: String?, version: String) {
    val theme = LocalZTheme.current
    when {
        flag != null -> PodChip(stringResource(R.string.pod_status_incompatible_short), POD_WARNING)
        installedVersion == version -> PodChip(stringResource(R.string.pod_installed_badge), theme.textSecondary)
        installedVersion != null -> PodChip(stringResource(R.string.pod_update), theme.accentStart, filled = true)
        else -> PodChip(stringResource(R.string.pod_install), theme.accentStart, filled = true)
    }
}
