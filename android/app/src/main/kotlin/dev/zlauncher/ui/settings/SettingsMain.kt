package dev.zlauncher.ui.settings

import android.content.Intent
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.KeyboardArrowDown
import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import dev.zlauncher.R
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.ui.common.BackPill
import dev.zlauncher.ui.common.HeaderTitle
import dev.zlauncher.ui.pad.FocusCursor
import androidx.compose.ui.unit.IntOffset
import dev.zlauncher.ui.theme.LocalBackdrop
import dev.zlauncher.ui.theme.LocalZTheme
import dev.zlauncher.ui.theme.ZPanel

/** Width of the category rail. */
private val RAIL_WIDTH = 243.dp

/** Where the rail starts, from the top. */
private val RAIL_TOP = 76.dp

/** Distance of the list panel from the top. */
private val LIST_TOP = 64.dp

/** Height of the glass band behind the header. */
private val HEADER_BAND = 50.dp

@Composable
private fun personalizationSubpageTitle(page: Page): String? = when (page) {
    Page.GlassOptions -> stringResource(R.string.st_glass_options)
    Page.Wallpapers -> stringResource(R.string.st_wallpapers)
    Page.Grid -> stringResource(R.string.st_grid)
    Page.AppIconStyle -> stringResource(R.string.st_app_icon_style)
    Page.HeroSettings -> stringResource(R.string.st_hero_settings)
    Page.DockStyle -> stringResource(R.string.st_dock_style)
    Page.SoundSettings -> stringResource(R.string.st_sound_settings)
    Page.JinglesSettings -> stringResource(R.string.st_jingles_settings)
    Page.Animation -> stringResource(R.string.st_animation_style)
    Page.NowPlaying -> stringResource(R.string.st_now_playing_style)
    else -> null
}

/** Title of a page: the category name on the root, the page's own name on a subpage. */
@Composable
private fun pageTitle(model: SettingsModel): String =
    personalizationSubpageTitle(model.page) ?: when (val page = model.page) {
        Page.Root -> stringResource(R.string.st_title)
        Page.Themes -> stringResource(R.string.st_theme)
        is Page.ApplyTheme -> page.name
        Page.PackLayers -> stringResource(R.string.st_packs)
        is Page.Packs -> stringResource(packTitleRes(page.category))
        Page.Roots -> stringResource(R.string.st_roots)
        Page.Emulators -> stringResource(R.string.st_emulators)
        else -> ""
    }

private fun packTitleRes(category: dev.zlauncher.core.nativecore.AssetCategory): Int = when (category) {
    dev.zlauncher.core.nativecore.AssetCategory.ICON_OVERLAYS -> R.string.st_pack_overlays
    dev.zlauncher.core.nativecore.AssetCategory.SMART_FOLDERS -> R.string.st_pack_folders
    else -> R.string.st_pack_sounds
}

/**
 * The main part of the settings: the header, the category rail on the left and the list of rows on the right.
 * Shown on the big display in the dual-screen mode (the grid display shows [SettingsAside]) and full screen in
 * the single-screen mode. The pickers for a theme ZIP and a ROM folder are opened from here when the model asks.
 */
@Composable
fun SettingsMain(app: AppViewModel) {
    val theme = LocalZTheme.current
    val model = app.settings
    val context = LocalContext.current
    val rows = remember(theme.revision, model.data, model.page, model.category, model.result, model.busy) { SettingsRows(app, model, theme, context).build() }
    PickerEffects(model)
    CompositionLocalProvider(LocalBackdrop provides null) {
        Box(Modifier.fillMaxSize().background(theme.backgroundBrush())) {
            val bandBrush = Brush.verticalGradient(
                listOf(theme.textPrimary.copy(alpha = 0.16f), Color.Transparent)
            )
            Box(Modifier.fillMaxWidth().height(HEADER_BAND).background(bandBrush))
            Row(Modifier.padding(start = 8.dp, top = 8.dp), verticalAlignment = Alignment.CenterVertically) {
                BackPill()
                HeaderTitle(pageTitle(model), Modifier.padding(start = 12.dp))
            }
            Rail(model, Modifier.padding(start = 16.dp, top = RAIL_TOP).width(RAIL_WIDTH))
            SettingsList(model, rows, Modifier.padding(start = 16.dp + RAIL_WIDTH + 26.dp, top = LIST_TOP, end = 30.dp).fillMaxHeight().fillMaxWidth())
        }
    }
}

/** Opens the system pickers when the model asks for them and hands the result back. */
@Composable
private fun PickerEffects(model: SettingsModel) {
    val context = LocalContext.current
    val zip = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri -> uri?.let(model::importZip) }
    val folder = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocumentTree()) { uri ->
        if (uri != null) {
            context.contentResolver.takePersistableUriPermission(uri, Intent.FLAG_GRANT_READ_URI_PERMISSION)
            model.addTree(uri.toString())
        }
    }
    LaunchedEffect(model.wantZip) {
        if (model.wantZip) {
            model.wantZip = false
            zip.launch(arrayOf("application/zip", "application/x-zip-compressed", "application/octet-stream"))
        }
    }
    LaunchedEffect(model.wantFolder) {
        if (model.wantFolder) {
            model.wantFolder = false
            folder.launch(null)
        }
    }
}

/** The scrolling list of rows in one glass panel, with a down arrow when there is more below. */
@Composable
private fun SettingsList(model: SettingsModel, rows: List<SettingRow>, modifier: Modifier) {
    val state = rememberLazyListState()
    LaunchedEffect(model.row, model.page, model.category) {
        val offset = -(state.layoutInfo.viewportSize.height / 3)
        state.animateScrollToItem(model.row.coerceIn(0, (rows.size - 1).coerceAtLeast(0)), offset)
    }
    Box(modifier) {
        ZPanel(Modifier.fillMaxSize(), RoundedCornerShape(topStart = 18.dp, topEnd = 18.dp), elevation = 6.dp) {
            LazyColumn(Modifier.fillMaxSize(), state) {
                itemsIndexed(rows, key = { _, r -> r.id }) { index, row ->
                    SettingRowItem(row, dropdownIndex = if (index == model.row) model.dropdown else -1)
                }
            }
            if (state.canScrollForward) {
                Icon(
                    Icons.Rounded.KeyboardArrowDown,
                    null,
                    Modifier.align(Alignment.BottomCenter).padding(bottom = 4.dp),
                    tint = LocalZTheme.current.accentStart
                )
            }
        }
        if (!model.inRail) RowCursor(state, model.row)
    }
}

/** The cursor brackets over the focused row, drawn outside the panel's clip so they may overhang its edge. */
@Composable
private fun RowCursor(state: androidx.compose.foundation.lazy.LazyListState, index: Int) {
    val info = state.layoutInfo.visibleItemsInfo.firstOrNull { it.index == index } ?: return
    val density = androidx.compose.ui.platform.LocalDensity.current
    val dividerPx = with(density) { 1.5.dp.toPx() }
    Box(Modifier.offset { IntOffset(0, info.offset) }.fillMaxWidth().height(with(density) { (info.size - dividerPx).toDp() })) {
        FocusCursor(Modifier.matchParentSize(), 5.dp, 26.dp)
    }
}

/** The category rail: the selected category is a card with its subtitle, the others plain rows. */
@Composable
private fun Rail(model: SettingsModel, modifier: Modifier) {
    Column(modifier) {
        SearchPill(Modifier.padding(bottom = 14.dp))
        VisibleCategories.forEach { category ->
            RailItem(category, selected = category == model.category, cursor = model.inRail && category == model.category)
        }
    }
}
