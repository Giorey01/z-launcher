package dev.zlauncher.ui.settings

import android.content.Context
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Brush
import androidx.compose.material.icons.rounded.Download
import androidx.compose.material.icons.rounded.FolderZip
import androidx.compose.material.icons.rounded.Layers
import androidx.compose.material.icons.rounded.Palette
import dev.zlauncher.R
import dev.zlauncher.core.nativecore.AssetCategory
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.UiMessage
import dev.zlauncher.ui.theme.ZTheme

/** Rows of the theme pages: the list of themes with the import actions, the apply choice and the layer packs. */
class ThemeRows(private val app: AppViewModel, private val model: SettingsModel, private val theme: ZTheme, private val context: Context) {
    /** A string resource. */
    private fun s(id: Int, vararg args: Any): String = context.getString(id, *args)

    /** The installed themes, the built-in one and the import actions. */
    fun themes(): List<SettingRow> {
        val data = model.data
        val rows = mutableListOf<SettingRow>()
        val builtinActive = data.themes.none { it.active }
        rows += SettingRow(
            "builtin",
            Icons.Rounded.Brush,
            s(R.string.st_theme_builtin),
            null,
            active(builtinActive),
            onActivate = { model.push(Page.ApplyTheme(null, s(R.string.st_theme_builtin))) }
        )
        data.themes.forEach { t ->
            val details = listOfNotNull(t.author, t.version?.let { "v$it" }).joinToString(" · ").ifEmpty { null }
            rows += SettingRow(
                "t:${t.id}",
                Icons.Rounded.Palette,
                t.name,
                details,
                active(t.active),
                onActivate = { model.push(Page.ApplyTheme(t.id, t.name)) },
                preview = t.preview
            )
        }
        data.cocoon.forEach { dir ->
            val sub = s(R.string.st_theme_import_cocoon_sub, dir.path, dir.themes.toInt(), dir.packs.toInt())
            rows += SettingRow(
                "cocoon:${dir.path}",
                Icons.Rounded.Download,
                s(R.string.st_theme_import_cocoon),
                resultOr("cocoon:${dir.path}", sub),
                onActivate = { importCocoon(dir.path) }
            )
        }
        rows += SettingRow(
            "zip",
            Icons.Rounded.FolderZip,
            s(R.string.st_theme_import_zip),
            resultOr("zip", s(R.string.st_theme_import_zip_sub)),
            onActivate = { model.wantZip = true }
        )
        return rows
    }

    /** A check mark when the item is the one in use. */
    private fun active(on: Boolean): Control = if (on) Control.Value("✓") else Control.None

    /** The result of the job the row [id] ran last, or [default] while it has none. */
    private fun resultOr(id: String, default: String): String {
        val busyDefault = if (model.busy && model.resultRow == null) s(R.string.st_working) else default
        val message = model.result?.takeIf { model.resultRow == id }
        return message?.let { formatUiMessage(context, it) } ?: busyDefault
    }

    /** Imports the themes and packs of a Cocoon data folder and reports the counts. */
    private fun importCocoon(path: String) {
        model.work("cocoon:$path") {
            val report = app.zl.core.call { it.importFromCocoon(path) }
            UiMessage(R.string.st_import_done, listOf(report.themesAdded.size.toString(), report.packsAdded.size.toString()))
        }
    }

    /** The choice of what to apply for the theme [page]: with its settings or only the theme. */
    fun apply(page: Page.ApplyTheme): List<SettingRow> {
        fun applyAs(withSettings: Boolean) {
            model.change { it.applyTheme(page.themeId, withSettings) }
            model.pop()
            model.pop()
        }
        return listOf(
            SettingRow(
                "with",
                Icons.Rounded.Brush,
                s(R.string.st_theme_apply_settings),
                s(R.string.st_theme_apply_settings_sub),
                onActivate = { applyAs(true) }
            ),
            SettingRow(
                "only",
                Icons.Rounded.Palette,
                s(R.string.st_theme_apply_only),
                s(R.string.st_theme_apply_only_sub),
                onActivate = { applyAs(false) }
            ),
        )
    }

    /** The packs of a layer: follow the theme, or one of the installed packs. */
    fun packs(category: AssetCategory): List<SettingRow> {
        val packs = model.data.packs.filter { it.category == category }
        val rows = mutableListOf<SettingRow>()
        rows += SettingRow(
            "follow",
            Icons.Rounded.Layers,
            s(R.string.st_pack_follow),
            null,
            active(packs.none { it.explicit }),
            onActivate = { model.change { it.setLayerPack(category, null) } }
        )
        packs.forEach { p ->
            val details = listOfNotNull(p.author, p.version?.let { "v$it" }).joinToString(" · ").ifEmpty { null }
            rows += SettingRow(
                "p:${p.id}",
                Icons.Rounded.Layers,
                p.name,
                details,
                active(p.inUse),
                onActivate = { model.change { it.setLayerPack(category, p.id) } },
                preview = p.preview
            )
        }
        return rows
    }

    /** The theme in use (kept for callers that style rows by it). */
    val current: ZTheme get() = theme
}
