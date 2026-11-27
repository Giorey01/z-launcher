package dev.zlauncher.ui

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import dev.zlauncher.nav.UiMessage

/** The text of a message in the current language (`stringResource` takes the placeholder values as a vararg). */
@Composable
@Suppress("SpreadOperator")
fun UiMessage.text(): String = literal ?: stringResource(res, *args.toTypedArray())
