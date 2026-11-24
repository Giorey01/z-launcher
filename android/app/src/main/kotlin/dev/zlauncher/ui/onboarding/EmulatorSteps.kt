package dev.zlauncher.ui.onboarding

import androidx.compose.foundation.layout.Arrangement
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
import dev.zlauncher.R
import dev.zlauncher.core.nativecore.ProposedDefault
import dev.zlauncher.ui.pad.PadButton
import dev.zlauncher.ui.pad.PadTile
import dev.zlauncher.ui.theme.TextPrimary
import dev.zlauncher.ui.theme.TextSecondary

/** The emulator of a proposal, with the core when RetroArch has one. */
@Composable
private fun emulatorLabel(proposal: ProposedDefault): String {
    val candidate = proposal.candidate
    return candidate.coreName?.let { stringResource(R.string.ob_emu_with_core, candidate.emulatorName, it) } ?: candidate.emulatorName
}

/**
 * Step 6: the emulators found on the device and the default proposed for each platform of the library.
 * The rows can be focused so the list scrolls with the D-pad; the button saves the proposals.
 */
@Composable
fun EmulatorsStep(state: OnboardingUi, onDetect: () -> Unit, onAccept: () -> Unit, onSkip: () -> Unit) {
    val focus = remember { FocusRequester() }
    val detection = state.detection
    LaunchedEffect(Unit) { onDetect() }
    val body = when {
        detection == null -> null
        detection.proposed.isEmpty() -> stringResource(R.string.ob_emu_none)
        else -> pluralStringResource(R.plurals.ob_emu_found, detection.installed.size, detection.installed.size)
    }
    StepFrame(stringResource(R.string.ob_emu_title), body, focus, detection != null, state.error) {
        LazyColumn(Modifier.weight(1f, fill = false), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            items(detection?.proposed.orEmpty(), key = { it.platformId }) { proposal ->
                PadTile({}, Modifier.fillMaxWidth()) {
                    Text(
                        stringResource(R.string.ob_emu_row, proposal.platformName, emulatorLabel(proposal)),
                        color = TextPrimary,
                        modifier = Modifier.padding(12.dp),
                    )
                }
            }
        }
        if (detection?.proposed?.isNotEmpty() == true) {
            PadButton(stringResource(R.string.ob_emu_accept), onAccept, Modifier.focusRequester(focus))
        } else if (detection != null) {
            PadButton(stringResource(R.string.ob_continue), onSkip, Modifier.focusRequester(focus))
        } else {
            Text(stringResource(R.string.ob_emu_detecting), color = TextSecondary)
        }
    }
}

/** Step 7: offer to become the device's Home (RF-28); the system shows its own confirmation. */
@Composable
fun HomeRoleStep(held: Boolean, onRequest: () -> Unit, onSkip: () -> Unit) {
    val focus = remember { FocusRequester() }
    val body = stringResource(if (held) R.string.ob_role_already else R.string.ob_role_body)
    StepFrame(stringResource(R.string.ob_role_title), body, focus, held) {
        if (!held) PadButton(stringResource(R.string.ob_role_set), onRequest, Modifier.focusRequester(focus))
        val skipFocus = if (held) Modifier.focusRequester(focus) else Modifier
        PadButton(stringResource(if (held) R.string.ob_continue else R.string.ob_role_skip), onSkip, skipFocus)
    }
}
