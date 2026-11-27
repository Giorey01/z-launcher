package dev.zlauncher.ui.onboarding

import android.app.Application
import android.content.Context
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import dev.zlauncher.R
import dev.zlauncher.ZLauncherApp
import dev.zlauncher.core.NewArt
import dev.zlauncher.core.RomHashing
import dev.zlauncher.core.nativecore.CocoonDataDir
import dev.zlauncher.core.nativecore.Core
import dev.zlauncher.core.nativecore.CoreException
import dev.zlauncher.core.nativecore.Detection
import dev.zlauncher.core.nativecore.LibraryMode
import dev.zlauncher.core.nativecore.RomRoot
import dev.zlauncher.core.nativecore.ScanEvent
import dev.zlauncher.core.nativecore.ScanListener
import dev.zlauncher.core.nativecore.ScanSummary
import dev.zlauncher.core.nativecore.pathOfSafTree
import dev.zlauncher.core.nativecore.suggestedRomFolder
import dev.zlauncher.launch.toMessage
import dev.zlauncher.nav.UiMessage
import dev.zlauncher.storage.Storage
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

/** Name of the preferences file that remembers the Cocoon offer. */
private const val PREFS = "setup"

/** Key: the Cocoon import was already offered. */
private const val KEY_OFFERED = "cocoon_offered"

/** The steps of the setup (RF-29), in order. */
enum class Step {
    /** Introduction. */
    Welcome,

    /** "All files access". */
    Permission,

    /** ROM folders. */
    Folders,

    /** Smart Folders or Simple. */
    Mode,

    /** Scan with live progress. */
    Scan,

    /** Detected emulators and proposed defaults. */
    Emulators,

    /** Offer to become the Home. */
    HomeRole,

    /** Offer to import the themes and media of a Cocoon data folder found on the device. */
    Cocoon,

    /** Finished. */
    Done,
}

/** Progress of the scan step. */
data class ScanUi(
    /** The scan is running. */
    val running: Boolean = false,
    /** Platform folder being read. */
    val platform: String? = null,
    /** 1-based position of that folder. */
    val index: Int = 0,
    /** Number of folders. */
    val total: Int = 0,
    /** Files looked at so far. */
    val files: Long = 0,
    /** The final numbers, once finished. */
    val summary: ScanSummary? = null,
)

/** Everything the setup screens draw. */
data class OnboardingUi(
    /** The current step. */
    val step: Step = Step.Welcome,
    /** The app may read all files. */
    val allFiles: Boolean = false,
    /** ROM folders added. */
    val roots: List<RomRoot> = emptyList(),
    /** Folders offered for adding. */
    val folders: List<String> = emptyList(),
    /** Chosen library mode. */
    val mode: LibraryMode = LibraryMode.SMART,
    /** The scan step. */
    val scan: ScanUi = ScanUi(),
    /** Emulators found. */
    val detection: Detection? = null,
    /** The app already is the device's Home. */
    val roleHeld: Boolean = false,
    /** The Cocoon data folder on offer, `null` when there is none or it was offered before. */
    val cocoon: CocoonDataDir? = null,
    /** The Cocoon import is running. */
    val importing: Boolean = false,
    /** Last problem, as a translated message. */
    val error: UiMessage? = null,
)

/** Drives the setup: permission, folders, mode, scan, emulators, Home role. Core calls run on IO. */
class OnboardingViewModel(application: Application) : AndroidViewModel(application) {
    /** The shared application object. */
    private val zl = application as ZLauncherApp

    /** Backing state. */
    private val state = MutableStateFlow(OnboardingUi())

    /** The state the screens draw. */
    val ui: StateFlow<OnboardingUi> = state.asStateFlow()

    init {
        refreshPermission()
        viewModelScope.launch {
            core { it.roots() }?.let { roots -> state.update { it.copy(roots = roots) } }
        }
    }

    /** Runs a core call; an error becomes a message in the state and `null`. */
    private suspend fun <T> core(block: (Core) -> T): T? = try {
        zl.core.call(block)
    } catch (e: CoreException) {
        state.update { it.copy(error = e.toMessage()) }
        null
    }

    /** Rereads the all-files permission and the folders on offer (after coming back from Settings). */
    fun refreshPermission() {
        val allowed = Storage.hasAllFilesAccess()
        val folders = Storage.candidateFolders(getApplication(), suggestedRomFolder())
        state.update { it.copy(allFiles = allowed, folders = folders) }
    }

    /** Starts the setup from the first step (it was run before and is being run again). */
    fun restart() {
        state.value = OnboardingUi()
        refreshPermission()
        viewModelScope.launch {
            val offered = zl.getSharedPreferences(PREFS, Context.MODE_PRIVATE).getBoolean(KEY_OFFERED, false)
            val found = if (offered) null else core { it.detectCocoonDataDirs() }?.firstOrNull()
            state.update { it.copy(cocoon = found) }
        }
        viewModelScope.launch {
            core { it.roots() }?.let { roots -> state.update { s -> s.copy(roots = roots) } }
        }
    }

    /** Records whether the app is the device's Home. */
    fun setRoleHeld(held: Boolean) = state.update { it.copy(roleHeld = held) }

    /** Moves to [step]. */
    fun go(step: Step) = state.update { it.copy(step = step, error = null) }

    /** The step after the current one; the permission step is skipped when already granted. */
    fun next() {
        val current = state.value
        val target = Step.entries[(current.step.ordinal + 1).coerceAtMost(Step.Done.ordinal)]
        go(
            when {
                target == Step.Permission && current.allFiles -> Step.Folders
                target == Step.Cocoon && current.cocoon == null -> Step.Done
                else -> target
            },
        )
    }

    /** Back one step; `false` at the first step or while the scan runs. */
    fun back(): Boolean {
        val current = state.value
        if (current.step == Step.Welcome || current.scan.running) return false
        val target = Step.entries[current.step.ordinal - 1]
        go(
            when {
                target == Step.Permission && current.allFiles -> Step.Welcome
                target == Step.Cocoon && current.cocoon == null -> Step.HomeRole
                else -> target
            },
        )
        return true
    }

    /** Adds the folder the user granted: the core derives the path from the tree and stores both. */
    fun addPickedTree(treeUri: String) {
        viewModelScope.launch {
            val root = core { core ->
                val path = pathOfSafTree(treeUri)
                path?.let { core.addRoot(it, treeUri) }
            }
            if (root == null) {
                state.update { it.copy(error = UiMessage(R.string.ob_folder_unreadable)) }
            } else {
                core { it.roots() }?.let { roots -> state.update { s -> s.copy(roots = roots) } }
            }
        }
    }

    /** Stores the library mode and moves on. */
    fun chooseMode(mode: LibraryMode) {
        viewModelScope.launch {
            core { it.setLibraryMode(mode) } ?: return@launch
            state.update { it.copy(mode = mode) }
            next()
        }
    }

    /** Scans the folders, updating the progress as events arrive. */
    fun startScan() {
        if (state.value.scan.running) return
        state.update { it.copy(scan = ScanUi(running = true)) }
        viewModelScope.launch {
            val since = NewArt.now()
            val summary = core { it.scan(listener) }
            summary?.let { NewArt.afterScan(zl, since, it.added) }
            summary?.let { RomHashing.afterScan(zl) }
            state.update { it.copy(scan = it.scan.copy(running = false, summary = summary ?: it.scan.summary)) }
        }
    }

    /** Receives the events of the scan thread. */
    private val listener = object : ScanListener {
        /** Updates the progress; the state is thread-safe. */
        override fun onEvent(event: ScanEvent) {
            state.update { s ->
                val scan = s.scan
                s.copy(
                    scan = when (event) {
                        is ScanEvent.Started -> scan.copy(total = event.platforms.toInt())
                        is ScanEvent.PlatformStarted -> scan.copy(platform = event.platformId, index = event.index.toInt(), total = event.total.toInt())
                        is ScanEvent.FilesSeen -> scan.copy(files = event.count.toLong())
                        is ScanEvent.Finished -> scan.copy(summary = event.summary, files = event.summary.filesSeen.toLong())
                    },
                )
            }
        }
    }

    /** Detects the emulators installed on the device. */
    fun detect() {
        viewModelScope.launch {
            val packages = zl.apps.list().map { it.packageName }
            core { it.detectInstalled(packages) }?.let { found -> state.update { it.copy(detection = found) } }
        }
    }

    /** Saves the proposed defaults and moves on. */
    fun acceptDefaults() {
        viewModelScope.launch {
            val packages = zl.apps.list().map { it.packageName }
            core { it.acceptProposedDefaults(packages) } ?: return@launch
            next()
        }
    }

    /** Remembers that the Cocoon import was offered, so it is not offered again. */
    private fun markOffered() {
        zl.getSharedPreferences(PREFS, Context.MODE_PRIVATE).edit().putBoolean(KEY_OFFERED, true).apply()
    }

    /** The user declined the Cocoon import. */
    fun skipCocoon() {
        markOffered()
        next()
    }

    /**
     * Imports the themes and the media of the Cocoon folder, applies the first installed theme (the core cannot know
     * which one Cocoon had active; the user can change it in the settings) and moves on.
     */
    fun importCocoon() {
        val dir = state.value.cocoon ?: return
        state.update { it.copy(importing = true) }
        viewModelScope.launch {
            val done = core { c ->
                c.importFromCocoon(dir.path)
                c.importMedia()
                c.installedThemes().firstOrNull()?.let { c.applyTheme(it.id, true) }
            }
            markOffered()
            if (done != null) {
                try {
                    zl.themeStore.reload()
                } catch (e: CoreException) {
                    state.update { it.copy(error = e.toMessage()) }
                }
            }
            state.update { it.copy(importing = false) }
            next()
        }
    }

    /** Marks the setup as finished in the core. */
    fun finish(onDone: () -> Unit) {
        viewModelScope.launch {
            core { it.setOnboardingDone(true) } ?: return@launch
            onDone()
        }
    }
}
