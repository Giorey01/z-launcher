package dev.zlauncher.ui.picker

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBarsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.zlauncher.R
import dev.zlauncher.core.nativecore.CoreException
import dev.zlauncher.core.nativecore.PlatformEntry
import dev.zlauncher.launch.toMessage
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.Screen
import dev.zlauncher.ui.pad.FocusRestorer
import dev.zlauncher.ui.pad.PadTile
import dev.zlauncher.ui.theme.TextPrimary
import dev.zlauncher.ui.theme.TextSecondary

/** Focus key of a console row. */
private fun key(console: PlatformEntry) = "k:${console.id}"

/**
 * The consoles an Android app can be filed under (a recomp or a port): those with games first, then the others.
 * A files the app under the focused console, B goes back to the drawer.
 */
@Composable
fun ConsolePickerScreen(app: AppViewModel, screen: Screen.ConsolePicker) {
    val memory = app.memory("consoles:${screen.packageName}")
    var consoles by remember { mutableStateOf<List<PlatformEntry>?>(null) }
    LaunchedEffect(screen.packageName) {
        consoles = try {
            app.zl.core.call { it.appConsoles() }
        } catch (e: CoreException) {
            app.message = e.toMessage()
            emptyList()
        }
    }
    val list = consoles.orEmpty()
    Column(Modifier.fillMaxSize().systemBarsPadding().padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
        Text(stringResource(R.string.console_picker_title, screen.label), style = MaterialTheme.typography.headlineSmall)
        Text(stringResource(R.string.console_picker_hint), color = TextSecondary)
        LazyColumn(Modifier.fillMaxSize(), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            items(list, key = ::key) { console ->
                PadTile(
                    { app.library.fileApp(screen.packageName, screen.label, console.id, console.name) },
                    Modifier.fillMaxWidth().then(memory.track(key(console))),
                ) {
                    Column(Modifier.padding(16.dp)) {
                        Text(console.name, color = TextPrimary, fontSize = 18.sp)
                        if (console.games > 0u) {
                            Text(stringResource(R.string.console_picker_games, console.games.toString()), color = TextSecondary, fontSize = 14.sp)
                        }
                    }
                }
            }
        }
    }
    FocusRestorer(memory, app.resumeTick, consoles != null, list.firstOrNull()?.let(::key))
}
