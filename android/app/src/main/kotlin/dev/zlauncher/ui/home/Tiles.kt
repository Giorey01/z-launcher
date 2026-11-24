package dev.zlauncher.ui.home

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.pluralStringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.zlauncher.R
import dev.zlauncher.core.nativecore.GameEntry
import dev.zlauncher.core.nativecore.PlatformEntry
import dev.zlauncher.ui.pad.PadTile
import dev.zlauncher.ui.theme.TextPrimary
import dev.zlauncher.ui.theme.TextSecondary

/** Width over height of a platform tile. */
private const val PLATFORM_ASPECT = 1.35f

/** Width over height of a game tile. */
private const val GAME_ASPECT = 1.1f

/** Lines a game title may take in a tile. */
private const val TITLE_LINES = 3

/** The Smart Folder of one platform: its name and how many games it holds (artwork comes later). */
@Composable
fun PlatformTile(platform: PlatformEntry, onOpen: () -> Unit, modifier: Modifier = Modifier) {
    PadTile(onOpen, modifier.fillMaxWidth().aspectRatio(PLATFORM_ASPECT)) {
        Column(
            Modifier.align(Alignment.Center).padding(12.dp),
            verticalArrangement = Arrangement.spacedBy(6.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Text(platform.name, color = TextPrimary, fontSize = 18.sp, fontWeight = FontWeight.SemiBold, textAlign = TextAlign.Center)
            val games = platform.games.toInt()
            Text(pluralStringResource(R.plurals.games_count, games, games), color = TextSecondary, fontSize = 14.sp)
        }
    }
}

/** A game: its title, and in the Simple mode (where platforms mix) the platform name. */
@Composable
fun GameTile(game: GameEntry, showPlatform: Boolean, onLaunch: () -> Unit, modifier: Modifier = Modifier) {
    PadTile(onLaunch, modifier.fillMaxWidth().aspectRatio(GAME_ASPECT)) {
        Column(Modifier.align(Alignment.Center).padding(10.dp), horizontalAlignment = Alignment.CenterHorizontally) {
            Text(
                game.title,
                color = TextPrimary,
                fontSize = 15.sp,
                fontWeight = FontWeight.Medium,
                textAlign = TextAlign.Center,
                maxLines = TITLE_LINES,
                overflow = TextOverflow.Ellipsis,
            )
            if (showPlatform) Text(game.platformName, color = TextSecondary, fontSize = 12.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
        }
    }
}
