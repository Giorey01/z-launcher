package dev.zlauncher.ui.onboarding

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBarsPadding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.unit.dp
import dev.zlauncher.nav.UiMessage
import dev.zlauncher.ui.pad.requestSoon
import dev.zlauncher.ui.theme.ErrorColor
import dev.zlauncher.ui.theme.TextSecondary
import dev.zlauncher.ui.text

/**
 * The layout every setup step shares: a title, an optional explanation, the step's [content] and
 * a line for the last error. [focus] is the element that gets the focus when the step appears or
 * when [focusKey] changes (for example when a button becomes available).
 */
@Composable
fun StepFrame(
    title: String,
    body: String?,
    focus: FocusRequester,
    focusKey: Any = Unit,
    error: UiMessage? = null,
    content: @Composable ColumnScope.() -> Unit,
) {
    LaunchedEffect(focusKey) { focus.requestSoon() }
    Column(
        Modifier.fillMaxSize().systemBarsPadding().padding(24.dp),
        verticalArrangement = Arrangement.spacedBy(14.dp, Alignment.CenterVertically),
    ) {
        Text(title, style = MaterialTheme.typography.headlineMedium)
        body?.let { Text(it, color = TextSecondary, style = MaterialTheme.typography.bodyLarge) }
        content()
        error?.let { Text(it.text(), color = ErrorColor) }
    }
}
