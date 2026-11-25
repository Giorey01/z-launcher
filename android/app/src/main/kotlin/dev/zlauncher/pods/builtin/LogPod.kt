package dev.zlauncher.pods.builtin

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
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.History
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.zlauncher.R
import dev.zlauncher.core.nativecore.GameEntry
import dev.zlauncher.pods.Pod
import dev.zlauncher.pods.PodHost
import dev.zlauncher.pods.PodRoute
import dev.zlauncher.pods.PodSlot
import dev.zlauncher.ui.pad.PadButton
import dev.zlauncher.ui.pad.PadTile
import dev.zlauncher.ui.theme.LocalZTheme
import dev.zlauncher.ui.theme.ZPanel

/** Game play summary record for the Log leaderboard. */
data class GameLogStat(
    /** Game entry. */
    val game: GameEntry,
    /** Total seconds played. */
    val secondsPlayed: Long,
)

/** Number of seconds in an hour. */
private const val SECONDS_IN_HOUR = 3600f

/** The built-in Log pod (§7, RF-42): play time totals, calendar, and game rankings. */
class LogPod : Pod {
    override val id: String = "builtin.log"
    override val icon: ImageVector = Icons.Rounded.History

    override fun title(context: Context): String = context.getString(R.string.pod_log_title)

    override fun slots(): List<PodSlot> = listOf(
        PodSlot.StartMenu("Log", Icons.Rounded.History, PodRoute("main")),
    )

    @Composable
    override fun Content(host: PodHost, route: PodRoute) {
        val theme = host.theme
        var stats by remember { mutableStateOf<List<GameLogStat>>(emptyList()) }
        var totalSeconds by remember { mutableStateOf(0L) }

        LaunchedEffect(Unit) {
            val games = host.games(null)
            val loadedStats = mutableListOf<GameLogStat>()
            var secondsSum = 0L
            for (g in games) {
                val playStats = host.playStats(g.id)
                val total = playStats.deviceSeconds + playStats.steamSeconds
                if (total > 0L) {
                    loadedStats += GameLogStat(g, total)
                    secondsSum += total
                }
            }
            stats = loadedStats.sortedByDescending { it.secondsPlayed }
            totalSeconds = secondsSum
        }

        Box(
            Modifier.fillMaxSize().background(theme.backgroundBrush())
                .padding(horizontal = 24.dp, vertical = 20.dp),
        ) {
            Column(Modifier.fillMaxSize()) {
                LogHeader(host, totalSeconds, stats.size)
                Spacer(Modifier.height(16.dp))
                LogBody(host, stats)
            }
        }
    }
}

@Composable
private fun LogBody(host: PodHost, stats: List<GameLogStat>) {
    val theme = host.theme
    if (stats.isEmpty()) {
        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            Text(stringResource(R.string.pod_log_no_stats), color = theme.textSecondary, fontSize = 16.sp)
        }
    } else {
        LazyColumn(Modifier.fillMaxSize(), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            itemsIndexed(stats, key = { _, item -> item.game.id }) { index, item ->
                LogItemRow(host, index + 1, item)
            }
        }
    }
}

/** Header displaying total stats and close button. */
@Composable
private fun LogHeader(host: PodHost, totalSeconds: Long, playedGamesCount: Int) {
    val theme = host.theme
    val hours = totalSeconds.toFloat() / SECONDS_IN_HOUR
    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
        Icon(Icons.Rounded.History, null, tint = theme.accentStart, modifier = Modifier.size(32.dp))
        Spacer(Modifier.width(12.dp))
        Column(Modifier.weight(1f)) {
            Text(stringResource(R.string.pod_log_title), color = theme.textPrimary, fontSize = 22.sp, fontWeight = FontWeight.Bold)
            val sub = "${stringResource(R.string.pod_log_total_time)}: ${stringResource(R.string.pod_log_hours, hours)} · " +
                "$playedGamesCount ${stringResource(R.string.pod_log_played_games)}"
            Text(sub, color = theme.textSecondary, fontSize = 13.sp)
        }
        PadButton(stringResource(R.string.cancel), onClick = { host.close() })
    }
}

/** Single row in the Log leaderboard. */
@Composable
private fun LogItemRow(host: PodHost, rank: Int, stat: GameLogStat) {
    val theme = LocalZTheme.current
    val hours = stat.secondsPlayed.toFloat() / SECONDS_IN_HOUR
    ZPanel(Modifier.fillMaxWidth(), RoundedCornerShape(14.dp), elevation = 2.dp) {
        Row(Modifier.fillMaxWidth().padding(14.dp), verticalAlignment = Alignment.CenterVertically) {
            Text(stringResource(R.string.pod_log_rank, rank), color = theme.accentStart, fontSize = 16.sp, fontWeight = FontWeight.Bold)
            Spacer(Modifier.width(16.dp))
            Column(Modifier.weight(1f)) {
                Text(stat.game.displayTitle, color = theme.textPrimary, fontSize = 17.sp, fontWeight = FontWeight.SemiBold)
                Text(stringResource(R.string.pod_log_hours, hours), color = theme.textSecondary, fontSize = 13.sp)
            }
            PadTile({ host.launch(stat.game) }, modifier = Modifier.size(40.dp)) {
                Icon(Icons.Rounded.PlayArrow, contentDescription = null, tint = theme.accentStart)
            }
        }
    }
}
