package dev.zlauncher.ui.home

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.unit.Dp
import dev.zlauncher.core.nativecore.SingleRowStyle
import kotlin.math.abs

/** How far (in cell widths) from the middle a CoverFlow item is turned away at most. */
private const val FLOW_RANGE = 2f

/** Rotation of the farthest CoverFlow items, in degrees. */
private const val FLOW_ANGLE = 38f

/** Smallest scale of the farthest CoverFlow and carousel items. */
private const val FLOW_MIN_SCALE = 0.72f

/** The shape of the library grid: how many columns, and how it is laid out when it is a single row. */
data class GridSpec(
    /** Columns of the platform grid. */
    val platformColumns: Int = 3,
    /** Columns of the game grid. */
    val gameColumns: Int = 5,
    /** Rows shown; `1` makes a horizontal row drawn in [style]. */
    val rows: Int = 0,
    /** How the single row looks. */
    val style: SingleRowStyle = SingleRowStyle.STANDARD,
    /** Height of the single row (and so the size of its cells). */
    val rowHeight: Dp = Dp.Unspecified,
)

/** The grid shape of the screen; the dual-screen grid display uses the defaults. */
val LocalGridSpec = compositionLocalOf { GridSpec() }

/**
 * A single horizontal row of square cells of the spec's height: Standard is a plain row, Carousel scales the cells
 * down away from the middle, CoverFlow also turns them in perspective.
 */
@OptIn(ExperimentalFoundationApi::class)
@Composable
fun <T> OneRow(items: List<T>, spec: GridSpec, modifier: Modifier = Modifier, cell: @Composable (T, Modifier) -> Unit) {
    val state = rememberLazyListState()
    LazyRow(modifier.fillMaxHeight(), state) {
        itemsIndexed(items) { index, item ->
            val distance = remember(state, index) {
                derivedStateOf {
                    val info = state.layoutInfo.visibleItemsInfo.firstOrNull { it.index == index }
                    distanceFromMiddle(state.layoutInfo.viewportStartOffset, state.layoutInfo.viewportEndOffset, info)
                }
            }
            val shaped = if (spec.style == SingleRowStyle.STANDARD) Modifier else Modifier.graphicsLayer {
                val d = distance.value.coerceIn(-FLOW_RANGE, FLOW_RANGE)
                val scale = 1f - (1f - FLOW_MIN_SCALE) * abs(d) / FLOW_RANGE
                scaleX = scale
                scaleY = scale
                if (spec.style == SingleRowStyle.COVER_FLOW) {
                    rotationY = -d / FLOW_RANGE * FLOW_ANGLE
                    cameraDistance = 12f * density
                }
            }
            Box(Modifier.width(spec.rowHeight).fillMaxHeight().then(shaped)) { cell(item, Modifier) }
        }
    }
}

/** Distance of an item's centre from the viewport middle, in item widths (0 when it is not visible). */
private fun distanceFromMiddle(start: Int, end: Int, info: androidx.compose.foundation.lazy.LazyListItemInfo?): Float {
    if (info == null || info.size == 0) return 0f
    val itemCentre = info.offset + info.size / 2f
    return (itemCentre - (start + end) / 2f) / info.size
}
