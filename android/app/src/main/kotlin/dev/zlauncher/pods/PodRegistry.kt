package dev.zlauncher.pods

import java.util.ServiceLoader

/** Characters an id may have (see [Pod.id]). */
private val ID_PATTERN = Regex("[a-z0-9._-]+")

/** Failures after which the providers are no longer read (a broken services file could fail on every step). */
private const val MAX_PROVIDER_FAILURES = 16

/** Why a Pod was left out. */
enum class PodProblem {
    /** Its id is empty or has other characters than the allowed ones. */
    BAD_ID,

    /** Another Pod already has its id; the first one stays. */
    DUPLICATE_ID,

    /** Its provider failed to load or to list its Pods. */
    PROVIDER_FAILED,
}

/** The Pods of this build, and what was left out. */
class PodRegistry(
    /** The usable Pods, in provider order. */
    val pods: List<Pod>,
    /** What was left out, with the id or provider class it was about. */
    val problems: List<Pair<PodProblem, String>>,
) {
    /** The Pod with [id], if there is one. */
    fun find(id: String): Pod? = pods.firstOrNull { it.id == id }

    /** Builds registries from the services files or from a list. */
    companion object {
        /** The Pods the providers listed in `META-INF/services` give, through [loader]. */
        fun load(loader: ClassLoader): PodRegistry {
            val problems = mutableListOf<Pair<PodProblem, String>>()
            val pods = providers(loader, problems).flatMap { provider ->
                runCatching { provider.pods() }
                    .onFailure { problems += PodProblem.PROVIDER_FAILED to provider.javaClass.name }
                    .getOrDefault(emptyList())
            }
            return from(pods, problems)
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
        fun from(candidates: List<Pod>, earlier: List<Pair<PodProblem, String>> = emptyList()): PodRegistry {
            val problems = earlier.toMutableList()
            val seen = HashSet<String>()
            val pods = candidates.filter { pod ->
                when {
                    !ID_PATTERN.matches(pod.id) -> false.also { problems += PodProblem.BAD_ID to pod.id }
                    !seen.add(pod.id) -> false.also { problems += PodProblem.DUPLICATE_ID to pod.id }
                    else -> true
                }
            }
            return PodRegistry(pods, problems)
        }
    }
}
