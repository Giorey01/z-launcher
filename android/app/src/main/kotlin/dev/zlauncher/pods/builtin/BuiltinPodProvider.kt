package dev.zlauncher.pods.builtin

import dev.zlauncher.pods.Pod
import dev.zlauncher.pods.PodProvider

/** Provider of built-in Pods (§7, RF-42, RF-50): Theme Hub and Log. */
class BuiltinPodProvider : PodProvider {
    override fun pods(): List<Pod> = listOf(
        ThemeHubPod(),
        LogPod(),
    )
}
