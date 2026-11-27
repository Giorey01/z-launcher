package dev.zlauncher.pods

import dev.zlauncher.core.nativecore.GameDetails
import dev.zlauncher.core.nativecore.GameEntry
import dev.zlauncher.core.nativecore.PlatformEntry
import dev.zlauncher.core.nativecore.PlayStats
import dev.zlauncher.core.nativecore.SoundAction
import dev.zlauncher.ui.theme.ZTheme
import java.io.File
import kotlinx.coroutines.flow.SharedFlow

/** Thrown when a Pod invokes a host service whose permission was not declared in manifest.json (§8). */
class PermissionNotDeclaredException(
    /** The permission name required (e.g. `library.read`, `launch`). */
    val permission: String,
) : SecurityException("Pod requested service requiring permission '$permission', but permission was not declared in manifest.json")

/** Access to isolated string resources from the Pod's `i18n/<lang>.json` bundle (§8). */
interface PodStrings {
    /** Looks up [key], falling back to English then to [key] itself. */
    fun get(key: String): String

    /** Looks up [key] and formats with [args]. */
    fun get(key: String, vararg args: Any): String = get(key)
}

/** Isolated file directories and key-value preferences for a specific Pod (§8). */
interface PodStorage {
    /** Private application files directory dedicated to this Pod. */
    val filesDir: File

    /** Private cache directory dedicated to this Pod. */
    val cacheDir: File

    /** Reads a string setting. */
    fun getString(key: String, default: String? = null): String?

    /** Writes a string setting. */
    fun setString(key: String, value: String?)

    /** Reads a boolean setting. */
    fun getBoolean(key: String, default: Boolean = false): Boolean

    /** Writes a boolean setting. */
    fun setBoolean(key: String, value: Boolean)

    /** Reads an integer setting. */
    fun getInt(key: String, default: Int = 0): Int

    /** Writes an integer setting. */
    fun setInt(key: String, value: Int)
}

/**
 * Host environment and services provided to a Pod (§8).
 * Services requiring undeclared permissions throw [PermissionNotDeclaredException].
 */
@Suppress("TooManyFunctions")
interface PodHost {
    /** Active visual theme (colours, typography, settings). Always available. */
    val theme: ZTheme

    /** Plays an interface sound effect using the launcher's audio manager. Always available. */
    fun play(action: SoundAction)

    /** Isolated localization strings for this Pod. Always available. */
    val strings: PodStrings

    /** Isolated storage and preferences for this Pod. Always available. */
    val storage: PodStorage

    /**
     * Shows [text] as a notice of the launcher (the line at the bottom of the screen, gone after a few seconds) and
     * plays the theme's sound of [sound] (`null` for silence; the theme may have none for it). Works on any screen and
     * when the launcher comes back from a game, for example on [PodEvent.GameEnded]. The text is cut at
     * `MAX_TOAST_CHARS`; a newer notice replaces the one on screen. Always available (API 3).
     */
    fun toast(text: String, sound: SoundAction? = SoundAction.NOTIFICATION)

    /** Closes the active Pod screen, returning to the previous screen. Always available. */
    fun close()

    /** Navigates to a new destination route within this Pod. Always available. */
    fun navigate(route: PodRoute)

    /** Stream of host events dispatched to this Pod. Always available. */
    val events: SharedFlow<PodEvent>

    // --- Services requiring declared permissions (§8) ---

    /** Lists home folders and platforms. Requires permission `library.read`. */
    suspend fun folders(): List<PlatformEntry>

    /** Lists games in a folder (`null` for all). Requires permission `library.read`. */
    suspend fun games(folder: String?): List<GameEntry>

    /** Searches games matching [query] with typo-tolerant fuzzy ranking. Requires permission `library.read`. */
    suspend fun search(query: String, limit: Int = 50): List<GameEntry>

    /** Fetches details for a specific game. Requires permission `library.read`. */
    suspend fun details(gameId: Long): GameDetails

    /** Fetches play stats for a specific game. Requires permission `library.read`. */
    suspend fun playStats(gameId: Long): PlayStats

    /** Requests rescanning of a platform library. Requires permission `library.rescan`. */
    suspend fun rescan(platformId: String)

    /** Launches a game using user-configured emulators and settings. Requires permission `launch`. */
    fun launch(game: GameEntry)

    /**
     * A secret of this Pod (an API key, a token), decrypted, or `null`. Kept encrypted with a key of the
     * Android Keystore, apart from every other Pod's. Requires permission `secrets`.
     */
    suspend fun getSecret(key: String): String?

    /** Stores [value] encrypted under [key] (`null` removes it); throws when the key store fails. Requires permission `secrets`. */
    suspend fun setSecret(key: String, value: String?)

    /**
     * GETs [url] with [headers]; the body. Retries a busy server (429, 5xx) with backoff. Throws
     * [dev.zlauncher.core.nativecore.TransferException] (`Status`, `Network`…). Requires permission `http`.
     */
    suspend fun httpGet(url: String, headers: Map<String, String> = emptyMap()): ByteArray

    /**
     * POSTs [body] as [contentType] to [url]; the body of the answer. Only a refusal the server did
     * not act on (429, 5xx) is retried. Throws [dev.zlauncher.core.nativecore.TransferException].
     * Requires permission `http`.
     */
    suspend fun httpPost(url: String, body: ByteArray, contentType: String, headers: Map<String, String> = emptyMap()): ByteArray

    /**
     * Downloads [url] into the file [dest], telling [progress] the bytes so far and the total (null when
     * the server does not say); the size of the file. It writes `dest.part` and renames it at the end: a
     * download stopped (the coroutine cancelled) or failed leaves the part, and the next call for the same
     * [dest] resumes it when the server can. There is no limit on the whole download, only on silence.
     * Requires permission `http` (API 2).
     */
    suspend fun download(
        url: String,
        dest: File,
        headers: Map<String, String> = emptyMap(),
        progress: (done: Long, total: Long?) -> Unit = { _, _ -> },
    ): Long

    /**
     * Extracts [archive] (.zip, .7z, or the first volume of a .rar set, told by the extension) into the
     * folder [dest], telling [progress] the bytes written and the total; the files written. Every entry
     * stays inside [dest] (one that would leave it fails the extraction), and on any failure, cancelling
     * the coroutine included, what was written is removed. Throws
     * [dev.zlauncher.core.nativecore.TransferException] (`Unsupported`, `Unsafe`, `Password`, `Damaged`,
     * `NoSpace`…). Requires permission `archive` (API 2).
     */
    suspend fun extract(
        archive: File,
        dest: File,
        password: String? = null,
        progress: (done: Long, total: Long?) -> Unit = { _, _ -> },
    ): List<File>

    /**
     * The folder of a platform in the library (absolute path), where a game of that platform
     * belongs; `null` when the library has none. Requires permission `folders`.
     */
    suspend fun platformRoot(platformId: String): String?

    /** Long tasks kept running by the launcher with a notification. Requires permission `work`. */
    val work: PodWork
}
