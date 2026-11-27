package dev.zlauncher.ui.info

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.zlauncher.R
import dev.zlauncher.core.nativecore.CoreException
import dev.zlauncher.core.nativecore.GameEntry
import dev.zlauncher.launch.toMessage
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.ui.pad.PadButton
import dev.zlauncher.ui.pods.DialogButtons
import dev.zlauncher.ui.pods.PodDialog
import dev.zlauncher.ui.theme.LocalZTheme
import java.util.TimeZone
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch

/** Seconds in a minute. */
private const val MINUTE = 60L

/** Seconds in an hour. */
private const val HOUR = 3_600L

/** Seconds in a day. */
private const val DAY = 86_400L

/** Milliseconds in a second. */
private const val MS = 1000L

/** Minutes of the length a session added by hand starts with (an hour). */
private const val DEFAULT_ADD_SECONDS = HOUR

/** The step buttons of a duration: the label and the change in seconds. */
private val DURATION_STEPS = listOf(
    R.string.act_minus_hour to -HOUR,
    R.string.act_minus_10m to -10 * MINUTE,
    R.string.act_minus_minute to -MINUTE,
    R.string.act_plus_minute to MINUTE,
    R.string.act_plus_10m to 10 * MINUTE,
    R.string.act_plus_hour to HOUR,
)

/** The dialog of the Activity tab that is open, if any (RF-41). The state is [InfoTabs.dialog], so B closes it. */
@Composable
fun ActivityDialogs(app: AppViewModel, game: GameEntry) {
    when (val dialog = app.infoTabs.dialog) {
        is ActivityDialog.Edit -> EditSessionDialog(app, dialog)
        is ActivityDialog.Delete -> DeleteSessionDialog(app, dialog)
        ActivityDialog.Add -> AddSessionDialog(app, game)
        null -> Unit
    }
}

/** Runs a change of the sessions in the core; when it works the dialog closes and the screens read the sessions again. */
private fun CoroutineScope.saveSessions(app: AppViewModel, change: suspend () -> Unit) {
    launch {
        try {
            change()
            app.infoTabs.dialog = null
            app.infoTabs.sessionsChanged()
            app.refresh()
        } catch (e: CoreException) {
            app.message = e.toMessage()
        }
    }
}

/** A row of buttons that move [seconds] by the steps. */
@Composable
private fun DurationSteps(seconds: Long, onChange: (Long) -> Unit) {
    val context = LocalContext.current
    Text(
        stringResource(R.string.act_duration_value, formatDuration(context, seconds)),
        color = LocalZTheme.current.textPrimary,
        fontSize = 18.sp,
    )
    Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
        for ((label, delta) in DURATION_STEPS) PadButton(stringResource(label), { onChange(adjustDuration(seconds, delta)) })
    }
}

/** Corrects the duration of a session; its end moves with it. */
@Composable
private fun EditSessionDialog(app: AppViewModel, dialog: ActivityDialog.Edit) {
    val scope = rememberCoroutineScope()
    var seconds by remember(dialog.session.id) { mutableLongStateOf(dialog.session.seconds.coerceIn(MIN_MANUAL_SECONDS, MAX_MANUAL_SECONDS)) }
    PodDialog(stringResource(R.string.act_edit_title), { app.infoTabs.dialog = null }, maxWidth = 620.dp) {
        Text(formatDate(LocalContext.current, dialog.session.startedAt), color = LocalZTheme.current.textSecondary, fontSize = 14.sp)
        DurationSteps(seconds) { seconds = it }
        DialogButtons(
            stringResource(R.string.act_save),
            { scope.saveSessions(app) { app.zl.core.call { it.setSessionSeconds(dialog.session.id, seconds) } } },
            { app.infoTabs.dialog = null },
        )
    }
}

/** Asks before a session is deleted. */
@Composable
private fun DeleteSessionDialog(app: AppViewModel, dialog: ActivityDialog.Delete) {
    val scope = rememberCoroutineScope()
    val context = LocalContext.current
    val text = stringResource(
        R.string.act_delete_confirm,
        formatDuration(context, dialog.session.seconds),
        formatDate(context, dialog.session.startedAt),
    )
    PodDialog(stringResource(R.string.act_delete_title), { app.infoTabs.dialog = null }) {
        Text(text, color = LocalZTheme.current.textSecondary, fontSize = 15.sp)
        DialogButtons(
            stringResource(R.string.act_delete),
            { scope.saveSessions(app) { app.zl.core.call { it.deleteSession(dialog.session.id) } } },
            { app.infoTabs.dialog = null },
        )
    }
}

/** Adds a session by hand: its length and the day it was played (today, or up to a year back). */
@Composable
private fun AddSessionDialog(app: AppViewModel, game: GameEntry) {
    val scope = rememberCoroutineScope()
    var seconds by remember { mutableLongStateOf(DEFAULT_ADD_SECONDS) }
    var daysAgo by remember { mutableIntStateOf(0) }
    PodDialog(stringResource(R.string.act_add_title), { app.infoTabs.dialog = null }, maxWidth = 620.dp) {
        DurationSteps(seconds) { seconds = it }
        DayLabel(daysAgo)
        Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            DayStep(R.string.act_earlier_week, daysAgo, 7) { daysAgo = it }
            DayStep(R.string.act_earlier_day, daysAgo, 1) { daysAgo = it }
            DayStep(R.string.act_later_day, daysAgo, -1) { daysAgo = it }
            DayStep(R.string.act_later_week, daysAgo, -7) { daysAgo = it }
        }
        DialogButtons(
            stringResource(R.string.act_save),
            { scope.saveSessions(app) { addSession(app, game, seconds, daysAgo) } },
            { app.infoTabs.dialog = null },
        )
    }
}

/** Stores the session of [seconds] played [daysAgo] days ago. */
private suspend fun addSession(app: AppViewModel, game: GameEntry, seconds: Long, daysAgo: Int) {
    val now = System.currentTimeMillis()
    val offset = TimeZone.getDefault().getOffset(now) / MS
    val start = manualSessionStart(now / MS, offset, daysAgo, seconds)
    app.zl.core.call { it.addManualSession(game.id, start, seconds) }
}

/** A button that moves the day by [moreDaysAgo] days (positive goes back in time), kept between today and a year ago. */
@Composable
private fun DayStep(label: Int, daysAgo: Int, moreDaysAgo: Int, onChange: (Int) -> Unit) {
    val next = (daysAgo + moreDaysAgo).coerceIn(0, MAX_DAYS_AGO)
    PadButton(stringResource(label), { onChange(next) })
}

/** "Day: today", "Day: yesterday" or "Day: 12 Nov 2026". */
@Composable
private fun DayLabel(daysAgo: Int) {
    val context = LocalContext.current
    val day = when (daysAgo) {
        0 -> stringResource(R.string.act_day_today)
        1 -> stringResource(R.string.act_day_yesterday)
        else -> formatDay(context, System.currentTimeMillis() / MS - daysAgo * DAY)
    }
    Text(stringResource(R.string.act_day_value, day), color = LocalZTheme.current.textPrimary, fontSize = 18.sp)
}
