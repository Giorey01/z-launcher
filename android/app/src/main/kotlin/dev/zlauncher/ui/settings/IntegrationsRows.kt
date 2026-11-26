package dev.zlauncher.ui.settings

import android.content.Context
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.CloudDownload
import androidx.compose.material.icons.rounded.Extension
import androidx.compose.material.icons.rounded.Image
import androidx.compose.material.icons.rounded.SportsEsports
import dev.zlauncher.R
import dev.zlauncher.core.nativecore.ScanEvent
import dev.zlauncher.core.nativecore.ScanListener
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.Screen
import dev.zlauncher.nav.UiMessage

/** A scan listener that ignores the progress (the row only shows the result). */
private object QuietImportScan : ScanListener {
    /** Progress is not shown here. */
    override fun onEvent(event: ScanEvent) = Unit
}

/**
 * Builds rows for the Integrations category. Only integrations that work are listed: SteamGridDB, whose key the
 * user pastes here, and Steam, whose Web API key and profile let the launcher import the owned games
 * (RetroAchievements, Discord and ScreenScraper come back when they are implemented).
 */
class IntegrationsRows(
    private val app: AppViewModel,
    private val model: SettingsModel,
    private val context: Context,
) {
    /** A string resource. */
    private fun s(id: Int, vararg args: Any): String = context.getString(id, *args)

    /** Builds all rows of the Integrations root list. */
    fun root(): List<SettingRow> = listOf(steamGridDbRow(), steamAccountRow(), steamImportRow()) + podRows()

    /** The settings sections of the Pods: A opens the Pod's own screen for them. */
    private fun podRows(): List<SettingRow> = app.zl.pods.settingsSectionSlots().map { (podId, slot) ->
        SettingRow(
            id = "pod:$podId:${slot.label}",
            icon = Icons.Rounded.Extension,
            title = slot.label,
            subtitle = s(R.string.st_pod_section),
            control = Control.Chevron,
            onActivate = { app.open(Screen.Pod(podId, slot.route)) },
        )
    }

    /** The SteamGridDB key: says whether one is stored; A opens the editor. */
    private fun steamGridDbRow(): SettingRow = SettingRow(
        id = "steamgriddb",
        icon = Icons.Rounded.Image,
        title = s(R.string.st_steamgrid),
        subtitle = s(if (model.data.hasSgdbKey) R.string.st_sgdb_key_set else R.string.st_sgdb_key_unset),
        control = Control.Chevron,
        onActivate = { model.wantSgdbKey = true },
    )

    /** The Steam account: the profile when one is set; A opens the editor. */
    private fun steamAccountRow(): SettingRow {
        val steam = model.data.steam
        val subtitle = when {
            steam.account != null && steam.hasKey -> steam.account
            else -> s(R.string.st_steam_unset)
        }
        return SettingRow(
            id = "steam_account",
            icon = Icons.Rounded.SportsEsports,
            title = s(R.string.st_steam),
            subtitle = subtitle,
            control = Control.Chevron,
            onActivate = { model.wantSteamAccount = true },
        )
    }

    /** Imports the owned games as Steam tag files; the result replaces the subtitle. */
    private fun steamImportRow(): SettingRow {
        val running = model.busy && model.resultRowRunning == ROW_IMPORT
        val message = model.result?.takeIf { model.resultRow == ROW_IMPORT }
        val subtitle = when {
            running -> s(R.string.st_working)
            message != null -> formatUiMessage(context, message)
            else -> s(R.string.st_steam_import_sub)
        }
        return SettingRow(ROW_IMPORT, Icons.Rounded.CloudDownload, s(R.string.st_steam_import), subtitle, onActivate = ::import)
    }

    /** Runs the import, unless one is running or no account is set. */
    private fun import() {
        val steam = model.data.steam
        when {
            model.busy -> Unit
            !steam.hasKey || steam.account == null -> model.wantSteamAccount = true
            else -> model.work(ROW_IMPORT) {
                val report = app.zl.core.call { it.importSteamLibrary(QuietImportScan) }
                report.problem?.let(::steamProblemMessage)
                    ?: UiMessage(R.string.msg_steam_imported, listOf(report.added.toString(), report.present.toString()))
            }
        }
    }

    private companion object {
        /** Id of the import row. */
        const val ROW_IMPORT = "steam_import"
    }
}
