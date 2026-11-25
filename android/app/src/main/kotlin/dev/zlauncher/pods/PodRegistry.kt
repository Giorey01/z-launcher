package dev.zlauncher.pods

import android.content.Context
import androidx.compose.ui.graphics.vector.ImageVector
import java.util.ServiceLoader

/** Characters an id may have (see [Pod.id]). */
private val ID_PATTERN = Regex("[a-z0-9._-]+")

/** Failures after which the providers are no longer read (a broken services file could fail on every step). */
private const val MAX_PROVIDER_FAILURES = 16

/** Maximum slot declarations evaluated per Pod (§7). */
const val MAX_SLOTS_PER_POD: Int = 16

/** Maximum lead tiles shown on a platform grid (§7). */
const val MAX_LEAD_TILES: Int = 3

/** Why a Pod was left out. */
enum class PodProblem {
    /** Its id is empty or has other characters than the allowed ones. */
    BAD_ID,

    /** Another Pod already has its id; the first one stays. */
    DUPLICATE_ID,

    /** Its provider failed to load or to list its Pods. */
    PROVIDER_FAILED,
}

/** A resolved Start menu item from a Pod slot. */
data class StartMenuSlotItem(
    /** Originating Pod id. */
    val podId: String,
    /** Item label. */
    val label: String,
    /** Icon displayed in the Start menu. */
    val icon: ImageVector,
    /** Target navigation route. */
    val route: PodRoute,
)

/** A resolved lead tile item for a platform grid slot. */
data class LeadTileSlotItem(
    /** Originating Pod id. */
    val podId: String,
    /** Label on the lead tile. */
    val label: String,
    /** Icon on the lead tile. */
    val icon: ImageVector,
    /** Target navigation route for the platform. */
    val route: PodRoute,
)

/** The Pods of this build, their resolved slots, and what was left out. */
class PodRegistry(
    /** The usable Pods with their metadata. */
    val entries: List<PodWithMetadata>,
    /** What was left out, with the id or provider class it was about. */
    val problems: List<Pair<PodProblem, String>>,
    /** Optional callback when a Pod entry point throws a failure (§10). */
    var onCrash: ((podId: String, reason: String) -> Unit)? = null,
) {
    /** The usable Pods, in registration order. */
    val pods: List<Pod> get() = entries.map { it.pod }

    /** The Pod with [id], if there is one. */
    fun find(id: String): Pod? = entries.firstOrNull { it.pod.id == id }?.pod

    /** The metadata for Pod with [id], if there is one. */
    fun findEntry(id: String): PodWithMetadata? = entries.firstOrNull { it.pod.id == id }

    /** Returns the evaluated and filtered slots declared by [entry] (§7, §10). */
    fun validSlots(entry: PodWithMetadata): List<PodSlot> {
        val result = runCatching { entry.pod.slots() }
        if (result.isFailure) {
            onCrash?.invoke(entry.pod.id, "Exception in slots(): ${result.exceptionOrNull()?.message}")
            return emptyList()
        }
        val rawSlots = result.getOrDefault(emptyList()).take(MAX_SLOTS_PER_POD)
        return if (!entry.isExternal) {
            rawSlots
        } else {
            rawSlots.filter { it.manifestSlotName in entry.declaredSlots }
        }
    }

    /** Resolves all entries for the Start menu slot, synthesizing a fallback entry for slotless pods. */
    fun startMenuSlots(context: Context): List<StartMenuSlotItem> {
        val out = mutableListOf<StartMenuSlotItem>()
        for (entry in entries) {
            val slots = validSlots(entry)
            val menuSlots = slots.filterIsInstance<PodSlot.StartMenu>()
            if (menuSlots.isNotEmpty()) {
                for (slot in menuSlots) {
                    out += StartMenuSlotItem(entry.pod.id, slot.label, slot.icon, slot.route)
                }
            } else if (slots.isEmpty()) {
                // v1 backwards-compatibility: pods with no declared slots appear in the Start menu
                out += StartMenuSlotItem(entry.pod.id, entry.pod.title(context), entry.pod.icon, PodRoute("main"))
            }
        }
        return out
    }

    /** Resolves up to [MAX_LEAD_TILES] (3) platform grid lead tiles for [platformId] (§7). */
    fun platformGridLeadSlots(platformId: String): List<LeadTileSlotItem> {
        val out = mutableListOf<LeadTileSlotItem>()
        for (entry in entries) {
            out.addAll(matchingLeadSlots(entry, platformId))
            if (out.size >= MAX_LEAD_TILES) {
                return out.take(MAX_LEAD_TILES)
            }
        }
        return out
    }

    private fun matchingLeadSlots(entry: PodWithMetadata, platformId: String): List<LeadTileSlotItem> {
        val slots = validSlots(entry).filterIsInstance<PodSlot.PlatformGridLead>()
        return slots.filter { slot ->
            val platforms = slot.platforms
            platforms == null || platformId in platforms
        }.map {
            LeadTileSlotItem(entry.pod.id, it.label, it.icon, it.routeForPlatform(platformId))
        }
    }

    /** Builds registries from services files, external pods, or a raw list. */
    companion object {
        /** The Pods the providers listed in `META-INF/services` give plus external pods. */
        fun load(
            loader: ClassLoader,
            externalPods: List<PodWithMetadata> = emptyList(),
            onCrash: ((podId: String, reason: String) -> Unit)? = null,
        ): PodRegistry {
            val problems = mutableListOf<Pair<PodProblem, String>>()
            val builtinPods = providers(loader, problems).flatMap { provider ->
                runCatching { provider.pods() }
                    .onFailure { problems += PodProblem.PROVIDER_FAILED to provider.javaClass.name }
                    .getOrDefault(emptyList())
            }
            val builtinEntries = builtinPods.map { PodWithMetadata(it, emptySet(), emptySet(), isExternal = false) }
            return from(builtinEntries + externalPods, problems, onCrash)
        }

        /** The providers [loader] finds; one that fails to load is noted in [problems] and skipped. */
        private fun providers(loader: ClassLoader, problems: MutableList<Pair<PodProblem, String>>): List<PodProvider> {
            val iterator = ServiceLoader.load(PodProvider::class.java, loader).iterator()
            val out = mutableListOf<PodProvider>()
            var more = true
            while (more && problems.size < MAX_PROVIDER_FAILURES) {
                val step = runCatching { if (iterator.hasNext()) iterator.next() else null }
                step.onFailure { problems += PodProblem.PROVIDER_FAILED to it.toString() }
                val provider = step.getOrNull()
                if (step.isSuccess && provider == null) more = false
                provider?.let(out::add)
            }
            return out
        }

        /** Checks [candidates]: bad ids and repeated ones are left out, the first of a repeated id stays. */
        fun from(
            candidates: List<PodWithMetadata>,
            earlier: List<Pair<PodProblem, String>> = emptyList(),
            onCrash: ((podId: String, reason: String) -> Unit)? = null,
        ): PodRegistry {
            val problems = earlier.toMutableList()
            val seen = HashSet<String>()
            val entries = candidates.filter { entry ->
                when {
                    !ID_PATTERN.matches(entry.pod.id) -> false.also { problems += PodProblem.BAD_ID to entry.pod.id }
                    !seen.add(entry.pod.id) -> false.also { problems += PodProblem.DUPLICATE_ID to entry.pod.id }
                    else -> true
                }
            }
            return PodRegistry(entries, problems, onCrash)
        }
    }
}
