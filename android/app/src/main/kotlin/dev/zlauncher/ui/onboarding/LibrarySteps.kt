package dev.zlauncher.ui.onboarding

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.res.pluralStringResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.zlauncher.R
import dev.zlauncher.core.nativecore.LibraryMode
import dev.zlauncher.ui.pad.PadButton
import dev.zlauncher.ui.pad.PadTile
import dev.zlauncher.ui.theme.TextPrimary
import dev.zlauncher.ui.theme.TextSecondary

/** A selectable row of text: the title and an optional second line. */
@Composable
private fun ChoiceRow(title: String, subtitle: String?, modifier: Modifier, onClick: () -> Unit) {
    PadTile(onClick, modifier.fillMaxWidth()) {
        Column(Modifier.padding(16.dp)) {
            Text(title, color = TextPrimary, fontSize = 18.sp)
            subtitle?.let { Text(it, color = TextSecondary, fontSize = 14.sp) }
        }
    }
}

/**
 * Step 3: ROM folders. Each folder on offer, and "another folder", opens the system folder picker (the
 * grant emulators need); folders already added are listed with a check. Continue appears once one exists.
 */
@Composable
fun FoldersStep(state: OnboardingUi, onPick: (String?) -> Unit, onContinue: () -> Unit) {
    val focus = remember { FocusRequester() }
    val added = state.roots.map { it.path }.toSet()
    StepFrame(stringResource(R.string.ob_folders_title), stringResource(R.string.ob_folders_body), focus, added.size, state.error) {
        LazyColumn(Modifier.weight(1f, fill = false), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            items(state.folders.filter { it !in added }, key = { it }) { path ->
                ChoiceRow(path, stringResource(R.string.ob_folder_offer), Modifier.focusRequester(focus)) { onPick(path) }
            }
            items(state.roots, key = { "root:${it.path}" }) { root ->
                ChoiceRow("✓ ${root.path}", stringResource(R.string.ob_folder_added), Modifier) { onPick(root.path) }
            }
            items(listOf("other")) { ChoiceRow(stringResource(R.string.ob_folder_other), null, Modifier) { onPick(null) } }
        }
        if (added.isNotEmpty()) PadButton(stringResource(R.string.ob_continue), onContinue, Modifier.focusRequester(focus))
    }
}

/** Step 4: Smart Folders or Simple mode. */
@Composable
fun ModeStep(onChoose: (LibraryMode) -> Unit) {
    val focus = remember { FocusRequester() }
    StepFrame(stringResource(R.string.ob_mode_title), stringResource(R.string.ob_mode_body), focus) {
        ChoiceRow(stringResource(R.string.ob_mode_smart), stringResource(R.string.ob_mode_smart_desc), Modifier.focusRequester(focus)) {
            onChoose(LibraryMode.SMART)
        }
        ChoiceRow(stringResource(R.string.ob_mode_simple), stringResource(R.string.ob_mode_simple_desc), Modifier) {
            onChoose(LibraryMode.SIMPLE)
        }
    }
}

/** Step 5: the scan, with the folder being read and the files seen; Continue when it ends. */
@Composable
fun ScanStep(state: OnboardingUi, onStart: () -> Unit, onContinue: () -> Unit) {
    val focus = remember { FocusRequester() }
    val scan = state.scan
    LaunchedEffect(Unit) { onStart() }
    StepFrame(stringResource(R.string.ob_scan_title), null, focus, scan.summary != null, state.error) {
        val summary = scan.summary
        if (scan.running || summary == null) {
            scan.platform?.let { Text(stringResource(R.string.ob_scan_folder, it, scan.index, scan.total), color = TextPrimary) }
            Text(pluralStringResource(R.plurals.ob_scan_files, scan.files.toInt(), scan.files.toInt()), color = TextSecondary)
        } else {
            val found = (summary.added + summary.updated + summary.unchanged + summary.restored).toInt()
            Text(pluralStringResource(R.plurals.ob_scan_done, found, found), color = TextPrimary, fontSize = 18.sp)
            PadButton(stringResource(R.string.ob_continue), onContinue, Modifier.focusRequester(focus))
        }
    }
}
