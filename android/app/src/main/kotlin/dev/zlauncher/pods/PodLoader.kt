package dev.zlauncher.pods

import android.content.Context
import dev.zlauncher.core.nativecore.InstalledPodView
import java.io.File

/** Log tag of the Pod loader. */
private const val LOG_TAG = "PodLoader"

/** Result of initializing external Pod loading at app startup (§10). */
data class PodLoadResult(
    /** Loaded external pods with metadata. */
    val externalPods: List<PodWithMetadata>,
    /** Whether safe mode was triggered due to an unhandled crash during previous loading. */
    val safeModeTriggered: Boolean,
    /** Id of the pod that crashed during loading, if safe mode triggered. */
    val crashedPodId: String? = null,
)

/** Encapsulates an active Pod alongside its manifest slot declarations and permissions. */
data class PodWithMetadata(
    /** The runtime Pod instance. */
    val pod: Pod,
    /** Slots declared in manifest.json. */
    val declaredSlots: Set<String>,
    /** Permissions declared in manifest.json. */
    val permissions: Set<String>,
    /** Whether this is an externally installed Pod (`true`) or a builtin Pod (`false`). */
    val isExternal: Boolean = false,
    /** The `i18n/` folder of an installed Pod, null for a built-in one (§8). */
    val i18nDir: File? = null,
)

/**
 * Loads external Pods from disk using DexClassLoader with safe-mode sentinel isolation (§9, §10).
 */
class PodLoader(
    private val context: Context? = null,
    private val podsDir: File,
    private val sentinelFile: File = File(podsDir, "loading"),
) {
    /**
     * Checks sentinel file for previous crashes during loading, and loads active external pods.
     * If a sentinel file exists, enters safe mode for this launch and records crash for that pod.
     */
    fun loadExternalPods(activePods: List<InstalledPodView>, onCrash: (podId: String, reason: String) -> Unit): PodLoadResult {
        if (sentinelFile.exists()) {
            val crashedId = runCatching { sentinelFile.readText().trim() }.getOrDefault("unknown")
            sentinelFile.delete()
            onCrash(crashedId, "Process died during DexClassLoader initialization")
            return PodLoadResult(emptyList(), safeModeTriggered = true, crashedPodId = crashedId)
        }

        val loaded = mutableListOf<PodWithMetadata>()
        for (info in activePods) {
            val pod = loadSinglePod(info, onCrash)
            if (pod != null) {
                loaded += pod
            }
        }
        return PodLoadResult(loaded, safeModeTriggered = false)
    }

    @Suppress("TooGenericExceptionCaught")
    private fun loadSinglePod(info: InstalledPodView, onCrash: (podId: String, reason: String) -> Unit): PodWithMetadata? {
        val podDir = File(podsDir, "${info.id}/${info.version}")
        val dexFile = File(podDir, "classes.dex")
        if (!dexFile.isFile) return null

        var result: PodWithMetadata? = null
        try {
            sentinelFile.writeText(info.id)
            val classLoader = createClassLoader(dexFile, File(podDir, "lib/arm64-v8a"))
            val entryClass = classLoader.loadClass(info.entry)
            val instance = entryClass.getDeclaredConstructor().newInstance()
            val pod = when (instance) {
                is Pod -> instance
                is PodProvider -> instance.pods().firstOrNull { it.id == info.id }
                else -> null
            }
            if (pod != null) {
                result = PodWithMetadata(
                    pod = pod,
                    declaredSlots = info.slots.toSet(),
                    permissions = info.permissions.toSet(),
                    isExternal = true,
                    i18nDir = File(podDir, "i18n"),
                )
            }
        } catch (e: Exception) {
            loadFailed(info.id, e, onCrash)
        } catch (e: LinkageError) {
            // A Pod built against classes or methods this launcher does not have (another version, or code R8 removed).
            loadFailed(info.id, e, onCrash)
        } finally {
            sentinelFile.delete()
        }
        return result
    }

    /** Records a Pod that did not load as a crash (§10) and writes the cause to the log. */
    private fun loadFailed(id: String, error: Throwable, onCrash: (podId: String, reason: String) -> Unit) {
        android.util.Log.w(LOG_TAG, "Pod $id did not load", error)
        onCrash(id, "Failed to load entry class: $error")
    }

    private fun createClassLoader(dexFile: File, nativeLibDir: File): ClassLoader {
        return try {
            val parentLoader = context?.classLoader ?: ClassLoader.getSystemClassLoader()
            val dexOptDir = File(context?.cacheDir ?: podsDir, "dexopt").apply { mkdirs() }
            val nativePath = if (nativeLibDir.isDirectory) nativeLibDir.absolutePath else null
            dalvik.system.DexClassLoader(dexFile.absolutePath, dexOptDir.absolutePath, nativePath, parentLoader)
        } catch (_: NoClassDefFoundError) {
            context?.classLoader ?: ClassLoader.getSystemClassLoader()
        }
    }
}
