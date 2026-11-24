package dev.zlauncher.ui.settings

import android.content.Context
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.CreateNewFolder
import androidx.compose.material.icons.rounded.Folder
import androidx.compose.material.icons.rounded.Image
import androidx.compose.material.icons.rounded.Memory
import androidx.compose.material.icons.rounded.Refresh
import androidx.compose.material.icons.rounded.ViewModule
import dev.zlauncher.R
import dev.zlauncher.core.nativecore.LibraryMode
import dev.zlauncher.core.nativecore.ScanEvent
import dev.zlauncher.core.nativecore.ScanListener
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.UiMessage

/** A scan listener that ignores the progress (the settings only show the result). */
private object SilentScan : ScanListener {
    /** Progress is not shown here. */
    override fun onEvent(event: ScanEvent) = Unit
}

/** Rows of the Library and data category: folders, library mode, emulators, media import and scan. */
class LibraryRows(private val app: AppViewModel, private val model: SettingsModel, private val context: Context) {
    /** A string resource. */
    private fun s(id: Int, vararg args: Any): String = context.getString(id, *args)

    /** The result of the job row [id] ran last, "Working…" while it runs, or [default]. */
    private fun resultOr(id: String, default: String): String {
        if (model.busy) return s(R.string.st_working)
        val message = model.result?.takeIf { model.resultRow == id }
        return message?.let { formatUiMessage(context, it) } ?: default
    }

    /** The root list of the category. */
    fun root(): List<SettingRow> {
        val rows = mutableListOf<SettingRow>()
        rows += SettingRow(
            "roots",
            Icons.Rounded.Folder,
            s(R.string.st_roots),
            s(R.string.st_roots_sub),
            Control.Chevron,
            onActivate = { model.push(Page.Roots) }
        )
        val modes = listOf(LibraryMode.SMART, LibraryMode.SIMPLE)
        val modeNames = listOf(s(R.string.ob_mode_smart), s(R.string.ob_mode_simple))
        val modeIndex = modes.indexOf(model.data.libraryMode).coerceAtLeast(0)
        rows += SettingRow(
            "mode",
            Icons.Rounded.ViewModule,
            s(R.string.st_library_mode),
            s(R.string.st_library_mode_sub),
            Control.Dropdown(modeNames, modeIndex) { i -> model.change { it.setLibraryMode(modes[i]) } }
        )
        rows += SettingRow(
            "emus",
            Icons.Rounded.Memory,
            s(R.string.st_emulators),
            s(R.string.st_emulators_sub),
            Control.Chevron,
            onActivate = { model.push(Page.Emulators) }
        )
        rows += SettingRow(
            "media",
            Icons.Rounded.Image,
            s(R.string.st_media),
            resultOr("media", s(R.string.st_media_sub)),
            onActivate = ::importMedia
        )
        rows += scanRow()
        return rows
    }

    /** The "scan the library" action. */
    private fun scanRow(): SettingRow = SettingRow(
        "scan",
        Icons.Rounded.Refresh,
        s(R.string.st_rescan),
        resultOr("scan", s(R.string.st_rescan_sub)),
        onActivate = ::scan
    )

    /** Runs the media import and reports how many images were matched. */
    private fun importMedia() {
        if (model.busy) return
        model.work("media") {
            val report = app.zl.core.call { it.importMedia() }
            UiMessage(R.string.st_media_result, listOf(report.counts.sumOf { it.count.toInt() }.toString(), report.unmatched.toString()))
        }
    }

    /** Scans the enabled folders and reports what was found. */
    private fun scan() {
        if (model.busy) return
        model.work("scan") {
            val summary = app.zl.core.call { it.scan(SilentScan) }
            UiMessage(R.string.st_scan_result, listOf(summary.added.toString(), summary.updated.toString(), summary.missing.toString()))
        }
    }

    /** The ROM folders with a switch each (Y removes one), the add action and the scan. */
    fun roots(): List<SettingRow> {
        val rows = model.data.roots.map { root ->
            SettingRow(
                "r:${root.id}",
                Icons.Rounded.Folder,
                root.path,
                s(R.string.st_root_hint),
                Control.Switch(root.enabled) { on -> model.change { it.setRootEnabled(root.id, on) } },
                onSecondary = { model.change { it.removeRoot(root.id) } }
            )
        }
        val addRow = SettingRow(
            "add",
            Icons.Rounded.CreateNewFolder,
            s(R.string.st_root_add),
            s(R.string.st_root_add_sub),
            onActivate = { model.wantFolder = true }
        )
        return rows + addRow + scanRow()
    }

    /** The default emulator of each platform as a dropdown of the installed candidates. */
    fun emulators(): List<SettingRow> = model.data.emulators.map { platform ->
        val names = platform.candidates.map { c -> c.coreName?.let { s(R.string.ob_emu_with_core, c.emulatorName, it) } ?: c.emulatorName }
        val current = platform.candidates.indexOfFirst { it.profile == platform.current?.profile }.coerceAtLeast(0)
        val control = if (names.isEmpty()) {
            Control.Value(s(R.string.st_emulators_none))
        } else {
            Control.Dropdown(names, current) { i ->
                model.change { it.setPlatformDefault(platform.platformId, platform.candidates[i].profile) }
            }
        }
        SettingRow("e:${platform.platformId}", Icons.Rounded.Memory, platform.platformName, null, control)
    }
}
