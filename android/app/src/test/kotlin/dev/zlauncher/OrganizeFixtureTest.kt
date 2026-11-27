package dev.zlauncher

import dev.zlauncher.core.nativecore.Core
import dev.zlauncher.core.nativecore.CoreException
import dev.zlauncher.core.nativecore.InstalledApp
import dev.zlauncher.core.nativecore.SortOrder
import dev.zlauncher.nav.manualFolderId
import java.io.File
import java.nio.file.Files
import org.json.JSONArray
import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertThrows
import org.junit.Test

/**
 * The organisation of the library through the real Rust library: the Apps and Emulators folders read from the
 * fixture `fixtures/organization/app_folders.json` (the Rust test of `zl-core` reads the same file), and the manual
 * folders and sort orders the screens call.
 */
class OrganizeFixtureTest {
    /** The fixture document. */
    private val doc = JSONObject(File(checkNotNull(System.getProperty("zl.fixturesDir")), "organization/app_folders.json").readText())

    /** A core over a throwaway data folder. */
    private fun newCore(): Core = Core(Files.createTempDirectory("zl-organize-test").toString())

    /** The strings of a JSON array. */
    private fun strings(array: JSONArray): List<String> = List(array.length()) { array.getString(it) }

    /** The installed apps of the fixture. */
    private fun installed(): List<InstalledApp> = doc.getJSONArray("installed").let { array ->
        List(array.length()) {
            val app = array.getJSONObject(it)
            InstalledApp(app.getString("package"), app.getBoolean("is_game"))
        }
    }

    /** Apps and Emulators hold what the fixture says. */
    @Test
    fun appFoldersFollowTheFixture() {
        val core = newCore()
        assertEquals(strings(doc.getJSONArray("apps")), core.appFolderPackages("@apps", installed()))
        assertEquals(strings(doc.getJSONArray("emulators")), core.appFolderPackages("@emulators", installed()))
        assertEquals(listOf(2u, 1u), core.appFolderEntries(installed()).map { it.games })
        assertThrows(CoreException.InvalidInput::class.java) { core.appFolderPackages("@favorites", installed()) }
    }

    /** A manual folder is made, renamed and deleted, and names are unique. */
    @Test
    fun manualFoldersAreMadeRenamedAndDeleted() {
        val core = newCore()
        val made = core.createCollection("  Co-op ")
        assertEquals("Co-op", made.name)
        assertEquals(made.id, manualFolderId(made.folderId))
        assertThrows(CoreException.InvalidInput::class.java) { core.createCollection("CO-OP") }
        core.renameCollection(made.id, "Family")
        assertEquals(listOf("Family"), core.collections().map { it.name })
        assertEquals("an empty manual folder shows on the Home", listOf("Family"), core.platforms().map { it.name })
        core.deleteCollection(made.id)
        assertEquals(emptyList<String>(), core.collections().map { it.name })
        assertThrows(CoreException.NotFound::class.java) { core.deleteCollection(made.id) }
    }

    /** The order of a folder: the global choice, the folder's own, and no manual order outside a manual folder. */
    @Test
    fun sortOrdersFollowTheFolderThenTheGlobalChoice() {
        val core = newCore()
        assertEquals(SortOrder.NAME, core.sortOrder("gba"))
        core.setSortOrder(null, SortOrder.RATING)
        assertEquals(SortOrder.RATING, core.sortOrder("gba"))
        core.setSortOrder("gba", SortOrder.ADDED)
        assertEquals(SortOrder.ADDED, core.sortOrder("gba"))
        core.setSortOrder("gba", null)
        assertNull(core.sortOverride("gba"))
        assertThrows(CoreException.InvalidInput::class.java) { core.setSortOrder("gba", SortOrder.MANUAL) }
        assertEquals(SortOrder.MANUAL, core.sortOrder(core.createCollection("A").folderId))
    }

    /** Only the ids of manual folders map to a row. */
    @Test
    fun onlyManualFolderIdsHaveARow() {
        assertEquals(7L, manualFolderId("@folder:7"))
        assertNull(manualFolderId("@favorites"))
        assertNull(manualFolderId("gba"))
        assertNull(manualFolderId("@folder:x"))
        assertNull(manualFolderId(null))
    }
}
