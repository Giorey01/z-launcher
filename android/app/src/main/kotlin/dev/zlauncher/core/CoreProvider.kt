package dev.zlauncher.core

import dev.zlauncher.core.nativecore.Core
import dev.zlauncher.core.nativecore.SecretStore
import java.io.File
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

/**
 * Opens the Rust core once, in the app's private folder, and runs calls on the IO dispatcher. The API keys go to
 * [secrets] (the Keystore), which the core gets before any other call; keys an older version kept in the database
 * move there then.
 */
class CoreProvider(private val dataDir: File, private val secrets: SecretStore) {
    /** The core; the database is opened on the first use, never on the main thread. */
    private val core: Core by lazy { Core(dataDir.absolutePath).also { it.setSecretStore(secrets) } }

    /** Runs [block] with the core on [Dispatchers.IO]; core errors propagate as `CoreException`. */
    suspend fun <T> call(block: (Core) -> T): T = withContext(Dispatchers.IO) { block(core) }
}
