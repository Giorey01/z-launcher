package dev.zlauncher.ui.onboarding

import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.res.stringResource
import dev.zlauncher.R
import dev.zlauncher.nav.UiMessage
import dev.zlauncher.ui.pad.PadButton

/** Step 1: what the launcher is and what the setup will do. */
@Composable
fun WelcomeStep(onStart: () -> Unit) {
    val focus = remember { FocusRequester() }
    StepFrame(stringResource(R.string.ob_welcome_title), stringResource(R.string.ob_welcome_body), focus) {
        PadButton(stringResource(R.string.ob_start), onStart, Modifier.focusRequester(focus))
    }
}

/** Step 2: "All files access" (RNF-10), the main storage mode; the button opens the system screen. */
@Composable
fun PermissionStep(granted: Boolean, error: UiMessage?, onOpenSettings: () -> Unit, onContinue: () -> Unit) {
    val focus = remember { FocusRequester() }
    val body = stringResource(if (granted) R.string.ob_perm_granted else R.string.ob_perm_body)
    StepFrame(stringResource(R.string.ob_perm_title), body, focus, granted, error) {
        if (granted) {
            PadButton(stringResource(R.string.ob_continue), onContinue, Modifier.focusRequester(focus))
        } else {
            PadButton(stringResource(R.string.ob_perm_open), onOpenSettings, Modifier.focusRequester(focus))
        }
    }
}

/** Last step: the setup is finished. */
@Composable
fun DoneStep(onFinish: () -> Unit) {
    val focus = remember { FocusRequester() }
    StepFrame(stringResource(R.string.ob_done_title), stringResource(R.string.ob_done_body), focus) {
        PadButton(stringResource(R.string.ob_done_go), onFinish, Modifier.focusRequester(focus))
    }
}
