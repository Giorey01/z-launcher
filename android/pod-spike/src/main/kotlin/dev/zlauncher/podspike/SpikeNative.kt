package dev.zlauncher.podspike

object SpikeNative {
    private var loaded = false

    fun load(libPath: String? = null) {
        if (loaded) return
        if (libPath != null) {
            try {
                System.load(libPath)
                loaded = true
                return
            } catch (_: Throwable) {
                // fallback to System.loadLibrary
            }
        }
        System.loadLibrary("spike_native")
        loaded = true
    }

    @JvmStatic
    external fun getMagicNumber(): Int
}
