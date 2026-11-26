package dev.zlauncher.pods

import android.net.Uri
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

/** A row of the Pod screen that is not a Pod: the switch and the install actions. */
enum class PodAction {
    /** The "external Pods" switch (§6). */
    EXTERNAL_TOGGLE,

    /** Install a `.zlpod` chosen with the system file picker. */
    INSTALL_FILE,

    /** Install a `.zlpod` from a link typed with the on-screen keyboard. */
    INSTALL_URL,

    /** Load the store index again. */
    REFRESH_STORE,
}

/** What the cursor is on in the Pod screen: the details panel describes it. */
sealed interface PodFocus {
    /** An installed external Pod. */
    data class Installed(
        /** The Pod. */
        val pod: InstalledPodView,
    ) : PodFocus

    /** A Pod built into this launcher. */
    data class Builtin(
        /** The Pod and its declarations. */
        val entry: PodWithMetadata,
    ) : PodFocus

    /** An entry of the store index. */
    data class Store(
        /** The index entry. */
        val entry: IndexEntryView,
    ) : PodFocus

    /** One of the action rows. */
    data class Action(
        /** Which action. */
        val action: PodAction,
    ) : PodFocus
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
    /** The row under the cursor, described by the details panel. */
    val focus: PodFocus? = null,
    /** The screen should open the system file picker for a `.zlpod`. */
    val wantFile: Boolean = false,
    /** Installed Pod whose actions (open, enable, uninstall…) are shown. */
    val actionsFor: InstalledPodView? = null,
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

    /** The tabs to show: Built-in only when this build has built-in Pods. */
    fun tabs(): List<PodTab> = PodTab.entries.filter { it != PodTab.BUILTIN || state.builtinPods.isNotEmpty() }

    /** Moves to the previous ([delta] < 0) or next tab, stopping at the ends (L1 and R1). */
    fun stepTab(delta: Int) {
        val tabs = tabs()
        val next = tabs.getOrNull(tabs.indexOf(state.activeTab) + delta) ?: return
        selectTab(next)
    }

    /** Closes the open dialog, if any (B in the Pod screen); `false` when none was open. */
    fun dismissDialog(): Boolean {
        val s = state
        if (s.pendingPreview != null) {
            cancelPendingInstall()
            return true
        }
        val open = s.showExternalPrompt || s.showUrlDialog || s.showUninstallConfirm != null || s.actionsFor != null
        if (open) {
            state = s.copy(showExternalPrompt = false, pendingUrlAfterPrompt = null, showUrlDialog = false, showUninstallConfirm = null, actionsFor = null)
        }
        return open
    }

    /** Remembers the row under the cursor for the details panel. */
    fun focus(focus: PodFocus) {
        state = state.copy(focus = focus)
    }

    /** Shows or hides the actions of an installed Pod. */
    fun showActions(pod: InstalledPodView?) {
        state = state.copy(actionsFor = pod)
    }

    /** Asks the screen for the file picker; with external Pods off, asks to turn them on first (§6). */
    fun requestFile() {
        state = if (state.externalPodsEnabled) state.copy(wantFile = true) else state.copy(showExternalPrompt = true)
    }

    /** The screen opened the file picker. */
    fun fileRequested() {
        state = state.copy(wantFile = false)
    }

    /** Copies the chosen `.zlpod` into the cache and inspects it, like a download (§6). */
    @Suppress("TooGenericExceptionCaught")
    fun inspectUri(uri: Uri) {
        app.viewModelScope.launch {
            state = state.copy(inspecting = true)
            val tempFile = File(app.zl.cacheDir, "pod_${System.currentTimeMillis()}.zlpod")
            try {
                withContext(Dispatchers.IO) {
                    val input = checkNotNull(app.zl.contentResolver.openInputStream(uri)) { "no stream for $uri" }
                    input.use { i -> tempFile.outputStream().use { o -> i.copyTo(o) } }
                }
                inspectDownloaded(tempFile, null, null)
            } catch (_: Exception) {
                tempFile.delete()
                state = state.copy(inspecting = false)
                app.message = UiMessage(R.string.pod_error_file)
            }
        }
    }

    /** Inspects a package already on disk and shows the confirmation (§6); a bad package shows its codes. */
    private suspend fun inspectDownloaded(file: File, expectedHash: String?, expectedSigner: String?) {
        try {
            val preview = app.zl.core.call { it.inspectPod(file.absolutePath) }
            state = state.copy(
                inspecting = false,
                pendingPreview = preview,
                pendingPackageFile = file,
                pendingHash = expectedHash,
                pendingSigner = expectedSigner,
            )
        } catch (e: CoreException) {
            file.delete()
            state = state.copy(inspecting = false)
            app.message = e.toMessage()
        }
    }

    /** Selects the active tab. */
    fun selectTab(tab: PodTab) {
        state = state.copy(activeTab = tab, focus = null)
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
            } catch (_: Exception) {
                tempFile.delete()
                state = state.copy(inspecting = false)
                app.message = UiMessage(R.string.pod_error_download)
                return@launch
            }
            inspectDownloaded(tempFile, expectedHash, expectedSigner)
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
        // The switch is saved in the background: the pending install must already see it on.
        state = state.copy(showExternalPrompt = false, pendingUrlAfterPrompt = null, externalPodsEnabled = true)
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
