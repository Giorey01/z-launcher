package dev.zlauncher.apps

import android.content.Intent
import android.content.pm.ApplicationInfo
import android.content.pm.PackageManager
import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.drawable.AdaptiveIconDrawable
import android.graphics.drawable.Drawable
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

/** Size of an adaptive icon canvas, in its own units. */
private const val ADAPTIVE_FULL = 108f

/** Part of the canvas that stays visible after the system mask. */
private const val ADAPTIVE_VISIBLE = 72f

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
            renderIcon(pm.getApplicationIcon(packageName)).asImageBitmap().also { icons[packageName] = it }
        } catch (_: PackageManager.NameNotFoundException) {
            null
        }
    }

    /**
     * Draws an icon without the system's mask: an adaptive icon gets its background and foreground drawn on the
     * visible 72 of its 108 units, so the launcher can clip it to its own shape; other icons are drawn as they are.
     */
    private fun renderIcon(icon: Drawable): Bitmap {
        if (icon !is AdaptiveIconDrawable) return icon.toBitmap(ICON_SIZE_PX, ICON_SIZE_PX, Bitmap.Config.ARGB_8888)
        val bitmap = Bitmap.createBitmap(ICON_SIZE_PX, ICON_SIZE_PX, Bitmap.Config.ARGB_8888)
        val canvas = Canvas(bitmap)
        val full = (ICON_SIZE_PX * ADAPTIVE_FULL / ADAPTIVE_VISIBLE).toInt()
        val margin = (full - ICON_SIZE_PX) / 2
        for (layer in listOfNotNull(icon.background, icon.foreground)) {
            layer.setBounds(-margin, -margin, ICON_SIZE_PX + margin, ICON_SIZE_PX + margin)
            layer.draw(canvas)
        }
        return bitmap
    }
}
