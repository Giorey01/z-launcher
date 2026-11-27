package dev.zlauncher.ui.info

import android.content.Context
import android.text.format.DateFormat
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.produceState
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.zlauncher.R
import dev.zlauncher.core.nativecore.CoreException
import dev.zlauncher.core.nativecore.GameActivity
import dev.zlauncher.core.nativecore.GameEntry
import dev.zlauncher.core.nativecore.SessionRow
import dev.zlauncher.core.nativecore.SessionSource
import dev.zlauncher.launch.toMessage
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.ui.pad.FocusMemory
import dev.zlauncher.ui.pad.PadButton
import dev.zlauncher.ui.pad.PadTile
import dev.zlauncher.ui.theme.LocalZTheme
import java.util.Date
import java.util.TimeZone
import kotlinx.coroutines.launch

/** Focus key of the button that adds a session; the Activity tab's first stop. */
internal const val ACTIVITY_FIRST_KEY = "act:add"

/** Share of the tab's width the numbers and the chart take; the list of sessions takes the rest. */
private const val STATS_SHARE = 0.46f

/** Milliseconds in a second. */
private const val MS = 1000L

/** The Activity tab of a game (RF-31): the totals, the weekly chart and the sessions, which can be corrected (RF-41). */
@Composable
fun ActivityTab(app: AppViewModel, game: GameEntry, memory: FocusMemory, modifier: Modifier = Modifier) {
    val tabs = app.infoTabs
    val activity by produceState<GameActivity?>(null, game.id, app.resumeTick, tabs.sessionsVersion) {
        value = try {
            val now = System.currentTimeMillis()
            app.zl.core.call { it.gameActivity(game.id, now / MS, TimeZone.getDefault().getOffset(now) / MS) }
        } catch (e: CoreException) {
            app.message = e.toMessage()
            null
        }
    }
    val data = activity ?: return
    Row(modifier.fillMaxSize(), horizontalArrangement = Arrangement.spacedBy(20.dp)) {
        Column(Modifier.fillMaxWidth(STATS_SHARE).fillMaxHeight(), verticalArrangement = Arrangement.spacedBy(14.dp)) {
            ActivityStats(data)
            WeeklyChart(data.summary.weeks)
            Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                PadButton(stringResource(R.string.act_add), { tabs.dialog = ActivityDialog.Add }, memory.track(ACTIVITY_FIRST_KEY))
            }
            TrackingToggle(app, game, memory)
        }
        SessionList(app, data.sessions, memory, Modifier.weight(1f).fillMaxHeight())
    }
}

/** The four numbers (total, sessions, average, last) and, when there is one, the time Steam reports on top. */
@Composable
private fun ActivityStats(data: GameActivity) {
    val context = LocalContext.current
    val summary = data.summary
    val last = if (summary.lastPlayed > 0) formatDate(context, summary.lastPlayed) else "—"
    Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            Stat(stringResource(R.string.act_total), formatDuration(context, summary.totalSeconds), Modifier.weight(1f))
            Stat(stringResource(R.string.act_sessions), summary.sessionCount.toString(), Modifier.weight(1f))
        }
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            Stat(stringResource(R.string.act_average), formatDuration(context, summary.averageSeconds), Modifier.weight(1f))
            Stat(stringResource(R.string.act_last), last, Modifier.weight(1f))
        }
        if (data.steamSeconds > 0) {
            Text(
                stringResource(R.string.act_steam_time, formatDuration(context, data.steamSeconds)),
                color = LocalZTheme.current.textSecondary,
                fontSize = 14.sp,
            )
        }
    }
}

/** One number with its name over it. */
@Composable
private fun Stat(label: String, value: String, modifier: Modifier) {
    val theme = LocalZTheme.current
    Column(modifier) {
        Text(label, color = theme.textSecondary, fontSize = 13.sp, maxLines = 1)
        Text(value, color = theme.textPrimary, fontSize = 20.sp, fontWeight = FontWeight.Bold, maxLines = 1)
    }
}

/** The list of the sessions, newest first; A corrects the one under the cursor and Y deletes it. */
@Composable
private fun SessionList(app: AppViewModel, sessions: List<SessionRow>, memory: FocusMemory, modifier: Modifier) {
    val theme = LocalZTheme.current
    Column(modifier, verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
            Text(
                stringResource(R.string.act_sessions_title),
                Modifier.weight(1f),
                color = theme.textPrimary,
                fontSize = 17.sp,
                fontWeight = FontWeight.SemiBold,
            )
            if (sessions.isNotEmpty()) Text(stringResource(R.string.act_row_hint), color = theme.textSecondary, fontSize = 12.sp)
        }
        if (sessions.isEmpty()) {
            Text(stringResource(R.string.act_no_sessions), color = theme.textSecondary, fontSize = 15.sp)
        } else {
            LazyColumn(Modifier.fillMaxSize(), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                items(sessions, key = { it.id }) { session -> SessionRowTile(app, session, memory) }
            }
        }
    }
}

/** A session: its date and time, how it was counted, and its length. */
@Composable
private fun SessionRowTile(app: AppViewModel, session: SessionRow, memory: FocusMemory) {
    val theme = LocalZTheme.current
    val context = LocalContext.current
    PadTile(
        onClick = { app.infoTabs.dialog = ActivityDialog.Edit(session) },
        modifier = Modifier.fillMaxWidth().then(memory.track("act:s:${session.id}")),
        onSecondary = { app.infoTabs.dialog = ActivityDialog.Delete(session) },
    ) {
        Row(Modifier.fillMaxWidth().padding(horizontal = 14.dp, vertical = 9.dp), verticalAlignment = Alignment.CenterVertically) {
            Column(Modifier.weight(1f)) {
                Text(formatDate(context, session.startedAt), color = theme.textPrimary, fontSize = 15.sp, maxLines = 1)
                Text(sourceLabel(session.source), color = theme.textSecondary, fontSize = 12.sp, maxLines = 1)
            }
            Text(formatDuration(context, session.seconds), color = theme.accentStart, fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
        }
    }
}

/** How a session was counted, in words. */
@Composable
private fun sourceLabel(source: SessionSource): String = stringResource(
    when (source) {
        SessionSource.USAGE -> R.string.act_source_usage
        SessionSource.MANUAL -> R.string.act_source_manual
        else -> R.string.act_source_estimated
    },
)

/** The app a game runs in and whether its time is left out of the tracking. */
private data class Tracked(
    /** The package the game was last started through, `null` before its first launch. */
    val pkg: String?,
    /** Whether that app is left out. */
    val excluded: Boolean,
)

/**
 * The button that leaves the app running this game out of the tracking, or puts it back (RF-41). The app is the
 * package the game was last started through; before the first launch there is none and a hint says so.
 */
@Composable
private fun TrackingToggle(app: AppViewModel, game: GameEntry, memory: FocusMemory) {
    val tabs = app.infoTabs
    val scope = rememberCoroutineScope()
    val tracked by produceState<Tracked?>(null, game.id, app.resumeTick, tabs.sessionsVersion) {
        value = try {
            app.zl.core.call { core ->
                val pkg = game.appPackage ?: core.trackingPackage(game.id)
                Tracked(pkg, pkg != null && core.isAppTrackingExcluded(pkg))
            }
        } catch (e: CoreException) {
            app.message = e.toMessage()
            null
        }
    }
    val state = tracked ?: return
    val pkg = state.pkg
    if (pkg == null) {
        Text(stringResource(R.string.act_track_unknown), color = LocalZTheme.current.textSecondary, fontSize = 13.sp)
        return
    }
    val label = stringResource(if (state.excluded) R.string.act_track_include else R.string.act_track_exclude, pkg)
    PadButton(label, {
        scope.launch {
            try {
                app.zl.core.call { it.setAppTrackingExcluded(pkg, !state.excluded) }
                tabs.sessionsChanged()
            } catch (e: CoreException) {
                app.message = e.toMessage()
            }
        }
    }, memory.track("act:track"))
}

/** A date in the user's format ("27 Nov 2026"). */
internal fun formatDay(context: Context, unixSeconds: Long): String = DateFormat.getMediumDateFormat(context).format(Date(unixSeconds * MS))

/** A date and time in the user's formats ("27 Nov 2026, 21:14"). */
internal fun formatDate(context: Context, unixSeconds: Long): String =
    formatDay(context, unixSeconds) + ", " + DateFormat.getTimeFormat(context).format(Date(unixSeconds * MS))
