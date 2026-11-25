package dev.zlauncher.pods.builtin

import android.content.Context
import androidx.compose.foundation.background
import androidx.compose.foundation.border
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
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Palette
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.zlauncher.R
import dev.zlauncher.pods.Pod
import dev.zlauncher.pods.PodHost
import dev.zlauncher.pods.PodRoute
import dev.zlauncher.pods.PodSlot
import dev.zlauncher.ui.pad.PadButton
import dev.zlauncher.ui.theme.ZPanel

/** Represents an entry in the Theme Hub catalog. */
data class ThemeHubEntry(
    /** Theme unique identifier. */
    val id: String,
    /** Theme display name. */
    val name: String,
    /** Author name. */
    val author: String,
    /** Semantic version. */
    val version: String,
    /** Background colour preview. */
    val background: Color,
    /** Accent colour preview. */
    val accent: Color,
    /** Surface card colour preview. */
    val surface: Color,
)

/** The built-in Theme Hub pod (§7, RF-50): theme catalog, previews, and installer. */
class ThemeHubPod : Pod {
    override val id: String = "builtin.themehub"
    override val icon: ImageVector = Icons.Rounded.Palette

    override fun title(context: Context): String = context.getString(R.string.theme_hub_title)

    override fun slots(): List<PodSlot> = listOf(
        PodSlot.StartMenu("Theme Hub", Icons.Rounded.Palette, PodRoute("main")),
        PodSlot.SettingsSection("Theme Hub", PodRoute("main")),
    )

    @Composable
    override fun Content(host: PodHost, route: PodRoute) {
        val theme = host.theme
        val themes by remember { mutableStateOf(sampleThemes()) }

        Box(
            Modifier.fillMaxSize().background(theme.backgroundBrush())
                .padding(horizontal = 24.dp, vertical = 20.dp),
        ) {
            Column(Modifier.fillMaxSize()) {
                ThemeHubHeader(host)
                Spacer(Modifier.height(16.dp))
                ThemeList(themes)
            }
        }
    }
}

@Suppress("MagicNumber")
private fun sampleThemes(): List<ThemeHubEntry> = listOf(
    ThemeHubEntry(
        id = "default",
        name = "Cocoon Classic",
        author = "Z-Launcher Team",
        version = "1.0.0",
        background = Color(0xFF121418),
        accent = Color(0xFF4A90E2),
        surface = Color(0xFF1E2228),
    ),
    ThemeHubEntry(
        id = "cyberpunk",
        name = "Neon Night",
        author = "Cocoon Community",
        version = "1.2.0",
        background = Color(0xFF0F051D),
        accent = Color(0xFFFF007F),
        surface = Color(0xFF24103E),
    ),
    ThemeHubEntry(
        id = "retro",
        name = "Onyx Slate",
        author = "RetroCraft",
        version = "1.1.0",
        background = Color(0xFF181818),
        accent = Color(0xFFF39C12),
        surface = Color(0xFF282828),
    ),
)

@Composable
private fun ThemeList(themes: List<ThemeHubEntry>) {
    LazyColumn(Modifier.fillMaxSize(), verticalArrangement = Arrangement.spacedBy(12.dp)) {
        items(themes, key = { it.id }) { item ->
            ThemeCard(item)
        }
    }
}

/** Header of the Theme Hub screen. */
@Composable
private fun ThemeHubHeader(host: PodHost) {
    val theme = host.theme
    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
        Icon(Icons.Rounded.Palette, null, tint = theme.accentStart, modifier = Modifier.size(32.dp))
        Spacer(Modifier.width(12.dp))
        Column(Modifier.weight(1f)) {
            Text(stringResource(R.string.theme_hub_title), color = theme.textPrimary, fontSize = 22.sp, fontWeight = FontWeight.Bold)
            Text(stringResource(R.string.theme_hub_sub), color = theme.textSecondary, fontSize = 13.sp)
        }
        PadButton(stringResource(R.string.cancel), onClick = { host.close() })
    }
}

/** Card previewing a theme in the catalog. */
@Composable
private fun ThemeCard(entry: ThemeHubEntry) {
    val theme = dev.zlauncher.ui.theme.LocalZTheme.current
    ZPanel(Modifier.fillMaxWidth(), RoundedCornerShape(16.dp), elevation = 4.dp) {
        Row(Modifier.fillMaxWidth().padding(16.dp), verticalAlignment = Alignment.CenterVertically) {
            Column(Modifier.weight(1f)) {
                Text(entry.name, color = theme.textPrimary, fontSize = 18.sp, fontWeight = FontWeight.SemiBold)
                Text("${entry.author} · v${entry.version}", color = theme.textSecondary, fontSize = 13.sp)
                Spacer(Modifier.height(8.dp))
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    ColorSwatch(entry.background)
                    ColorSwatch(entry.accent)
                    ColorSwatch(entry.surface)
                }
            }
            PadButton(stringResource(R.string.theme_hub_apply), onClick = { /* applied through launcher settings */ })
        }
    }
}

/** Circular color preview swatch. */
@Composable
private fun ColorSwatch(color: Color) {
    Box(
        Modifier.size(20.dp).background(color, CircleShape)
            .border(1.dp, Color.White.copy(alpha = 0.3f), CircleShape),
    )
}
