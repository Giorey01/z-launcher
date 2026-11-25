package dev.zlauncher.ui.settings

import android.content.Context
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.VolumeUp
import androidx.compose.material.icons.rounded.Animation
import androidx.compose.material.icons.rounded.AspectRatio
import androidx.compose.material.icons.rounded.BatteryChargingFull
import androidx.compose.material.icons.rounded.BlurOn
import androidx.compose.material.icons.rounded.Brush
import androidx.compose.material.icons.rounded.Check
import androidx.compose.material.icons.rounded.CropFree
import androidx.compose.material.icons.rounded.Devices
import androidx.compose.material.icons.rounded.Dock
import androidx.compose.material.icons.rounded.Extension
import androidx.compose.material.icons.rounded.FormatColorFill
import androidx.compose.material.icons.rounded.Gamepad
import androidx.compose.material.icons.rounded.GridOn
import androidx.compose.material.icons.rounded.HighlightAlt
import androidx.compose.material.icons.rounded.Image
import androidx.compose.material.icons.rounded.Info
import androidx.compose.material.icons.rounded.Layers
import androidx.compose.material.icons.rounded.LineWeight
import androidx.compose.material.icons.rounded.Memory
import androidx.compose.material.icons.rounded.Monitor
import androidx.compose.material.icons.rounded.MusicNote
import androidx.compose.material.icons.rounded.Opacity
import androidx.compose.material.icons.rounded.Palette
import androidx.compose.material.icons.rounded.RestartAlt
import androidx.compose.material.icons.rounded.Speed
import androidx.compose.material.icons.rounded.SwapVert
import androidx.compose.material.icons.rounded.TextFields
import androidx.compose.material.icons.rounded.ViewAgenda
import androidx.compose.material.icons.rounded.ViewStream
import androidx.compose.material.icons.rounded.Wallpaper
import androidx.compose.material.icons.rounded.Smartphone
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.graphics.vector.ImageVector
import dev.zlauncher.R
import dev.zlauncher.core.nativecore.AssetCategory
import dev.zlauncher.nav.Screen
import dev.zlauncher.core.nativecore.CursorStyle
import dev.zlauncher.core.nativecore.ExternalDisplayDocking
import dev.zlauncher.core.nativecore.IconStyle
import dev.zlauncher.core.nativecore.LaunchScreen
import dev.zlauncher.core.nativecore.ScreenMode
import dev.zlauncher.core.nativecore.SingleRowStyle
import dev.zlauncher.core.nativecore.SurfaceMaterial
import dev.zlauncher.core.nativecore.UserSettings
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.UiMessage
import dev.zlauncher.ui.theme.ZTheme

/** Accent swatches matching Cocoon 1:1: 11 distinct colours. */
val AccentSwatches: List<Color> = listOf(
    Color(0xFFFF5376), Color(0xFFFF8D3B), Color(0xFFFFC107), Color(0xFF00C988), Color(0xFF54B4FF),
    Color(0xFF4B6BFF), Color(0xFF8358FF), Color(0xFFE056FD), Color(0xFFC2C7D0), Color(0xFF525D6D),
    Color(0xFF1E232B),
)

/** Percent as a text for sliders. */
private fun percent(value: Float): String = "${Math.round(value * PERCENT)}%"

private const val PERCENT = 100
private const val PERCENT_STEP = 0.05f
private const val MIN_HIGHLIGHT_THICKNESS = 0.5f
private const val MAX_HIGHLIGHT_THICKNESS = 1.0f
private const val MIN_HERO_SPLIT = 0.2f
private const val MAX_HERO_SPLIT = 0.7f
private const val MIN_SINGLE_SCREEN_ROWS = 1f
private const val MAX_SINGLE_SCREEN_ROWS = 5f
private const val ROW_COUNT_STEP = 1f

private data class RowMeta(val id: String, val icon: ImageVector, val title: Int, val subtitle: Int)

/** Builds settings rows for pages and categories matching Cocoon 1:1. */
class SettingsRows(
    private val app: AppViewModel,
    private val model: SettingsModel,
    private val theme: ZTheme,
    private val context: Context,
) {
    private fun s(id: Int, vararg args: Any): String = context.getString(id, *args)

    /** The rows of the current page. */
    fun build(): List<SettingRow> = allRows().filter { it.id !in NOT_WIRED }

    /** Every row of the current page, before the ones that do nothing yet are left out. */
    private fun allRows(): List<SettingRow> = subpageRows() ?: when (val page = model.page) {
        Page.Root -> root()
        Page.Themes -> ThemeRows(app, model, theme, context).themes()
        is Page.ApplyTheme -> ThemeRows(app, model, theme, context).apply(page)
        Page.PackLayers -> packLayers()
        is Page.Packs -> ThemeRows(app, model, theme, context).packs(page.category)
        Page.Roots -> LibraryRows(app, model, context).roots()
        Page.Emulators -> LibraryRows(app, model, context).emulators()
        else -> emptyList()
    }

    private fun subpageRows(): List<SettingRow>? {
        val sub = PersonalizationSubRows(model, theme, context)
        val st = theme.settings
        return when (model.page) {
            Page.GlassOptions -> sub.glassOptions(st)
            Page.Wallpapers -> sub.wallpapers()
            Page.Grid -> sub.grid(st)
            Page.AppIconStyle -> sub.appIconStyle(st)
            Page.HeroSettings -> sub.heroSettings(st)
            Page.DockStyle -> sub.dockStyle(st)
            Page.SoundSettings -> sub.soundSettings(st)
            Page.JinglesSettings -> sub.jinglesSettings(st)
            Page.Animation -> sub.animation(st)
            Page.NowPlaying -> sub.nowPlaying(st)
            else -> null
        }
    }

    /** The root list of the selected category. */
    private fun root(): List<SettingRow> = when (model.category) {
        Category.PERSONALIZATION -> personalization()
        Category.LIBRARY -> LibraryRows(app, model, context).root()
        Category.INTEGRATIONS -> IntegrationsRows(app, model, context).root()
        Category.SYSTEM -> system()
        Category.ABOUT -> about()
    }

    private fun <T> choice(
        meta: RowMeta,
        values: List<T>,
        labels: List<Int>,
        current: T,
        set: (dev.zlauncher.core.nativecore.Core, T) -> Unit,
    ): SettingRow {
        val selectedIndex = values.indexOf(current).coerceAtLeast(0)
        val dropdown = Control.Dropdown(labels.map { s(it) }, selectedIndex) { i ->
            model.change { set(it, values[i]) }
        }
        return SettingRow(meta.id, meta.icon, s(meta.title), s(meta.subtitle), dropdown)
    }

    private fun toggle(
        meta: RowMeta,
        on: Boolean,
        set: (dev.zlauncher.core.nativecore.Core, Boolean) -> Unit,
    ): SettingRow = SettingRow(
        meta.id, meta.icon, s(meta.title), s(meta.subtitle),
        Control.Switch(on) { v -> model.change { set(it, v) } },
    )

    private fun subpageRow(id: String, icon: ImageVector, titleRes: Int, subRes: Int, page: Page): SettingRow =
        SettingRow(id, icon, s(titleRes), s(subRes), Control.Chevron, onActivate = { model.push(page) })

    /** 20 rows of Personalization in the exact order of Cocoon. */
    private fun personalization(): List<SettingRow> {
        val st = theme.settings
        return personalizationFirstHalf(st) + personalizationSecondHalf(st)
    }

    /** Rows 1 to 10 of Personalization. */
    private fun personalizationFirstHalf(st: UserSettings): List<SettingRow> {
        val themeRow = SettingRow(
            "theme", Icons.Rounded.Brush, s(R.string.st_theme), theme.name, Control.Chevron,
            onActivate = { model.push(Page.Themes) },
        )
        val cursorMeta = RowMeta("cursor", Icons.Rounded.HighlightAlt, R.string.st_cursor, R.string.st_cursor_sub)
        val cursorLabels = listOf(
            R.string.st_cursor_animated, R.string.st_cursor_outlined,
            R.string.st_cursor_alt_animated, R.string.st_cursor_alt,
        )
        val surfaceMeta = RowMeta("surface", Icons.Rounded.BlurOn, R.string.st_surface, R.string.st_surface_sub)
        val iconStyleMeta = RowMeta("icons", Icons.Rounded.Palette, R.string.st_icon_style, R.string.st_icon_style_sub)
        val iconLabels = listOf(R.string.st_icon_accent, R.string.st_icon_colored, R.string.st_icon_flat)
        val thickSlider = Control.Slider(
            st.highlightThickness, MIN_HIGHLIGHT_THICKNESS, MAX_HIGHLIGHT_THICKNESS,
            PERCENT_STEP, percent(st.highlightThickness),
        ) { v -> model.change { it.setHighlightThickness(v) } }
        return listOf(
            themeRow,
            accent(),
            choice(cursorMeta, CursorStyle.entries, cursorLabels, st.cursorStyle) { c, v -> c.setCursorStyle(v) },
            SettingRow("thickness", Icons.Rounded.LineWeight, s(R.string.st_thickness), null, thickSlider),
            choice(
                surfaceMeta, SurfaceMaterial.entries,
                listOf(R.string.st_solid, R.string.st_glass), st.surfaceMaterial,
            ) { c, v -> c.setSurfaceMaterial(v) },
            subpageRow(
                "glass_opt", Icons.Rounded.Opacity, R.string.st_glass_options,
                R.string.st_glass_options_sub, Page.GlassOptions,
            ),
            choice(iconStyleMeta, IconStyle.entries, iconLabels, st.iconStyle) { c, v -> c.setIconStyle(v) },
            subpageRow(
                "wallpapers", Icons.Rounded.Wallpaper, R.string.st_wallpapers,
                R.string.st_wallpapers_sub, Page.Wallpapers,
            ),
            subpageRow("grid", Icons.Rounded.GridOn, R.string.st_grid, R.string.st_grid_sub, Page.Grid),
            subpageRow(
                "app_icon_style", Icons.Rounded.CropFree, R.string.st_app_icon_style,
                R.string.st_app_icon_style_sub, Page.AppIconStyle,
            ),
        )
    }

    /** Rows 11 to 20 of Personalization. */
    private fun personalizationSecondHalf(st: UserSettings): List<SettingRow> {
        val packControl = Control.Dropdown(listOf(s(R.string.st_app_icon_pack_default)), 0) {}
        val packRow = SettingRow(
            "app_icon_pack", Icons.Rounded.Palette, s(R.string.st_app_icon_pack),
            s(R.string.st_app_icon_pack_sub), packControl,
        )
        return listOf(
            packRow,
            subpageRow("hero", Icons.Rounded.Image, R.string.st_hero_settings, R.string.st_hero_settings_sub, Page.HeroSettings),
            subpageRow("dock", Icons.Rounded.Dock, R.string.st_dock_style, R.string.st_dock_style_sub, Page.DockStyle),
            subpageRow(
                "sound", Icons.AutoMirrored.Rounded.VolumeUp, R.string.st_sound_settings,
                R.string.st_sound_settings_sub, Page.SoundSettings,
            ),
            subpageRow(
                "jingles", Icons.Rounded.MusicNote, R.string.st_jingles_settings,
                R.string.st_jingles_settings_sub, Page.JinglesSettings,
            ),
            subpageRow(
                "anim", Icons.Rounded.Speed, R.string.st_animation_style,
                R.string.st_animation_style_sub, Page.Animation,
            ),
            subpageRow(
                "now_playing", Icons.Rounded.Animation, R.string.st_now_playing_style,
                R.string.st_now_playing_style_sub, Page.NowPlaying,
            ),
            toggle(
                RowMeta(
                    "fullscreen_folders", Icons.Rounded.AspectRatio,
                    R.string.st_fullscreen_folders, R.string.st_fullscreen_folders_sub,
                ),
                st.fullscreenFolders,
            ) { c, v -> c.setFullscreenFolders(v) },
            toggle(
                RowMeta(
                    "battery_pct", Icons.Rounded.BatteryChargingFull,
                    R.string.st_show_battery_percentage, R.string.st_show_battery_percentage_sub,
                ),
                st.showBatteryPercentage,
            ) { c, v -> c.setShowBatteryPercentage(v) },
            toggle(
                RowMeta("hints", Icons.Rounded.Gamepad, R.string.st_corner_hints, R.string.st_corner_hints_sub),
                st.cornerHints,
            ) { c, v -> c.setCornerHints(v) },
        )
    }

    /** The accent colour row with 11 swatches. */
    private fun accent(): SettingRow {
        val override = theme.settings.accentOverride
        val colors: List<Color?> = listOf<Color?>(null) + AccentSwatches
        val selected = override?.let { argb ->
            AccentSwatches.indexOfFirst { it.toArgbUInt() == argb }.takeIf { it >= 0 }?.plus(1)
        } ?: 0
        return SettingRow(
            "accent", Icons.Rounded.FormatColorFill, s(R.string.st_accent), null,
            Control.Swatches(colors, selected) { i ->
                model.change { it.setAccentOverride(if (i == 0) null else AccentSwatches[i - 1].toArgbUInt()) }
            },
        )
    }

    /** The layers that can have a pack of their own. */
    private fun packLayers(): List<SettingRow> = listOf(
        SettingRow(
            "overlays", Icons.Rounded.Layers, s(R.string.st_pack_overlays), null, Control.Chevron,
            onActivate = { model.push(Page.Packs(AssetCategory.ICON_OVERLAYS)) },
        ),
        SettingRow(
            "folders", Icons.Rounded.Layers, s(R.string.st_pack_folders), null, Control.Chevron,
            onActivate = { model.push(Page.Packs(AssetCategory.SMART_FOLDERS)) },
        ),
        SettingRow(
            "sounds", Icons.AutoMirrored.Rounded.VolumeUp, s(R.string.st_pack_sounds), null, Control.Chevron,
            onActivate = { model.push(Page.Packs(AssetCategory.SOUNDS)) },
        ),
    )

    /** System and advanced settings list. */
    private fun system(): List<SettingRow> {
        val st = theme.settings
        val screensMeta = RowMeta("screens", Icons.Rounded.Devices, R.string.st_screen_mode, R.string.st_screen_mode_sub)
        val reverseMeta = RowMeta("reverse", Icons.Rounded.SwapVert, R.string.st_reverse, R.string.st_reverse_sub)
        val launchMeta = RowMeta("launch_screen", Icons.Rounded.Smartphone, R.string.st_launch_screen, R.string.st_launch_screen_sub)
        val launchLabels = listOf(R.string.st_launch_hero, R.string.st_launch_grid)
        val dockingMeta = RowMeta("docking", Icons.Rounded.Monitor, R.string.st_docking, R.string.st_docking_sub)
        val dockingLabels = listOf(R.string.st_dock_auto, R.string.st_dock_always, R.string.st_dock_never)
        val rowStyleMeta = RowMeta("rowstyle", Icons.Rounded.ViewStream, R.string.st_row_style, R.string.st_row_style_sub)
        val rowStyleLabels = listOf(R.string.st_row_standard, R.string.st_row_carousel, R.string.st_row_coverflow)
        val splitSlider = Control.Slider(
            st.heroSplit, MIN_HERO_SPLIT, MAX_HERO_SPLIT, PERCENT_STEP, percent(st.heroSplit),
        ) { v -> model.change { it.setHeroSplit(v) } }
        val rowsSlider = Control.Slider(
            st.singleScreenRows.toFloat(), MIN_SINGLE_SCREEN_ROWS, MAX_SINGLE_SCREEN_ROWS,
            ROW_COUNT_STEP, st.singleScreenRows.toString(),
        ) { v -> model.change { it.setSingleScreenRows(v.toInt().toUByte()) } }
        return listOf(
            choice(screensMeta, ScreenMode.entries, listOf(R.string.st_dual, R.string.st_single), st.screenMode) { c, v ->
                c.setScreenMode(v)
            },
            toggle(reverseMeta, st.reverseDisplayRoles) { c, v -> c.setReverseDisplayRoles(v) },
            choice(launchMeta, LaunchScreen.entries, launchLabels, st.launchScreen) { c, v -> c.setLaunchScreen(v) },
            choice(dockingMeta, ExternalDisplayDocking.entries, dockingLabels, st.externalDisplayDocking) { c, v ->
                c.setExternalDisplayDocking(v)
            },
            SettingRow("split", Icons.Rounded.ViewAgenda, s(R.string.st_hero_split), s(R.string.st_hero_split_sub), splitSlider),
            SettingRow("rows", Icons.Rounded.ViewStream, s(R.string.st_rows), s(R.string.st_rows_sub), rowsSlider),
            choice(rowStyleMeta, SingleRowStyle.entries, rowStyleLabels, st.singleRowStyle) { c, v -> c.setSingleRowStyle(v) },
            SettingRow(
                "pods", Icons.Rounded.Extension, s(R.string.pod_screen_title), s(R.string.pod_screen_sub),
                Control.Chevron, onActivate = { app.open(Screen.PodManagement) },
            ),
            SettingRow("setup", Icons.Rounded.RestartAlt, s(R.string.st_setup), s(R.string.st_setup_sub), Control.Chevron, onActivate = { app.reopenSetup() }),
        )
    }

    /** The about list. */
    private fun about(): List<SettingRow> {
        val version = context.packageManager.getPackageInfo(context.packageName, 0).versionName.orEmpty()
        return listOf(
            SettingRow("version", Icons.Rounded.Info, s(R.string.app_name), s(R.string.st_about_version, version)),
            SettingRow("core", Icons.Rounded.Memory, s(R.string.st_about_core_title), dev.zlauncher.core.nativecore.coreVersion()),
            SettingRow("displays", Icons.Rounded.Devices, s(R.string.st_screen_details), s(R.string.st_screen_details_sub)),
            SettingRow("font", Icons.Rounded.TextFields, s(R.string.st_about_font_title), s(R.string.st_about_font)),
        )
    }

    /** A check mark text for the row of the item in use. */
    fun checkOr(active: Boolean): Control = if (active) Control.Value("✓") else Control.None

    /** Keeps the import of the check icon referenced (used by [ThemeRows]). */
    val checkIcon: ImageVector = Icons.Rounded.Check
}

/** The colour as the `0xAARRGGBB` integer the core takes. */
fun Color.toArgbUInt(): UInt = toArgb().toUInt()

/** Formats a [UiMessage] using the [Context], avoiding spread operators. */
fun formatUiMessage(context: Context, message: UiMessage): String {
    val args = message.args
    val res = message.res
    return if (args.isEmpty()) {
        context.getString(res)
    } else if (args.size == 1) {
        context.getString(res, args[0])
    } else if (args.size == 2) {
        context.getString(res, args[0], args[1])
    } else {
        context.getString(res, args[0], args[1], args[2])
    }
}

/**
 * Rows whose setting is stored but not used anywhere yet, so changing it would do nothing: they stay hidden
 * until the feature behind them exists. Remove an id from here in the same change that makes it work.
 */
private val NOT_WIRED = setOf(
    "app_icon_pack", // Android icon packs are not loaded yet
    "jingles", // no background music or jingle player yet
    "anim", // the animation speed is not applied to any animation
    "fullscreen_folders", // folders always open the same way
    "refraction", "highlight", "tiles", // glass options the surfaces do not draw
    "discrete", // the grid always scrolls
    "icon_size", // the art size is fixed in the tiles
    "dock_bg", // the dock has no background to hide
)
