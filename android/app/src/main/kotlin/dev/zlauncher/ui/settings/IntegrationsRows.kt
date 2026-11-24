package dev.zlauncher.ui.settings

import android.content.Context
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.AccountCircle
import androidx.compose.material.icons.rounded.CloudSync
import androidx.compose.material.icons.rounded.EmojiEvents
import androidx.compose.material.icons.rounded.Image
import androidx.compose.material.icons.rounded.Share
import dev.zlauncher.R
import dev.zlauncher.core.nativecore.UserSettings

/** Builds rows for the Integrations category (RetroAchievements, Discord, SteamGridDB, ScreenScraper). */
class IntegrationsRows(
    private val model: SettingsModel,
    private val context: Context,
) {
    private fun s(id: Int, vararg args: Any): String = context.getString(id, *args)

    /** Builds all rows of the Integrations root list. */
    fun root(st: UserSettings): List<SettingRow> = listOf(
        retroAchievementsRow(st),
        discordPresenceRow(st),
        steamGridDbRow(),
        screenScraperRow(),
    )

    private fun retroAchievementsRow(st: UserSettings): SettingRow = SettingRow(
        id = "retroachievements",
        icon = Icons.Rounded.EmojiEvents,
        title = s(R.string.st_retroachievements),
        subtitle = s(R.string.st_retroachievements_sub),
        control = Control.Switch(st.retroachievementsEnabled) { on ->
            model.change { c -> c.setRetroachievementsEnabled(on) }
        },
    )

    private fun discordPresenceRow(st: UserSettings): SettingRow = SettingRow(
        id = "discord",
        icon = Icons.Rounded.Share,
        title = s(R.string.st_discord),
        subtitle = s(R.string.st_discord_sub),
        control = Control.Switch(st.discordPresenceEnabled) { on ->
            model.change { c -> c.setDiscordPresenceEnabled(on) }
        },
    )

    private fun steamGridDbRow(): SettingRow = SettingRow(
        id = "steamgriddb",
        icon = Icons.Rounded.Image,
        title = s(R.string.st_steamgrid),
        subtitle = s(R.string.st_steamgrid_sub),
        control = Control.Value("API"),
    )

    private fun screenScraperRow(): SettingRow = SettingRow(
        id = "screenscraper",
        icon = Icons.Rounded.CloudSync,
        title = s(R.string.st_screenscraper),
        subtitle = s(R.string.st_screenscraper_sub),
        control = Control.Value("Account"),
    )
}
