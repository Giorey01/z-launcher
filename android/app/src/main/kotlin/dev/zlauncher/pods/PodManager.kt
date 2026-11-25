package dev.zlauncher.pods

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.viewModelScope
import dev.zlauncher.R
import dev.zlauncher.core.nativecore.CoreException
import dev.zlauncher.core.nativecore.IndexEntryView
import dev.zlauncher.core.nativecore.InstalledPodView
import dev.zlauncher.core.nativecore.PodPreview
import dev.zlauncher.core.nativecore.defaultPodIndexUrl
import dev.zlauncher.launch.toMessage
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.UiMessage
import java.io.File
import java.net.HttpURLConnection
import java.net.URL
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/** The active tab in the Pod management screen. */
enum class PodTab {
    /** Installed external Pods. */
    INSTALLED,

    /** Built-in Pods provided by the build. */
    BUILTIN,

    /** Store index and remote installation. */
    STORE,
}

/** State of the Pod management and store screens (§11). */
data class PodManagementState(
    /** Installed external Pods from the core. */
    val installedPods: List<InstalledPodView> = emptyList(),
    /** Built-in Pods registered via ServiceLoader. */
    val builtinPods: List<PodWithMetadata> = emptyList(),
    /** Whether external Pod loading is enabled. */
    val externalPodsEnabled: Boolean = false,
    /** Whether store or list is loading. */
    val loading: Boolean = false,
    /** Store index entries fetched from remote. */
    val storeEntries: List<IndexEntryView> = emptyList(),
    /** Whether the store index is currently being loaded. */
    val storeLoading: Boolean = false,
    /** Issue or failure message while loading store index. */
    val storeError: Boolean = false,
    /** Pending package preview awaiting user confirmation. */
    val pendingPreview: PodPreview? = null,
    /** Temp file for the pending download. */
    val pendingPackageFile: File? = null,
    /** Expected SHA256 hash for pending package. */
    val pendingHash: String? = null,
    /** Expected signer for pending package. */
    val pendingSigner: String? = null,
    /** Show dialog asking user to enable external Pods (§6). */
    val showExternalPrompt: Boolean = false,
    /** Show URL input dialog with keyboard. */
    val showUrlDialog: Boolean = false,
    /** Pod to confirm uninstall for. */
    val showUninstallConfirm: InstalledPodView? = null,
    /** Downloading or inspecting in progress. */
    val inspecting: Boolean = false,
    /** Selected tab in management screen. */
    val activeTab: PodTab = PodTab.INSTALLED,
    /** Pending URL remembered while asking to enable external Pods. */
    val pendingUrlAfterPrompt: String? = null,
)

/** Characters shown from the signer fingerprint (§4). */
private const val FINGERPRINT_DISPLAY_LEN = 32

/** Size of chunks for displaying key fingerprint. */
private const val FINGERPRINT_CHUNK_SIZE = 4

/** Bytes in one kilobyte. */
private const val ONE_KB = 1024L

/** Bytes in one megabyte. */
private const val ONE_MB = 1024L * 1024L

/** Network connection timeout in milliseconds. */
private const val CONNECT_TIMEOUT_MS = 10_000

/** Read timeout for remote index JSON in milliseconds. */
private const val READ_INDEX_TIMEOUT_MS = 15_000

/** Read timeout for package download in milliseconds. */
private const val READ_PACKAGE_TIMEOUT_MS = 25_000

/** Formats a signer public key fingerprint in uppercase 4-character chunks (§4). */
fun formatSignerFingerprint(signer: String): String =
    signer.take(FINGERPRINT_DISPLAY_LEN).uppercase().chunked(FINGERPRINT_CHUNK_SIZE).joinToString(" ")

/** Formats byte size into human readable string. */
fun formatFileSize(bytes: Long): String = if (bytes < ONE_MB) {
    "${bytes / ONE_KB} KB"
} else {
    val mb = bytes.toDouble() / ONE_MB
    "%.1f MB".format(mb)
}

/** Controller for managing installed Pods, browsing the store, and executing installation. */
class PodManager(private val app: AppViewModel) {
    /** Observable state of the Pod management interface. */
    var state by mutableStateOf(PodManagementState())
        private set

    /** Selects the active tab. */
    fun selectTab(tab: PodTab) {
        state = state.copy(activeTab = tab)
        if (tab == PodTab.STORE && state.storeEntries.isEmpty() && !state.storeLoading) {
            fetchStoreIndex()
        }
    }

    /** Reloads the installed and built-in pods list from the core. */
    fun reload() {
        app.viewModelScope.launch {
            state = state.copy(loading = true)
            val enabled = runCatching { app.zl.core.call { it.externalPodsEnabled() } }.getOrDefault(false)
            val installed = runCatching { app.zl.core.call { it.listPods() } }.getOrDefault(emptyList())
            val builtin = app.zl.pods.entries.filter { !it.isExternal }
            state = state.copy(
                installedPods = installed,
                builtinPods = builtin,
                externalPodsEnabled = enabled,
                loading = false,
            )
        }
    }

    /** Enables or disables external Pod support. */
    fun setExternalEnabled(enabled: Boolean) {
        app.viewModelScope.launch {
            runCatching {
                app.zl.core.call { it.setExternalPodsEnabled(enabled) }
                app.zl.reloadPods()
            }
            reload()
        }
    }

    /** Enables or disables a specific installed Pod. */
    fun setPodEnabled(id: String, enabled: Boolean) {
        app.viewModelScope.launch {
            runCatching {
                app.zl.core.call { it.setPodEnabled(id, enabled) }
                app.zl.reloadPods()
            }
            reload()
        }
    }

    /** Resets the crash health counter and reactivates a Pod disabled by crashes (§10). */
    fun resetHealth(id: String) {
        app.viewModelScope.launch {
            runCatching {
                app.zl.core.call { it.resetPodHealth(id) }
                app.zl.reloadPods()
            }
            reload()
        }
    }

    /** Uninstalls an external Pod, removing its trust record and files (§4, §6). */
    fun uninstall(id: String) {
        app.viewModelScope.launch {
            runCatching {
                app.zl.core.call { it.uninstallPod(id) }
                app.zl.reloadPods()
            }
            state = state.copy(showUninstallConfirm = null)
            app.message = UiMessage(R.string.pod_uninstalled_success)
            reload()
        }
    }

    /** Asks confirmation to uninstall [pod]. */
    fun confirmUninstall(pod: InstalledPodView?) {
        state = state.copy(showUninstallConfirm = pod)
    }

    /** Opens or closes the URL entry dialog. */
    fun showUrlInput(show: Boolean) {
        state = state.copy(showUrlDialog = show)
    }

    /** Fetches and parses the remote Pod store index JSON (§5). */
    @Suppress("TooGenericExceptionCaught")
    fun fetchStoreIndex(url: String = defaultPodIndexUrl()) {
        app.viewModelScope.launch {
            state = state.copy(storeLoading = true, storeError = false)
            try {
                val bytes = withContext(Dispatchers.IO) {
                    val conn = URL(url).openConnection() as HttpURLConnection
                    conn.connectTimeout = CONNECT_TIMEOUT_MS
                    conn.readTimeout = READ_INDEX_TIMEOUT_MS
                    conn.inputStream.use { it.readBytes() }
                }
                val indexView = app.zl.core.call { it.parsePodIndex(bytes) }
                state = state.copy(storeEntries = indexView.entries, storeLoading = false)
            } catch (_: Exception) {
                state = state.copy(storeLoading = false, storeError = true)
            }
        }
    }

    /** Downloads a `.zlpod` package and inspects its preview metadata (§6). */
    @Suppress("TooGenericExceptionCaught")
    fun downloadAndInspect(url: String, expectedHash: String? = null, expectedSigner: String? = null) {
        if (!state.externalPodsEnabled) {
            state = state.copy(showExternalPrompt = true, pendingUrlAfterPrompt = url)
            return
        }
        app.viewModelScope.launch {
            state = state.copy(inspecting = true)
            val tempFile = File(app.zl.cacheDir, "pod_${System.currentTimeMillis()}.zlpod")
            try {
                withContext(Dispatchers.IO) {
                    val conn = URL(url).openConnection() as HttpURLConnection
                    conn.connectTimeout = CONNECT_TIMEOUT_MS
                    conn.readTimeout = READ_PACKAGE_TIMEOUT_MS
                    conn.inputStream.use { input ->
                        tempFile.outputStream().use { output ->
                            input.copyTo(output)
                        }
                    }
                }
                val preview = app.zl.core.call { it.inspectPod(tempFile.absolutePath) }
                state = state.copy(
                    inspecting = false,
                    pendingPreview = preview,
                    pendingPackageFile = tempFile,
                    pendingHash = expectedHash,
                    pendingSigner = expectedSigner,
                )
            } catch (_: Exception) {
                tempFile.delete()
                state = state.copy(inspecting = false)
                app.message = UiMessage(R.string.pod_error_download)
            }
        }
    }

    /** Confirms and executes installation of the pending package (§6). */
    @Suppress("TooGenericExceptionCaught")
    fun confirmPendingInstall() {
        val file = state.pendingPackageFile ?: return
        val hash = state.pendingHash
        val signer = state.pendingSigner
        app.viewModelScope.launch {
            try {
                val installed = app.zl.core.call { it.installPod(file.absolutePath, hash, signer) }
                file.delete()
                state = state.copy(
                    pendingPreview = null,
                    pendingPackageFile = null,
                    pendingHash = null,
                    pendingSigner = null,
                )
                app.zl.reloadPods()
                reload()
                app.message = UiMessage(R.string.pod_installed_success, listOf(installed.name))
            } catch (e: CoreException) {
                file.delete()
                state = state.copy(
                    pendingPreview = null,
                    pendingPackageFile = null,
                    pendingHash = null,
                    pendingSigner = null,
                )
                app.message = e.toMessage()
            } catch (e: Exception) {
                file.delete()
                state = state.copy(
                    pendingPreview = null,
                    pendingPackageFile = null,
                    pendingHash = null,
                    pendingSigner = null,
                )
                app.message = UiMessage(R.string.msg_pod, listOf(e.message.orEmpty()))
            }
        }
    }

    /** Cancels pending installation and deletes the temporary download. */
    fun cancelPendingInstall() {
        state.pendingPackageFile?.delete()
        state = state.copy(
            pendingPreview = null,
            pendingPackageFile = null,
            pendingHash = null,
            pendingSigner = null,
        )
    }

    /** Handles opening a deep link (`zlauncher://pod?url=...`). */
    fun handleDeepLink(url: String) {
        if (!state.externalPodsEnabled) {
            state = state.copy(showExternalPrompt = true, pendingUrlAfterPrompt = url)
        } else {
            downloadAndInspect(url)
        }
    }

    /** Called when the user confirms enabling external Pods in the prompt dialog. */
    fun onExternalPromptConfirmed() {
        val pendingUrl = state.pendingUrlAfterPrompt
        state = state.copy(showExternalPrompt = false, pendingUrlAfterPrompt = null)
        setExternalEnabled(true)
        if (!pendingUrl.isNullOrBlank()) {
            downloadAndInspect(pendingUrl)
        }
    }

    /** Called when the user dismisses the external Pods prompt dialog. */
    fun onExternalPromptDismissed() {
        state = state.copy(showExternalPrompt = false, pendingUrlAfterPrompt = null)
    }
}
