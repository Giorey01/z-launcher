package dev.zlauncher

import dev.zlauncher.pods.PodRegistry
import dev.zlauncher.pods.PodRoute
import dev.zlauncher.pods.PodSlot
import dev.zlauncher.pods.PodWithMetadata
import dev.zlauncher.pods.builtin.BuiltinPodProvider
import dev.zlauncher.pods.builtin.LogPod
import dev.zlauncher.pods.builtin.ThemeHubPod
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test

/** Tests for BuiltinPodProvider and its v2 Pod implementations (§7). */
class BuiltinPodProviderTest {

    @Test
    fun builtinPodProviderGivesThemeHubAndLogPods() {
        val provider = BuiltinPodProvider()
        val pods = provider.pods()
        assertEquals(2, pods.size)
        assertTrue(pods.any { it is ThemeHubPod })
        assertTrue(pods.any { it is LogPod })
    }

    @Test
    fun themeHubPodDeclaresStartMenuAndSettingsSlots() {
        val themeHub = ThemeHubPod()
        assertEquals("builtin.themehub", themeHub.id)
        val slots = themeHub.slots()
        assertEquals(2, slots.size)
        assertTrue(slots.any { it is PodSlot.StartMenu && it.route == PodRoute("main") })
        assertTrue(slots.any { it is PodSlot.SettingsSection && it.route == PodRoute("main") })
    }

    @Test
    fun logPodDeclaresStartMenuSlot() {
        val logPod = LogPod()
        assertEquals("builtin.log", logPod.id)
        val slots = logPod.slots()
        assertEquals(1, slots.size)
        assertTrue(slots.any { it is PodSlot.StartMenu && it.route == PodRoute("main") })
    }

    @Test
    fun builtinPodsIntegrateIntoRegistryWithoutIssues() {
        val provider = BuiltinPodProvider()
        val entries = provider.pods().map { PodWithMetadata(it, emptySet(), emptySet(), isExternal = false) }
        val registry = PodRegistry.from(entries)
        assertEquals(2, registry.pods.size)
        assertNotNull(registry.find("builtin.themehub"))
        assertNotNull(registry.find("builtin.log"))
        assertEquals(0, registry.problems.size)
    }
}
