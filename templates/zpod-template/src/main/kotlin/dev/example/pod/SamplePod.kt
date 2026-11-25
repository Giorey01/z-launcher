package dev.example.pod

import android.content.Context
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Extension
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.zlauncher.pods.Pod
import dev.zlauncher.pods.PodHost
import dev.zlauncher.pods.PodProvider
import dev.zlauncher.pods.PodRoute
import dev.zlauncher.pods.PodSlot
import dev.zlauncher.ui.pad.PadButton
import dev.zlauncher.ui.theme.ZPanel

/** Sample Pod implementation adhering to Z-Launcher v2 Pod contract. */
class SamplePod : Pod {
    override val id: String = "com.example.sample"
    override val icon: ImageVector = Icons.Rounded.Extension

    override fun title(context: Context): String = "Sample Pod"

    override fun slots(): List<PodSlot> = listOf(
        PodSlot.StartMenu("Sample", Icons.Rounded.Extension, PodRoute("main")),
    )

    @Composable
    override fun Content(host: PodHost, route: PodRoute) {
        val theme = host.theme

        Box(
            Modifier.fillMaxSize()
                .background(theme.backgroundBrush())
                .padding(24.dp),
        ) {
            Column(Modifier.fillMaxSize(), verticalArrangement = Arrangement.spacedBy(16.dp)) {
                SampleHeader(host)
                SampleBody(host)
            }
        }
    }
}

@Composable
private fun SampleHeader(host: PodHost) {
    val theme = host.theme
    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
        Icon(Icons.Rounded.Extension, null, tint = theme.accentStart, modifier = Modifier.size(32.dp))
        Spacer(Modifier.width(12.dp))
        Column(Modifier.weight(1f)) {
            Text("Sample Pod", color = theme.textPrimary, fontSize = 22.sp, fontWeight = FontWeight.Bold)
            Text("Demonstrating Z-Launcher Pod SDK v2", color = theme.textSecondary, fontSize = 13.sp)
        }
        PadButton("Close", onClick = { host.close() })
    }
}

@Composable
private fun SampleBody(host: PodHost) {
    val theme = host.theme
    ZPanel(Modifier.fillMaxWidth(), RoundedCornerShape(16.dp), elevation = 4.dp) {
        Column(Modifier.fillMaxWidth().padding(20.dp)) {
            Text(
                "Welcome to your new Pod!",
                color = theme.textPrimary,
                fontSize = 18.sp,
                fontWeight = FontWeight.SemiBold,
            )
            Spacer(Modifier.height(8.dp))
            Text(
                "This screen renders within the Z-Launcher process using theme colours, gamepad focus rings, " +
                    "and native sound effects.",
                color = theme.textSecondary,
                fontSize = 14.sp,
            )
        }
    }
}

/** Entry point provider discovered by Z-Launcher via entry in manifest.json. */
class SamplePodProvider : PodProvider {
    override fun pods(): List<Pod> = listOf(SamplePod())
}
