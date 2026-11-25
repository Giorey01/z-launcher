package dev.zlauncher.pods

import android.content.Context
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.vector.ImageVector

/**
 * A destination screen or subview within a Pod.
 * [screen] identifies the target screen ("main" by default).
 * [args] carries optional route arguments.
 */
data class PodRoute(
    val screen: String = "main",
    val args: Map<String, String> = emptyMap(),
)

/**
 * A Pod (RF-73, v2): an extension loaded from a signed `.zlpod` package (or built into the launcher)
 * that provides custom screens, slots, and services.
 * See `docs/spec/pods.md` §7, §8, §9.
 */
interface Pod {
    /** Stable id: lower-case letters, digits, `.`, `-` and `_`, matching manifest.json. */
    val id: String

    /** Main icon of the Pod. */
    val icon: ImageVector

    /** Display title of the Pod in the app's current language. */
    fun title(context: Context): String

    /** The slots this Pod declares for host surfaces (§7). */
    fun slots(): List<PodSlot> = emptyList()

    /** Content entry point for routes opened by the host. Delegates to [Content] for v1 backwards compatibility. */
    @Composable
    fun Content(host: PodHost, route: PodRoute) {
        Content(host)
    }

    /** Compatibility overload for v1 Pods without route support. */
    @Composable
    fun Content(host: PodHost) = Unit
}

/** Provides Pods; implementations are found with `ServiceLoader` and need a public no-argument constructor. */
interface PodProvider {
    /** The Pods provided by this provider. */
    fun pods(): List<Pod>
}
