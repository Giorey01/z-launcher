package dev.zlauncher.core

import dev.zlauncher.core.nativecore.Core
import java.io.File
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

/** Opens the Rust core once, in the app's private folder, and runs calls on the IO dispatcher. */
class CoreProvider(private val dataDir: File) {
    /** The core; the database is opened on the first use, never on the main thread. */
    private val core: Core by lazy { Core(dataDir.absolutePath) }

    /** Runs [block] with the core on [Dispatchers.IO]; core errors propagate as `CoreException`. */
    suspend fun <T> call(block: (Core) -> T): T = withContext(Dispatchers.IO) { block(core) }
}
