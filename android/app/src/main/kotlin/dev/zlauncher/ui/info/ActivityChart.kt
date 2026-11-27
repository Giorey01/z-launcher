package dev.zlauncher.ui.info

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.zlauncher.R
import dev.zlauncher.core.nativecore.WeekBucket
import dev.zlauncher.ui.theme.LocalZTheme

/** Height of the bars' area. */
private val CHART_HEIGHT = 96.dp

/** Share of a bar's slot that is empty on each side: the gap between two bars. */
private const val GAP_SHARE = 0.18f

/** Corner radius of a bar. */
private val BAR_RADIUS = 4.dp

/**
 * The weekly chart (RF-31): one bar per week, oldest at the left and the current week at the right, in the theme's accent
 * colours; the current week is the brightest. Under it the date of the first week and "This week", over it the busiest week.
 */
@Composable
fun WeeklyChart(weeks: List<WeekBucket>, modifier: Modifier = Modifier) {
    val theme = LocalZTheme.current
    val context = LocalContext.current
    val fractions = barFractions(weeks.map { it.seconds })
    val peak = weeks.maxOfOrNull { it.seconds } ?: 0L
    Column(modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(4.dp)) {
        Text(stringResource(R.string.act_chart_title, weeks.size), color = theme.textPrimary, fontSize = 14.sp)
        if (peak > 0) {
            Text(stringResource(R.string.act_chart_peak, formatDuration(context, peak)), color = theme.textSecondary, fontSize = 12.sp)
        }
        val brush = theme.accentBrush()
        val faint = theme.accentStart.copy(alpha = 0.55f)
        val baseline = theme.divider
        Canvas(Modifier.fillMaxWidth().height(CHART_HEIGHT)) {
            val slot = size.width / fractions.size.coerceAtLeast(1)
            val bar = slot * (1f - 2 * GAP_SHARE)
            fractions.forEachIndexed { index, share ->
                val height = size.height * share
                val left = slot * index + slot * GAP_SHARE
                val corner = CornerRadius(BAR_RADIUS.toPx())
                if (height > 0f) {
                    val topLeft = Offset(left, size.height - height)
                    if (index == fractions.lastIndex) {
                        drawRoundRect(brush, topLeft, Size(bar, height), corner)
                    } else {
                        drawRoundRect(faint, topLeft, Size(bar, height), corner)
                    }
                }
            }
            drawLine(baseline, Offset(0f, size.height), Offset(size.width, size.height), strokeWidth = 1.dp.toPx())
        }
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            val first = weeks.firstOrNull()
            Text(first?.let { formatDay(context, it.weekStart) } ?: "", color = theme.textSecondary, fontSize = 12.sp)
            Text(stringResource(R.string.act_this_week), color = theme.textSecondary, fontSize = 12.sp)
        }
    }
}
