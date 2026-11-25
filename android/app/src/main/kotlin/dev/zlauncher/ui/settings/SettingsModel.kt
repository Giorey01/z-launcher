package dev.zlauncher.ui.settings

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.viewModelScope
import dev.zlauncher.core.nativecore.AssetCategory
import dev.zlauncher.core.nativecore.CocoonDataDir
import dev.zlauncher.core.nativecore.CoreException
import dev.zlauncher.core.nativecore.InstalledPack
import dev.zlauncher.core.nativecore.InstalledTheme
import dev.zlauncher.core.nativecore.LibraryMode
import dev.zlauncher.core.nativecore.PlatformEmulatorSetting
import dev.zlauncher.core.nativecore.RomRoot
import dev.zlauncher.launch.toMessage
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.UiMessage
import kotlinx.coroutines.launch

/** The categories of the left rail matching Cocoon. */
enum class Category {
    /** Look: theme, accent, cursor, surface, sounds. */
    PERSONALIZATION,

    /** Folders, emulators, media. */
    LIBRARY,

    /** Accounts, RetroAchievements, Discord. */
    INTEGRATIONS,

    /** Screens and setup. */
    SYSTEM,

    /** Version and credits. */
    ABOUT,
}

/** A list of rows: the root of a category or a subpage. */
sealed interface Page {
    /** The root list of the selected category. */
    data object Root : Page

    /** The installed themes and the import actions. */
    data object Themes : Page

    /** Whether to apply the settings of a theme too. */
    data class ApplyTheme(
        /** Theme id, `null` for the built-in theme. */
        val themeId: String?,
        /** Theme name. */
        val name: String,
    ) : Page

    /** The layers that can have their own pack. */
    data object PackLayers : Page

    /** The packs of one layer. */
    data class Packs(
        /** The layer. */
        val category: AssetCategory,
    ) : Page

    /** The ROM folders. */
    data object Roots : Page

    /** The default emulator of each platform. */
    data object Emulators : Page

    /** Glass options (blur, refraction, rim highlight, etc.). */
    data object GlassOptions : Page

    /** Background wallpaper options. */
    data object Wallpapers : Page

    /** Home grid paging and row layout. */
    data object Grid : Page

    /** Tile roundness, scale, and hover scale. */
    data object AppIconStyle : Page

    /** Hero background display, gradients, and logo scales. */
    data object HeroSettings : Page

    /** Dock icon shape and styling. */
    data object DockStyle : Page

    /** UI sound effects and volumes. */
    data object SoundSettings : Page

    /** Background music and jingles. */
    data object JinglesSettings : Page

    /** UI animation speed. */
    data object Animation : Page

    /** Now playing dimming and saturation. */
    data object NowPlaying : Page
}

/** What the lists read from the core; loaded when a page opens. */
data class SettingsData(
    /** Installed themes. */
    val themes: List<InstalledTheme> = emptyList(),
    /** Installed packs of all layers. */
    val packs: List<InstalledPack> = emptyList(),
    /** Cocoon data folders found on the device. */
    val cocoon: List<CocoonDataDir> = emptyList(),
    /** ROM folders. */
    val roots: List<RomRoot> = emptyList(),
    /** Platforms with their emulator candidates. */
    val emulators: List<PlatformEmulatorSetting> = emptyList(),
    /** The library mode. */
    val libraryMode: LibraryMode = LibraryMode.SMART,
    /** Whether a SteamGridDB key is stored. */
    val hasSgdbKey: Boolean = false,
)

/**
 * The state of the settings screens: which category and page, which row has the cursor, the open dropdown, the
 * data read from the core and the last result message. It is driven by [SettingsKeys] so the same state can be
 * drawn on the big display while the buttons come from the other one.
 */
class SettingsModel(private val app: AppViewModel) {
    /** The selected category. */
    var category by mutableStateOf(Category.PERSONALIZATION)
        private set

    /** Pages above the root of the category. */
    val pages = mutableStateListOf<Page>()

    /** The row with the cursor on the current page. */
    var row by mutableIntStateOf(0)

    /** The cursor is on the category rail, not in the list. */
    var inRail by mutableStateOf(false)

    /** Index of the open dropdown's highlighted option, `-1` when no dropdown is open. */
    var dropdown by mutableIntStateOf(-1)

    /** Data from the core. */
    var data by mutableStateOf(SettingsData())
        private set

    /** The last result (import counts, scan result), shown as the subtitle of the row that produced it. */
    var result by mutableStateOf<UiMessage?>(null)

    /** Id of the row that produced [result]. */
    var resultRow by mutableStateOf<String?>(null)

    /** A long operation is running. */
    var busy by mutableStateOf(false)

    /** Id of the row whose long operation is running, `null` when none. */
    var resultRowRunning by mutableStateOf<String?>(null)
        private set

    /** Set to ask the screen to open the ZIP picker. */
    var wantZip by mutableStateOf(false)

    /** Set to ask the screen to open the ROM folder picker. */
    var wantFolder by mutableStateOf(false)

    /** Set while the SteamGridDB key editor is open. */
    var wantSgdbKey by mutableStateOf(false)

    /** The art download running now, `null` when none. */
    var artDownload by mutableStateOf<ArtDownload?>(null)

    /** Cursor row remembered per page, to come back to the same row. */
    private val rowMemory = HashMap<Any, Int>()

    /** The page being shown. */
    val page: Page get() = pages.lastOrNull() ?: Page.Root

    /** Opens the settings at the first category. */
    fun reset() {
        category = Category.PERSONALIZATION
        pages.clear()
        row = 0
        inRail = false
        dropdown = -1
        result = null
        resultRow = null
        reload()
    }

    /** Selects a category (the rail) and shows its root list. */
    fun select(next: Category) {
        category = next
        pages.clear()
        row = 0
        dropdown = -1
        result = null
    }

    /** Steps through the categories by [delta], wrapping around. */
    fun step(delta: Int) {
        val all = Category.entries
        select(all[(category.ordinal + delta).mod(all.size)])
    }

    /** Opens a subpage. */
    fun push(next: Page) {
        rowMemory[pageKey()] = row
        pages.add(next)
        row = 0
        reload()
    }

    /** Goes back one page; `false` when the root is showing (the caller leaves the settings). */
    fun pop(): Boolean {
        if (dropdown >= 0) {
            dropdown = -1
            return true
        }
        val popped = pages.isNotEmpty()
        if (popped) {
            pages.removeAt(pages.lastIndex)
            row = rowMemory[pageKey()] ?: 0
            reload()
        }
        return popped
    }

    /** Key of the current page in [rowMemory]. */
    private fun pageKey(): Any = category to page

    /** Reads the data the lists need from the core again. */
    fun reload() {
        app.viewModelScope.launch {
            try {
                val packages = app.zl.apps.list().map { it.packageName }
                data = app.zl.core.call { core ->
                    SettingsData(
                        themes = core.installedThemes(),
                        packs = core.installedPacks(),
                        cocoon = core.detectCocoonDataDirs(),
                        roots = core.roots(),
                        emulators = core.platformEmulators(packages),
                        libraryMode = core.libraryMode(),
                        hasSgdbKey = core.hasSteamgriddbKey(),
                    )
                }
            } catch (e: CoreException) {
                app.message = e.toMessage()
            }
        }
    }

    /** Runs a change in the core, then reloads the theme and the lists. [block] runs off the main thread. */
    fun change(block: (dev.zlauncher.core.nativecore.Core) -> Unit) {
        app.viewModelScope.launch {
            try {
                app.zl.core.call(block)
                app.reloadTheme()
                reload()
            } catch (e: CoreException) {
                app.message = e.toMessage()
            }
        }
    }

    /** Imports the theme ZIP the user picked: copies it to the cache (the core reads paths) and reports the counts. */
    fun importZip(uri: android.net.Uri) {
        work("zip") {
            val file = java.io.File(app.zl.cacheDir, "import.zip")
            app.zl.contentResolver.openInputStream(uri)?.use { input -> file.outputStream().use { input.copyTo(it) } }
            val report = app.zl.core.call { it.importThemeZip(file.absolutePath) }
            file.delete()
            UiMessage(dev.zlauncher.R.string.st_import_done, listOf(report.themesAdded.size.toString(), report.packsAdded.size.toString()))
        }
    }

    /** Adds the ROM folder the user picked (a tree URI) and keeps the grant so emulators can open the games. */
    fun addTree(treeUri: String) {
        change { core -> dev.zlauncher.core.nativecore.pathOfSafTree(treeUri)?.let { core.addRoot(it, treeUri) } }
    }

    /** Builds the rows of the current page (also called by the key handler, outside any composition). */
    fun rows(theme: dev.zlauncher.ui.theme.ZTheme): List<SettingRow> = SettingsRows(app, this, theme, app.zl).build()

    /** Runs a longer job with the busy flag on; [job] returns the message to show when it is done. */
    fun work(rowId: String, job: suspend () -> UiMessage?) {
        app.viewModelScope.launch {
            busy = true
            resultRowRunning = rowId
            try {
                result = job()
                resultRow = rowId
                app.reloadTheme()
                reload()
            } catch (e: CoreException) {
                app.message = e.toMessage()
            } finally {
                busy = false
                resultRowRunning = null
            }
        }
    }
}
