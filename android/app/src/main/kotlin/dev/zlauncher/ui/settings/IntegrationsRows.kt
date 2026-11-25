package dev.zlauncher.ui.settings

import android.content.Context
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Image
import dev.zlauncher.R

/**
 * Builds rows for the Integrations category. Only integrations that work are listed: today SteamGridDB, whose key
 * the user pastes here (RetroAchievements, Discord and ScreenScraper come back when they are implemented).
 */
class IntegrationsRows(
    private val model: SettingsModel,
    private val context: Context,
) {
    /** A string resource. */
    private fun s(id: Int, vararg args: Any): String = context.getString(id, *args)

    /** Builds all rows of the Integrations root list. */
    fun root(): List<SettingRow> = listOf(steamGridDbRow())

    /** The SteamGridDB key: says whether one is stored; A opens the editor. */
    private fun steamGridDbRow(): SettingRow = SettingRow(
        id = "steamgriddb",
        icon = Icons.Rounded.Image,
        title = s(R.string.st_steamgrid),
        subtitle = s(if (model.data.hasSgdbKey) R.string.st_sgdb_key_set else R.string.st_sgdb_key_unset),
        control = Control.Chevron,
        onActivate = { model.wantSgdbKey = true },
    )
}
