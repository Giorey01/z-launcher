package dev.zlauncher.pods

import dev.zlauncher.core.nativecore.TransferException
import dev.zlauncher.core.nativecore.TransferListener
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.job
import kotlinx.coroutines.withContext
import java.io.File
import dev.zlauncher.core.nativecore.downloadFile as coreDownload
import dev.zlauncher.core.nativecore.extractArchive as coreExtract
import dev.zlauncher.core.nativecore.httpGet as coreGet
import dev.zlauncher.core.nativecore.httpPost as corePost

/**
 * The transfers [PodHost] offers (§8, API 2): HTTP requests, downloads and archive extraction, run by
 * the core off the main thread. Cancelling the calling coroutine stops a download or an extraction.
 */
internal object HostTransfers {
    /** GET with retries; the body. */
    suspend fun get(url: String, headers: Map<String, String>): ByteArray = withContext(Dispatchers.IO) { coreGet(url, headers) }

    /** POST; the body of the answer. */
    suspend fun post(url: String, body: ByteArray, contentType: String, headers: Map<String, String>): ByteArray =
        withContext(Dispatchers.IO) { corePost(url, body, contentType, headers) }

    /** Download into [dest] (resumed from `dest.part`); the size of the file. */
    suspend fun download(url: String, dest: File, headers: Map<String, String>, progress: (Long, Long?) -> Unit): Long =
        running(progress) { coreDownload(url, dest.path, headers, it).toLong() }

    /** Extraction of [archive] into [dest]; the files written. */
    suspend fun extract(archive: File, dest: File, password: String?, progress: (Long, Long?) -> Unit): List<File> =
        running(progress) { l -> coreExtract(archive.path, dest.path, password, l).map(::File) }

    /**
     * Runs [block] on the IO dispatcher with a listener bound to this coroutine: progress goes to
     * [progress], cancellation stops the core's work and surfaces as the coroutine's cancellation.
     */
    private suspend fun <T> running(progress: (Long, Long?) -> Unit, block: (TransferListener) -> T): T = withContext(Dispatchers.IO) {
        val listener = CoroutineListener(coroutineContext.job, progress)
        try {
            block(listener)
        } catch (e: TransferException.Cancelled) {
            ensureActive()
            throw e
        }
    }
}

/** A [TransferListener] tied to a coroutine's [job]: stopped once the job is no longer active. */
internal class CoroutineListener(
    /** The coroutine the work belongs to. */
    private val job: Job,
    /** Told the bytes done and the total. */
    private val progress: (Long, Long?) -> Unit,
) : TransferListener {
    override fun onProgress(done: ULong, total: ULong?) = progress(done.toLong(), total?.toLong())

    override fun isCancelled(): Boolean = !job.isActive
}
