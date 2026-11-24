package dev.zlauncher.ui.settings

import android.content.Context
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.VolumeUp
import androidx.compose.material.icons.rounded.Animation
import androidx.compose.material.icons.rounded.AspectRatio
import androidx.compose.material.icons.rounded.BlurOn
import androidx.compose.material.icons.rounded.CropFree
import androidx.compose.material.icons.rounded.Dock
import androidx.compose.material.icons.rounded.Gradient
import androidx.compose.material.icons.rounded.GraphicEq
import androidx.compose.material.icons.rounded.GridOn
import androidx.compose.material.icons.rounded.Image
import androidx.compose.material.icons.rounded.Layers
import androidx.compose.material.icons.rounded.MusicNote
import androidx.compose.material.icons.rounded.Opacity
import androidx.compose.material.icons.rounded.PhotoSizeSelectLarge
import androidx.compose.material.icons.rounded.Speed
import androidx.compose.material.icons.rounded.ViewAgenda
import androidx.compose.material.icons.rounded.ViewCarousel
import androidx.compose.material.icons.rounded.ViewStream
import androidx.compose.material.icons.rounded.Wallpaper
import androidx.compose.material.icons.rounded.ZoomIn
import androidx.compose.ui.graphics.vector.ImageVector
import dev.zlauncher.R
import dev.zlauncher.core.nativecore.AnimationSpeed
import dev.zlauncher.core.nativecore.AssetCategory
import dev.zlauncher.core.nativecore.DockIconShape
import dev.zlauncher.core.nativecore.HeroDisplayStyle
import dev.zlauncher.core.nativecore.HeroGradientStyle
import dev.zlauncher.core.nativecore.MusicMode
import dev.zlauncher.core.nativecore.SingleRowStyle
import dev.zlauncher.core.nativecore.UserSettings
import dev.zlauncher.ui.theme.ZTheme

private const val PERCENT = 100
private const val STEP_05 = 0.05f
private const val BLUR_MAX = 10f
private const val BLUR_STEP = 0.5f
private const val ROWS_MIN = 1f
private const val ROWS_MAX = 5f
private const val ROWS_STEP = 1f
private const val ICON_SCALE_MIN = 0.5f
private const val ICON_SCALE_MAX = 1.0f
private const val HOVER_SCALE_MIN = 0.5f
private const val HOVER_SCALE_MAX = 1.5f
private const val ROUNDNESS_MAX = 0.5f
private const val LOGO_SCALE_MIN = 0.1f
private const val LOGO_SCALE_MAX = 1.0f

/** Formats a float fraction as percentage text. */
private fun pct(v: Float): String = "${Math.round(v * PERCENT)}%"

private data class SubMeta(
    val id: String,
    val icon: ImageVector,
    val titleRes: Int,
    val subRes: Int? = null,
)

private data class SliderSpec(
    val value: Float,
    val min: Float,
    val max: Float,
    val step: Float = STEP_05,
    val label: String? = null,
)

/**
 * Builds rows for Personalization subpages: glass, wallpapers, grid, icon style, hero, dock, sound,
 * jingles, animation, and now playing.
 */
class PersonalizationSubRows(
    private val model: SettingsModel,
    private val theme: ZTheme,
    private val context: Context,
) {
    private fun s(id: Int, vararg args: Any): String = context.getString(id, *args)

    private fun sliderRow(
        meta: SubMeta,
        spec: SliderSpec,
        onChange: (Float) -> Unit,
    ): SettingRow = SettingRow(
        id = meta.id,
        icon = meta.icon,
        title = s(meta.titleRes),
        subtitle = meta.subRes?.let { s(it) },
        control = Control.Slider(
            spec.value, spec.min, spec.max, spec.step,
            spec.label ?: pct(spec.value),
        ) { onChange(it) },
    )

    private fun toggleRow(
        meta: SubMeta,
        checked: Boolean,
        onToggle: (Boolean) -> Unit,
    ): SettingRow = SettingRow(
        id = meta.id,
        icon = meta.icon,
        title = s(meta.titleRes),
        subtitle = meta.subRes?.let { s(it) },
        control = Control.Switch(checked) { onToggle(it) },
    )

    private fun <T> choiceRow(
        meta: SubMeta,
        values: List<T>,
        labels: List<Int>,
        current: T,
        onSelect: (T) -> Unit,
    ): SettingRow {
        val selectedIdx = values.indexOf(current).coerceAtLeast(0)
        val dropdown = Control.Dropdown(labels.map { s(it) }, selectedIdx) { i ->
            onSelect(values[i])
        }
        return SettingRow(meta.id, meta.icon, s(meta.titleRes), meta.subRes?.let { s(it) }, dropdown)
    }

    /** Glass options rows: blur, refraction, highlight, rim strength, tint, glass on tiles. */
    fun glassOptions(st: UserSettings): List<SettingRow> = listOf(
        sliderRow(
            SubMeta("blur", Icons.Rounded.BlurOn, R.string.st_glass_blur),
            SliderSpec(st.glassBlur, 0f, BLUR_MAX, BLUR_STEP, "${st.glassBlur}"),
        ) { v -> model.change { it.setGlassBlur(v) } },
        sliderRow(
            SubMeta("refraction", Icons.Rounded.Opacity, R.string.st_glass_refraction),
            SliderSpec(st.glassRefraction, 0f, 1f),
        ) { v -> model.change { it.setGlassRefraction(v) } },
        sliderRow(
            SubMeta("highlight", Icons.Rounded.Gradient, R.string.st_glass_highlight),
            SliderSpec(st.glassHighlight, 0f, 1f),
        ) { v -> model.change { it.setGlassHighlight(v) } },
        sliderRow(
            SubMeta("rim", Icons.Rounded.CropFree, R.string.st_glass_rim_strength),
            SliderSpec(st.glassRimStrength, 0f, 1f),
        ) { v -> model.change { it.setGlassRimStrength(v) } },
        sliderRow(
            SubMeta("tint", Icons.Rounded.Opacity, R.string.st_glass_tint),
            SliderSpec(st.glassTint, 0f, 1f),
        ) { v -> model.change { it.setGlassTint(v) } },
        toggleRow(
            SubMeta("tiles", Icons.Rounded.GridOn, R.string.st_glass_on_tiles, R.string.st_glass_on_tiles_sub),
            st.glassOnTiles,
        ) { v -> model.change { it.setGlassOnTiles(v) } },
    )

    /** Wallpaper settings rows: main and external display wallpaper info. */
    fun wallpapers(): List<SettingRow> {
        val mainName = theme.wallpaperMain?.path?.substringAfterLast('/') ?: s(R.string.st_wallpaper_default)
        val extName = theme.wallpaperExternal?.path?.substringAfterLast('/') ?: s(R.string.st_wallpaper_default)
        return listOf(
            SettingRow(
                "wall_main", Icons.Rounded.Wallpaper, s(R.string.st_wallpaper_main),
                mainName, Control.Value(mainName),
            ),
            SettingRow(
                "wall_ext", Icons.Rounded.Image, s(R.string.st_wallpaper_external),
                extName, Control.Value(extName),
            ),
        )
    }

    /** Grid settings rows: discrete paging, rows per page, single row style. */
    fun grid(st: UserSettings): List<SettingRow> = listOf(
        toggleRow(
            SubMeta("discrete", Icons.Rounded.ViewCarousel, R.string.st_discrete_paging, R.string.st_discrete_paging_sub),
            st.discretePaging,
        ) { v -> model.change { it.setDiscretePaging(v) } },
        sliderRow(
            SubMeta("rows", Icons.Rounded.ViewStream, R.string.st_rows, R.string.st_rows_sub),
            SliderSpec(st.singleScreenRows.toFloat(), ROWS_MIN, ROWS_MAX, ROWS_STEP, "${st.singleScreenRows}"),
        ) { v -> model.change { it.setSingleScreenRows(v.toInt().toUByte()) } },
        choiceRow(
            SubMeta("rowstyle", Icons.Rounded.ViewAgenda, R.string.st_row_style, R.string.st_row_style_sub),
            SingleRowStyle.entries,
            listOf(R.string.st_row_standard, R.string.st_row_carousel, R.string.st_row_coverflow),
            st.singleRowStyle,
        ) { v -> model.change { it.setSingleRowStyle(v) } },
    )

    /** App icon style rows: size, hovered size, border radius. */
    fun appIconStyle(st: UserSettings): List<SettingRow> = listOf(
        sliderRow(
            SubMeta("icon_size", Icons.Rounded.PhotoSizeSelectLarge, R.string.st_icon_art_size),
            SliderSpec(st.iconScale, ICON_SCALE_MIN, ICON_SCALE_MAX),
        ) { v -> model.change { it.setIconScale(v) } },
        sliderRow(
            SubMeta("hover_size", Icons.Rounded.ZoomIn, R.string.st_hovered_icon_art_size),
            SliderSpec(st.hoverScale, HOVER_SCALE_MIN, HOVER_SCALE_MAX),
        ) { v -> model.change { it.setHoverScale(v) } },
        sliderRow(
            SubMeta("border_radius", Icons.Rounded.CropFree, R.string.st_border_radius),
            SliderSpec(st.iconRoundness, 0f, ROUNDNESS_MAX),
        ) { v -> model.change { it.setIconRoundness(v) } },
    )

    /** Hero settings rows: display style, gradient overlay, vignette size, logo scales. */
    fun heroSettings(st: UserSettings): List<SettingRow> {
        val styles = HeroDisplayStyle.entries
        val styleLabels = listOf(
            R.string.st_hero_style_vignette, R.string.st_hero_style_banner,
            R.string.st_hero_style_wallpaper, R.string.st_hero_style_none,
        )
        val grads = HeroGradientStyle.entries
        val gradLabels = listOf(R.string.st_hero_grad_none, R.string.st_hero_grad_black, R.string.st_hero_grad_accent)
        return listOf(
            choiceRow(
                SubMeta("hero_style", Icons.Rounded.Image, R.string.st_hero_bg_style),
                styles, styleLabels, st.heroDisplayStyle,
            ) { v -> model.change { it.setHeroDisplayStyle(v) } },
            choiceRow(
                SubMeta("hero_grad", Icons.Rounded.Gradient, R.string.st_hero_gradient),
                grads, gradLabels, st.heroGradientStyle,
            ) { v -> model.change { it.setHeroGradientStyle(v) } },
            sliderRow(
                SubMeta("vignette", Icons.Rounded.AspectRatio, R.string.st_hero_vignette_size),
                SliderSpec(st.heroVignetteSize, 0f, 1f),
            ) { v -> model.change { it.setHeroVignetteSize(v) } },
            sliderRow(
                SubMeta("game_logo", Icons.Rounded.PhotoSizeSelectLarge, R.string.st_hero_logo_size),
                SliderSpec(st.heroLogoScale, LOGO_SCALE_MIN, LOGO_SCALE_MAX),
            ) { v -> model.change { it.setHeroLogoScale(v) } },
            sliderRow(
                SubMeta("folder_logo", Icons.Rounded.PhotoSizeSelectLarge, R.string.st_hero_folder_logo_size),
                SliderSpec(st.heroFolderLogoScale, LOGO_SCALE_MIN, LOGO_SCALE_MAX),
            ) { v -> model.change { it.setHeroFolderLogoScale(v) } },
        )
    }

    /** Dock style rows: icon shape, dock on top, hide dock background. */
    fun dockStyle(st: UserSettings): List<SettingRow> {
        val shapes = DockIconShape.entries
        val shapeLabels = listOf(
            R.string.st_dock_shape_circle, R.string.st_dock_shape_rounded, R.string.st_dock_shape_square,
        )
        return listOf(
            choiceRow(
                SubMeta("dock_shape", Icons.Rounded.CropFree, R.string.st_dock_shape),
                shapes, shapeLabels, st.dockIconShape,
            ) { v -> model.change { it.setDockIconShape(v) } },
            toggleRow(
                SubMeta("dock_top", Icons.Rounded.Dock, R.string.st_dock_top, R.string.st_dock_top_sub),
                st.dockOnTop,
            ) { v -> model.change { it.setDockOnTop(v) } },
            toggleRow(
                SubMeta("dock_bg", Icons.Rounded.CropFree, R.string.st_dock_hide_bg, R.string.st_dock_hide_bg_sub),
                st.hideDockBackground,
            ) { v -> model.change { it.setHideDockBackground(v) } },
        )
    }

    /** Sound settings rows: SFX toggle, volume, and sound pack. */
    fun soundSettings(st: UserSettings): List<SettingRow> = listOf(
        toggleRow(
            SubMeta("sfx", Icons.AutoMirrored.Rounded.VolumeUp, R.string.st_sfx, R.string.st_sfx_sub),
            st.sfxEnabled,
        ) { v -> model.change { it.setSfxEnabled(v) } },
        sliderRow(
            SubMeta("sfx_vol", Icons.Rounded.GraphicEq, R.string.st_sfx_volume),
            SliderSpec(st.sfxVolume, 0f, 1f),
        ) { v -> model.change { it.setSfxVolume(v) } },
        SettingRow(
            "sound_pack", Icons.Rounded.Layers, s(R.string.st_pack_sounds),
            s(R.string.st_packs_sub), Control.Chevron, onActivate = {
                model.push(Page.Packs(AssetCategory.SOUNDS))
            },
        ),
    )

    /** Jingles and music settings rows: music mode, music volume, jingle volume. */
    fun jinglesSettings(st: UserSettings): List<SettingRow> {
        val modes = MusicMode.entries
        val modeLabels = listOf(R.string.st_music_disabled, R.string.st_music_playlist, R.string.st_music_time_based)
        return listOf(
            choiceRow(
                SubMeta("music_mode", Icons.Rounded.MusicNote, R.string.st_bg_music_mode),
                modes, modeLabels, st.musicMode,
            ) { v -> model.change { it.setMusicMode(v) } },
            sliderRow(
                SubMeta("music_vol", Icons.Rounded.GraphicEq, R.string.st_music_volume),
                SliderSpec(st.musicVolume, 0f, 1f),
            ) { v -> model.change { it.setMusicVolume(v) } },
            sliderRow(
                SubMeta("jingle_vol", Icons.AutoMirrored.Rounded.VolumeUp, R.string.st_jingle_volume),
                SliderSpec(st.jingleVolume, 0f, 1f),
            ) { v -> model.change { it.setJingleVolume(v) } },
        )
    }

    /** Animation settings rows: UI animation speed. */
    fun animation(st: UserSettings): List<SettingRow> {
        val speeds = AnimationSpeed.entries
        val speedLabels = listOf(R.string.st_anim_normal, R.string.st_anim_fast, R.string.st_anim_off)
        return listOf(
            choiceRow(
                SubMeta(
                    "anim_speed", Icons.Rounded.Speed, R.string.st_animation_speed,
                    R.string.st_animation_style_sub,
                ),
                speeds, speedLabels, st.animationSpeed,
            ) { v -> model.change { it.setAnimationSpeed(v) } },
        )
    }

    /** Now playing rows: idle screen darkening, idle color saturation. */
    fun nowPlaying(st: UserSettings): List<SettingRow> = listOf(
        sliderRow(
            SubMeta("np_dim", Icons.Rounded.Animation, R.string.st_now_playing_dimming),
            SliderSpec(st.nowPlayingDimming, 0f, 1f),
        ) { v -> model.change { it.setNowPlayingDimming(v) } },
        sliderRow(
            SubMeta("np_sat", Icons.Rounded.Opacity, R.string.st_now_playing_saturation),
            SliderSpec(st.nowPlayingSaturation, 0f, 1f),
        ) { v -> model.change { it.setNowPlayingSaturation(v) } },
    )
}
