package dev.zlauncher.ui.pods

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Extension
import androidx.compose.material.icons.rounded.FolderOpen
import androidx.compose.material.icons.rounded.Link
import androidx.compose.material.icons.rounded.PowerSettingsNew
import androidx.compose.material.icons.rounded.Refresh
import androidx.compose.material.icons.rounded.Storefront
import androidx.compose.material.icons.rounded.Warning
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.zlauncher.R
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.pods.PodAction
import dev.zlauncher.pods.PodFocus
import dev.zlauncher.pods.formatFileSize
import dev.zlauncher.pods.formatSignerFingerprint
import dev.zlauncher.ui.hero.StatusPill
import dev.zlauncher.ui.theme.LocalBackdrop
import dev.zlauncher.ui.theme.LocalZTheme
import dev.zlauncher.ui.theme.ZPanel

/** Widest the details card grows on the big display. */
private val ASIDE_MAX_WIDTH = 760.dp

/**
 * The big display while the Pod screen is open (dual-screen mode): the details of the row under the
 * cursor in one card, the status pill in the top-right corner.
 */
@Composable
fun PodAside(app: AppViewModel) {
    val theme = LocalZTheme.current
    CompositionLocalProvider(LocalBackdrop provides null) {
        Box(Modifier.fillMaxSize().background(theme.backgroundBrush())) {
            PodDetails(
                app,
                app.podManager.state.focus,
                Modifier.align(Alignment.Center).widthIn(max = ASIDE_MAX_WIDTH).fillMaxWidth().padding(start = 24.dp, end = 24.dp, top = 66.dp, bottom = 16.dp),
            )
            StatusPill(Modifier.align(Alignment.TopEnd).padding(top = 10.dp, end = 16.dp))
        }
    }
}

/** The card that describes [focus]; with nothing under the cursor, a summary of the Pods. */
@Composable
internal fun PodDetails(app: AppViewModel, focus: PodFocus?, modifier: Modifier) {
    ZPanel(modifier, RoundedCornerShape(20.dp), elevation = 6.dp) {
        Column(Modifier.fillMaxWidth().verticalScroll(rememberScrollState()).padding(24.dp), verticalArrangement = Arrangement.spacedBy(14.dp)) {
            when (focus) {
                null -> Summary(app)
                is PodFocus.Installed -> InstalledDetails(focus)
                is PodFocus.Builtin -> BuiltinDetails(focus)
                is PodFocus.Store -> StoreDetails(focus)
                is PodFocus.Action -> ActionDetails(focus.action)
            }
        }
    }
}

/** Large icon, title and the line under it. */
@Composable
private fun DetailHeader(icon: ImageVector, title: String, subtitle: String?) {
    val theme = LocalZTheme.current
    Row(verticalAlignment = Alignment.CenterVertically) {
        Box(Modifier.size(64.dp).background(theme.accentStart.copy(alpha = 0.14f), RoundedCornerShape(18.dp)), contentAlignment = Alignment.Center) {
            Icon(icon, null, Modifier.size(36.dp), tint = theme.accentStart)
        }
        Column(Modifier.padding(start = 18.dp)) {
            Text(title, color = theme.textPrimary, fontSize = 24.sp, fontWeight = FontWeight.Bold, maxLines = 2)
            if (subtitle != null) Text(subtitle, color = theme.textSecondary, fontSize = 15.sp, fontWeight = FontWeight.Medium)
        }
    }
}

/** A label and its value, one under the other. */
@Composable
private fun Fact(label: String, value: String) {
    val theme = LocalZTheme.current
    Column {
        Text(label, color = theme.textPrimary.copy(alpha = 0.5f), fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
        Text(value, color = theme.textPrimary, fontSize = 15.sp, fontWeight = FontWeight.Medium)
    }
}

/** A paragraph in the secondary colour. */
@Composable
private fun Paragraph(text: String) {
    Text(text, color = LocalZTheme.current.textSecondary, fontSize = 15.sp)
}

/** The declared permissions, one line each (§11). */
@Composable
private fun Permissions(permissions: List<String>) {
    val theme = LocalZTheme.current
    Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
        Text(stringResource(R.string.pod_details_permissions), color = theme.textPrimary.copy(alpha = 0.5f), fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
        if (permissions.isEmpty()) Paragraph(stringResource(R.string.pod_perm_none))
        for (p in permissions) Text(permissionText(p), color = theme.textPrimary, fontSize = 14.sp)
    }
}

/** The warning every external Pod carries (§11). */
@Composable
internal fun SecurityWarning() {
    Row(
        Modifier.fillMaxWidth().background(POD_WARNING.copy(alpha = 0.10f), RoundedCornerShape(12.dp)).padding(12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Icon(Icons.Rounded.Warning, null, Modifier.size(22.dp), tint = POD_WARNING)
        Text(stringResource(R.string.pod_install_warning), Modifier.padding(start = 10.dp), color = LocalZTheme.current.textPrimary, fontSize = 13.sp)
    }
}

/** Nothing under the cursor yet: how many Pods there are. */
@Composable
private fun Summary(app: AppViewModel) {
    val state = app.podManager.state
    val active = state.installedPods.count { it.state == "active" }
    DetailHeader(Icons.Rounded.Extension, stringResource(R.string.pod_screen_title), stringResource(R.string.pod_screen_sub))
    Paragraph(stringResource(R.string.pod_summary, state.installedPods.size, active))
}

/** An installed external Pod. */
@Composable
private fun InstalledDetails(focus: PodFocus.Installed) {
    val pod = focus.pod
    DetailHeader(Icons.Rounded.Extension, pod.name, stringResource(R.string.pod_version_author, pod.version, pod.authorName))
    Row { PodStateChip(pod.state) }
    pod.description?.let { Paragraph(it) }
    Fact(stringResource(R.string.pod_details_signer), formatSignerFingerprint(pod.signer))
    Permissions(pod.permissions)
    Paragraph(stringResource(R.string.pod_details_hint_installed))
}

/** A Pod built into the launcher. */
@Composable
private fun BuiltinDetails(focus: PodFocus.Builtin) {
    val context = LocalContext.current
    DetailHeader(focus.entry.pod.icon, focus.entry.pod.title(context), stringResource(R.string.pod_status_builtin))
    Permissions(focus.entry.permissions.toList())
}

/** An entry of the store. */
@Composable
private fun StoreDetails(focus: PodFocus.Store) {
    val entry = focus.entry
    DetailHeader(Icons.Rounded.Storefront, entry.name, "v${entry.version} · ${formatFileSize(entry.size.toLong())}")
    entry.description?.let { Paragraph(it) }
    Fact(stringResource(R.string.pod_details_signer), formatSignerFingerprint(entry.signer))
    if (entry.flag != null) Paragraph(stringResource(R.string.pod_incompatible_api))
    SecurityWarning()
}

/** One of the action rows: what it does. */
@Composable
private fun ActionDetails(action: PodAction) {
    when (action) {
        PodAction.EXTERNAL_TOGGLE -> {
            DetailHeader(Icons.Rounded.PowerSettingsNew, stringResource(R.string.pod_external_toggle), null)
            Paragraph(stringResource(R.string.pod_external_details))
            SecurityWarning()
        }
        PodAction.INSTALL_FILE -> {
            DetailHeader(Icons.Rounded.FolderOpen, stringResource(R.string.pod_install_file), null)
            Paragraph(stringResource(R.string.pod_install_file_details))
        }
        PodAction.INSTALL_URL -> {
            DetailHeader(Icons.Rounded.Link, stringResource(R.string.pod_direct_url), null)
            Paragraph(stringResource(R.string.pod_install_url_details))
        }
        PodAction.REFRESH_STORE -> {
            DetailHeader(Icons.Rounded.Refresh, stringResource(R.string.pod_refresh_store), null)
            Paragraph(stringResource(R.string.pod_refresh_store_sub))
        }
    }
}
