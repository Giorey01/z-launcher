package dev.zlauncher.apps

import android.content.Intent
import android.content.pm.ApplicationInfo
import android.content.pm.PackageManager
import android.graphics.Bitmap
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.core.graphics.drawable.toBitmap
import java.util.concurrent.ConcurrentHashMap
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

/** An app with a launcher activity (RF-06). */
data class AppEntry(
    /** Android package name. */
    val packageName: String,
    /** Label the user sees. */
    val label: String,
    /** The app declares itself a game (`ApplicationInfo.CATEGORY_GAME`). */
    val isGame: Boolean,
)

/** Side of the icons kept in memory, in pixels. */
private const val ICON_SIZE_PX = 144

/** Reads the launcher activities through `PackageManager` (no `QUERY_ALL_PACKAGES`, only a `<queries>` entry). */
class InstalledApps(private val pm: PackageManager) {
    /** Icons already decoded, by package. */
    private val icons = ConcurrentHashMap<String, ImageBitmap>()

    /** Every app with a launcher activity, sorted by label ignoring case. */
    suspend fun list(): List<AppEntry> = withContext(Dispatchers.IO) {
        val query = Intent(Intent.ACTION_MAIN).addCategory(Intent.CATEGORY_LAUNCHER)
        pm.queryIntentActivities(query, 0)
            .map { it.activityInfo }
            .distinctBy { it.packageName }
            .map { info ->
                AppEntry(
                    packageName = info.packageName,
                    label = info.loadLabel(pm).toString(),
                    isGame = info.applicationInfo?.category == ApplicationInfo.CATEGORY_GAME,
                )
            }
            .sortedBy { it.label.lowercase() }
    }

    /** The icon of [packageName], decoded off the main thread and cached; `null` when it cannot be read. */
    suspend fun icon(packageName: String): ImageBitmap? = icons[packageName] ?: withContext(Dispatchers.IO) {
        try {
            val bitmap = pm.getApplicationIcon(packageName).toBitmap(ICON_SIZE_PX, ICON_SIZE_PX, Bitmap.Config.ARGB_8888)
            bitmap.asImageBitmap().also { icons[packageName] = it }
        } catch (_: PackageManager.NameNotFoundException) {
            null
        }
    }
}
