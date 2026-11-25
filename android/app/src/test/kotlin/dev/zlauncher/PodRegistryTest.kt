package dev.zlauncher

import android.content.Context
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.vector.ImageVector
import dev.zlauncher.pods.Pod
import dev.zlauncher.pods.PodHost
import dev.zlauncher.pods.PodProblem
import dev.zlauncher.pods.PodProvider
import dev.zlauncher.pods.PodRegistry
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

/** A Pod that only has an id (the registry never draws it). */
class FakePod(override val id: String) : Pod {
    /** Not used by the registry. */
    override val icon: ImageVector get() = throw UnsupportedOperationException()

    /** Not used by the registry. */
    override fun title(context: Context): String = id

    /** Not drawn in these tests. */
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

/** The contract of the Pod registry (RF-73). */
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
}
