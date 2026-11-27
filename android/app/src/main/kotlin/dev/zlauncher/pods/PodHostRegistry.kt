package dev.zlauncher.pods

import dev.zlauncher.core.nativecore.GameEntry

/**
 * The hosts of the Pod screens and tabs that are on screen. They get the launcher's events (§8) while they are here:
 * a game ended (counted when the launcher comes back from it) reaches a Pod that has a screen or a tab open.
 */
class PodHostRegistry {
    /** The hosts on screen; touched from the main thread only. */
    private val hosts = mutableSetOf<AppPodHost>()

    /** [host] is on screen: it gets the events until [remove]. */
    fun add(host: AppPodHost) {
        hosts += host
    }

    /** [host] left the screen. */
    fun remove(host: AppPodHost) {
        hosts -= host
    }

    /** How many hosts are on screen. */
    fun size(): Int = hosts.size

    /**
     * Tells the hosts on screen that the game [gameId] ended after [seconds]. [find] looks the game up and is only called
     * when somebody is listening; a game it cannot find is not announced.
     */
    suspend fun announceGameEnded(gameId: Long, seconds: Long, find: suspend (Long) -> GameEntry?) {
        if (hosts.isEmpty()) return
        val game = find(gameId) ?: return
        for (host in hosts.toList()) host.dispatchEvent(PodEvent.GameEnded(game, seconds))
    }
}
