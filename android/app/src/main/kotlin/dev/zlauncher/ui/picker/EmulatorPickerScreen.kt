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
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.zlauncher.R
import dev.zlauncher.core.nativecore.Candidate
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.ui.pad.FocusRestorer
import dev.zlauncher.ui.pad.PadTile
import dev.zlauncher.ui.theme.TextPrimary
import dev.zlauncher.ui.theme.TextSecondary

/** Focus key of a candidate row. */
private fun key(candidate: Candidate) = "c:${candidate.profile.emulator}/${candidate.profile.profile}/${candidate.profile.core.orEmpty()}"

/**
 * The emulator picker (RF-08, RF-10): shown when no default is set and several emulators can launch the game.
 * The choice is saved as the platform's default and the game starts.
 */
@Composable
fun EmulatorPickerScreen(app: AppViewModel) {
    val choice = app.choice ?: return
    val memory = app.memory("picker")
    Column(Modifier.fillMaxSize().systemBarsPadding().padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
        Text(stringResource(R.string.picker_title, choice.game.platformName), style = MaterialTheme.typography.headlineSmall)
        Text(stringResource(R.string.picker_hint, choice.game.title), color = TextSecondary)
        LazyColumn(Modifier.fillMaxSize(), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            items(choice.candidates, key = ::key) { candidate ->
                PadTile({ app.choose(candidate) }, Modifier.fillMaxWidth().then(memory.track(key(candidate)))) {
                    Column(Modifier.padding(16.dp)) {
                        Text(candidate.emulatorName, color = TextPrimary, fontSize = 18.sp)
                        candidate.coreName?.let { Text(stringResource(R.string.picker_core, it), color = TextSecondary, fontSize = 14.sp) }
                    }
                }
            }
        }
    }
    FocusRestorer(memory, app.resumeTick, true, choice.candidates.firstOrNull()?.let(::key))
}
