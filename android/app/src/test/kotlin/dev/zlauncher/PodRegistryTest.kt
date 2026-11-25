package dev.zlauncher

import android.content.Context
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Star
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.vector.ImageVector
import dev.zlauncher.core.nativecore.InstalledPodView
import dev.zlauncher.pods.LeadTileSlotItem
import dev.zlauncher.pods.MAX_LEAD_TILES
import dev.zlauncher.pods.MAX_SLOTS_PER_POD
import dev.zlauncher.pods.Pod
import dev.zlauncher.pods.PodHost
import dev.zlauncher.pods.PodLoader
import dev.zlauncher.pods.PodProblem
import dev.zlauncher.pods.PodProvider
import dev.zlauncher.pods.PodRegistry
import dev.zlauncher.pods.PodRoute
import dev.zlauncher.pods.PodSlot
import dev.zlauncher.pods.PodWithMetadata
import dev.zlauncher.pods.StartMenuSlotItem
import java.io.File
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/** A Pod that only has an id and optional slots (the registry never draws it). */
class FakePod(
    override val id: String,
    private val declaredSlotsList: List<PodSlot> = emptyList(),
) : Pod {
    /** Not used by the registry. */
    override val icon: ImageVector get() = Icons.Rounded.Star

    /** Not used by the registry. */
    override fun title(context: Context): String = id

    /** Returns test slots. */
    override fun slots(): List<PodSlot> = declaredSlotsList

    /** Not drawn in these tests. */
    @Composable
    override fun Content(host: PodHost) = Unit
}

/** A Pod whose slots() call crashes. */
class CrashingSlotsPod(override val id: String) : Pod {
    override val icon: ImageVector get() = Icons.Rounded.Star
    override fun title(context: Context): String = id
    override fun slots(): List<PodSlot> = error("slots exploded")
    @Composable
    override fun Content(host: PodHost) = Unit
}

/** A provider listed in the test `META-INF/services`, as a build that adds Pods would list its own. */
class TestPods : PodProvider {
    /** Two Pods with a good id, one with a bad id, one repeating an id. */
    override fun pods(): List<Pod> = listOf(FakePod("stats"), FakePod("Bad Id"), FakePod("vault"), FakePod("stats"))
}

/** A provider that fails: the others still load. */
class BrokenPods : PodProvider {
    /** Always fails. */
    override fun pods(): List<Pod> = error("broken")
}

/** The contract of the Pod registry (RF-73, v2). */
class PodRegistryTest {
    /** Providers are found through `META-INF/services`; bad and repeated ids and failing providers are left out. */
    @Test
    fun providersAreFoundAndCheckedThroughServiceLoader() {
        val registry = PodRegistry.load(javaClass.classLoader!!)
        assertEquals(listOf("stats", "vault"), registry.pods.map { it.id })
        assertEquals(
            listOf(
                PodProblem.PROVIDER_FAILED to BrokenPods::class.java.name,
                PodProblem.BAD_ID to "Bad Id",
                PodProblem.DUPLICATE_ID to "stats",
            ).sortedBy { it.first },
            registry.problems.sortedBy { it.first },
        )
        assertEquals("vault", registry.find("vault")?.id)
        assertNull(registry.find("missing"))
    }

    /** With no provider the open-source build has no Pod and no problem. */
    @Test
    fun noProvidersMeansNoPods() {
        val registry = PodRegistry.from(emptyList())
        assertEquals(emptyList<Pod>(), registry.pods)
        assertEquals(emptyList<Pair<PodProblem, String>>(), registry.problems)
    }

    /** Slot declarations exceeding MAX_SLOTS_PER_POD (16) are truncated (§7). */
    @Test
    fun slotLimitEnforced() {
        val slots = (1..25).map { PodSlot.SettingsSection("Section $it") }
        val pod = FakePod("overflow", slots)
        val entry = PodWithMetadata(pod, emptySet(), emptySet(), isExternal = false)
        val registry = PodRegistry.from(listOf(entry))
        val evaluated = registry.validSlots(entry)
        assertEquals(MAX_SLOTS_PER_POD, evaluated.size)
    }

    /** For external pods, undeclared slot types are ignored (§7). */
    @Test
    fun manifestSlotFilteringForExternalPods() {
        val slots = listOf(
            PodSlot.StartMenu("Menu", Icons.Rounded.Star),
            PodSlot.SettingsSection("Settings"),
        )
        val pod = FakePod("ext", slots)
        // Manifest only declared "start_menu"
        val entry = PodWithMetadata(pod, declaredSlots = setOf("start_menu"), permissions = emptySet(), isExternal = true)
        val registry = PodRegistry.from(listOf(entry))
        val evaluated = registry.validSlots(entry)
        assertEquals(1, evaluated.size)
        assertTrue(evaluated.first() is PodSlot.StartMenu)
    }

    /** An exception in slots() records a crash and returns no slots (§10). */
    @Test
    fun failingSlotsInterceptedAsCrash() {
        var crashedId: String? = null
        var crashReason: String? = null
        val pod = CrashingSlotsPod("crasher")
        val entry = PodWithMetadata(pod, emptySet(), emptySet(), isExternal = false)
        val registry = PodRegistry.from(listOf(entry), onCrash = { id, reason ->
            crashedId = id
            crashReason = reason
        })
        val evaluated = registry.validSlots(entry)
        assertEquals(emptyList<PodSlot>(), evaluated)
        assertEquals("crasher", crashedId)
        assertTrue(crashReason?.contains("slots exploded") == true)
    }

    /** Platform lead tiles are filtered by platform and capped at MAX_LEAD_TILES (3) (§7). */
    @Test
    fun platformGridLeadCappedAtThree() {
        val slots1 = listOf(
            PodSlot.PlatformGridLead(platforms = setOf("snes", "gba"), label = "Action 1", icon = Icons.Rounded.Star),
            PodSlot.PlatformGridLead(platforms = null, label = "Action 2", icon = Icons.Rounded.Star),
        )
        val slots2 = listOf(
            PodSlot.PlatformGridLead(platforms = setOf("snes"), label = "Action 3", icon = Icons.Rounded.Star),
            PodSlot.PlatformGridLead(platforms = setOf("snes"), label = "Action 4", icon = Icons.Rounded.Star),
        )
        val entry1 = PodWithMetadata(FakePod("p1", slots1), emptySet(), emptySet())
        val entry2 = PodWithMetadata(FakePod("p2", slots2), emptySet(), emptySet())
        val registry = PodRegistry.from(listOf(entry1, entry2))

        val snesLeads = registry.platformGridLeadSlots("snes")
        assertEquals(MAX_LEAD_TILES, snesLeads.size)
        assertEquals(listOf("Action 1", "Action 2", "Action 3"), snesLeads.map { it.label })

        val psxLeads = registry.platformGridLeadSlots("psx")
        assertEquals(1, psxLeads.size)
        assertEquals("Action 2", psxLeads.first().label)
    }

    /** Sentinel file loading triggers safe mode and reports crash (§10). */
    @Test
    fun safeModeSentinelFileTriggered() {
        val tempDir = File.createTempFile("pod_test", "dir").apply { delete(); mkdirs() }
        try {
            val sentinel = File(tempDir, "loading").apply { writeText("bad_dex_pod") }
            var crashedId: String? = null
            var crashReason: String? = null

            val loader = PodLoader(
                context = null,
                podsDir = tempDir,
                sentinelFile = sentinel,
            )
            val result = loader.loadExternalPods(emptyList()) { id, reason ->
                crashedId = id
                crashReason = reason
            }
            assertTrue(result.safeModeTriggered)
            assertEquals("bad_dex_pod", result.crashedPodId)
            assertEquals("bad_dex_pod", crashedId)
            assertTrue(!sentinel.exists())
        } finally {
            tempDir.deleteRecursively()
        }
    }
}
