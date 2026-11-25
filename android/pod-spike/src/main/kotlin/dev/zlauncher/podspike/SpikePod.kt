package dev.zlauncher.podspike

import android.content.Context
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Star
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.zlauncher.pods.Pod
import dev.zlauncher.pods.PodHost
import dev.zlauncher.pods.PodProvider
import dev.zlauncher.ui.pad.PadButton
import dev.zlauncher.ui.pad.PadTile
import dev.zlauncher.ui.theme.LocalZTheme

class SpikePodProvider : PodProvider {
    override fun pods(): List<Pod> = listOf(SpikePod())
}

class SpikePod : Pod {
    override val id: String = "sample.spike"
    override val icon: ImageVector = Icons.Rounded.Star

    override fun title(context: Context): String = "Pod Spike"

    @Composable
    override fun Content(host: PodHost) {
        val theme = LocalZTheme.current
        var counter by remember { mutableIntStateOf(0) }
        var crashSync by remember { mutableStateOf(false) }
        var crashAsync by remember { mutableStateOf(false) }

        if (crashSync) {
            throw RuntimeException("Spike deliberate crash in Content()")
        }

        if (crashAsync) {
            LaunchedEffect(Unit) {
                throw RuntimeException("Spike deliberate crash in LaunchedEffect")
            }
        }

        val magic = remember {
            try {
                SpikeNative.load()
                SpikeNative.getMagicNumber()
            } catch (t: Throwable) {
                -999
            }
        }

        Column(
            modifier = Modifier
                .fillMaxSize()
                .background(theme.cardStart)
                .padding(24.dp),
            verticalArrangement = Arrangement.Center,
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Text(
                text = "POD SPIKE V2 RELOADED",
                color = theme.textPrimary,
                fontSize = 24.sp,
                fontWeight = FontWeight.Bold
            )
            Spacer(Modifier.height(12.dp))
            Text(
                text = "JNI libspike_native magic: $magic",
                color = theme.accentStart,
                fontSize = 18.sp
            )
            Spacer(Modifier.height(8.dp))
            Text(
                text = "Theme: ${theme.name} | TextPrimary: #${theme.textPrimary.value.toString(16)}",
                color = theme.textSecondary,
                fontSize = 14.sp
            )
            Spacer(Modifier.height(20.dp))
            Row(horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                PadButton(
                    label = "A: Tap me ($counter)",
                    onClick = { counter++ }
                )
                PadTile(
                    onClick = { counter += 10 },
                    modifier = Modifier.padding(4.dp)
                ) { focused ->
                    Box(Modifier.padding(14.dp)) {
                        Text(
                            "+10 Tile (focused=$focused)",
                            color = theme.textPrimary,
                            fontSize = 14.sp
                        )
                    }
                }
            }
            Spacer(Modifier.height(16.dp))
            Row(horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                PadButton(
                    label = "Crash in Content()",
                    onClick = { crashSync = true }
                )
                PadButton(
                    label = "Crash in LaunchedEffect",
                    onClick = { crashAsync = true }
                )
            }
            Spacer(Modifier.height(16.dp))
            Text(
                text = "Gamepad: A = Click, B = Back (torna al launcher)",
                color = theme.textSecondary,
                fontSize = 14.sp
            )
        }
    }
}
