package dev.zlauncher.pods

import android.content.Context
import dev.zlauncher.core.KeystoreSecrets
import dev.zlauncher.core.NewArt
import dev.zlauncher.core.nativecore.GameDetails
import dev.zlauncher.core.nativecore.GameEntry
import dev.zlauncher.core.nativecore.PlatformEntry
import dev.zlauncher.core.nativecore.PlayStats
import dev.zlauncher.core.nativecore.ScanEvent
import dev.zlauncher.core.nativecore.ScanListener
import dev.zlauncher.core.nativecore.SoundAction
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.Screen
import dev.zlauncher.pods.work.PodWorkRunner
import dev.zlauncher.ui.theme.ZTheme
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.File

/** A scan listener that discards progress events during background rescan. */
private object QuietScan : ScanListener {
    override fun onEvent(event: ScanEvent) = Unit
}

/** Isolated storage implementation backed by private directories and SharedPreferences. */
class AppPodStorage(context: Context, podId: String) : PodStorage {
    private val safeId = podId.replace(Regex("[^a-zA-Z0-9_-]"), "_")
    override val filesDir: File = File(context.filesDir, "pod_storage/$safeId/files").apply { mkdirs() }
    override val cacheDir: File = File(context.cacheDir, "pod_storage/$safeId/cache").apply { mkdirs() }
    private val prefs = context.getSharedPreferences("pod_$safeId", Context.MODE_PRIVATE)

    override fun getString(key: String, default: String?): String? = prefs.getString(key, default)
    override fun setString(key: String, value: String?) {
        prefs.edit().apply { if (value == null) remove(key) else putString(key, value) }.apply()
    }

    override fun getBoolean(key: String, default: Boolean): Boolean = prefs.getBoolean(key, default)
    override fun setBoolean(key: String, value: Boolean) {
        prefs.edit().putBoolean(key, value).apply()
    }

    override fun getInt(key: String, default: Int): Int = prefs.getInt(key, default)
    override fun setInt(key: String, value: Int) {
        prefs.edit().putInt(key, value).apply()
    }
}

/** In-memory storage for test environments. */
class MemoryPodStorage(
    override val filesDir: File = File(System.getProperty("java.io.tmpdir"), "pod_files"),
    override val cacheDir: File = File(System.getProperty("java.io.tmpdir"), "pod_cache"),
) : PodStorage {
    private val prefs = mutableMapOf<String, Any>()
    override fun getString(key: String, default: String?): String? = prefs[key] as? String ?: default
    override fun setString(key: String, value: String?) {
        if (value == null) prefs.remove(key) else prefs[key] = value
    }

    override fun getBoolean(key: String, default: Boolean): Boolean = prefs[key] as? Boolean ?: default
    override fun setBoolean(key: String, value: Boolean) {
        prefs[key] = value
    }

    override fun getInt(key: String, default: Int): Int = prefs[key] as? Int ?: default
    override fun setInt(key: String, value: Int) {
        prefs[key] = value
    }
}

/** Largest `i18n/<language>.json` read for a Pod: a text table, never this big in practice. */
private const val MAX_I18N_BYTES = 1L shl 20

/**
 * The texts of a Pod (§8): the table of the app's language from `i18n/<language>.json`, then the
 * English one, then the key itself. A file that is missing, too big or not a JSON object is skipped.
 */
class AppPodStrings(
    /** The tables in lookup order (the app's language first, English last). */
    private val tables: List<Map<String, String>> = emptyList(),
) : PodStrings {
    override fun get(key: String): String = tables.firstNotNullOfOrNull { it[key] } ?: key

    override fun get(key: String, vararg args: Any): String {
        val text = get(key)
        return runCatching { text.format(*args) }.getOrDefault(text)
    }

    /** Loading of the tables from a Pod's `i18n/` folder. */
    companion object {
        /** The tables of [language] and of English from [dir]; no folder gives the keys back. */
        fun load(dir: File?, language: String): AppPodStrings {
            if (dir == null) return AppPodStrings()
            val languages = listOf(language.lowercase(), "en").distinct()
            return AppPodStrings(languages.mapNotNull { readTable(File(dir, "$it.json")) })
        }

        /** One `key → text` object; null when the file is missing, too big or not an object of strings. */
        @Suppress("SwallowedException")
        private fun readTable(file: File): Map<String, String>? {
            if (!file.isFile || file.length() > MAX_I18N_BYTES) return null
            return try {
                val json = org.json.JSONObject(file.readText())
                json.keys().asSequence().mapNotNull { k -> (json.opt(k) as? String)?.let { k to it } }.toMap()
            } catch (e: org.json.JSONException) {
                null
            }
        }
    }
}

/** The launcher's side of [PodHost], with runtime permission enforcement (§8). */
@Suppress("TooManyFunctions")
class AppPodHost(
    private val podId: String,
    private val permissions: Set<String>,
    private val app: AppViewModel?,
    override val theme: ZTheme,
    override val strings: PodStrings = AppPodStrings(),
    override val storage: PodStorage = MemoryPodStorage(),
) : PodHost {
    /** Convenience constructor for production use with [AppViewModel]. */
    constructor(
        podId: String,
        permissions: Set<String>,
        app: AppViewModel,
        theme: ZTheme,
        i18nDir: File? = null,
    ) : this(podId, permissions, app, theme, AppPodStrings.load(i18nDir, java.util.Locale.getDefault().language), AppPodStorage(app.zl, podId))

    private val _events = MutableSharedFlow<PodEvent>(extraBufferCapacity = 64)
    override val events: SharedFlow<PodEvent> = _events.asSharedFlow()

    private fun requirePerm(permission: String) {
        if (!permissions.contains(permission)) {
            throw PermissionNotDeclaredException(permission)
        }
    }

    /** Emits a host event to this Pod if it has the required permission (§8). */
    fun dispatchEvent(event: PodEvent) {
        val needsLibrary = event is PodEvent.GameStarted || event is PodEvent.GameEnded || event is PodEvent.LibraryChanged
        if (needsLibrary && !permissions.contains("library.read")) return
        _events.tryEmit(event)
    }

    override fun play(action: SoundAction) {
        app?.play(action)
    }

    override fun close() {
        app?.back()
    }

    override fun navigate(route: PodRoute) {
        app?.open(Screen.Pod(podId, route))
    }

    override suspend fun folders(): List<PlatformEntry> {
        requirePerm("library.read")
        return app?.zl?.core?.call { it.platforms() } ?: emptyList()
    }

    override suspend fun games(folder: String?): List<GameEntry> {
        requirePerm("library.read")
        return app?.zl?.core?.call { it.games(folder) } ?: emptyList()
    }

    override suspend fun search(query: String, limit: Int): List<GameEntry> {
        requirePerm("library.read")
        return app?.zl?.core?.call { it.searchGames(query, limit.toUInt()) } ?: emptyList()
    }

    override suspend fun details(gameId: Long): GameDetails {
        requirePerm("library.read")
        return checkNotNull(app?.zl?.core?.call { it.gameDetails(gameId) })
    }

    override suspend fun playStats(gameId: Long): PlayStats {
        requirePerm("library.read")
        return checkNotNull(app?.zl?.core?.call { it.playStats(gameId) })
    }

    override suspend fun rescan(platformId: String) {
        requirePerm("library.rescan")
        val app = app ?: return
        val since = NewArt.now()
        val summary = app.zl.core.call { it.scan(QuietScan) }
        // The open screens read the library again, as after any scan of the app.
        withContext(Dispatchers.Main) { app.resumeTick++ }
        dispatchEvent(PodEvent.LibraryChanged(setOf(platformId)))
        NewArt.afterScan(app.zl, since, summary.added) { app.resumeTick++ }
    }

    override fun launch(game: GameEntry) {
        requirePerm("launch")
        app?.launchGame(game)
    }

    override suspend fun getSecret(key: String): String? {
        requirePerm("secrets")
        val app = app ?: return null
        return withContext(Dispatchers.IO) { KeystoreSecrets(app.zl).get(secretName(key)) }
    }

    override suspend fun setSecret(key: String, value: String?) {
        requirePerm("secrets")
        val app = app ?: return
        val saved = withContext(Dispatchers.IO) { KeystoreSecrets(app.zl).set(secretName(key), value) }
        check(saved) { "the key store did not save the secret" }
    }

    /** The name of a Pod's secret in the launcher's store: one namespace per Pod, so no Pod reads another's. */
    private fun secretName(key: String): String = "pod.$podId.$key"

    override suspend fun httpGet(url: String, headers: Map<String, String>): ByteArray {
        requirePerm("http")
        return HostTransfers.get(url, headers)
    }

    override suspend fun httpPost(url: String, body: ByteArray, contentType: String, headers: Map<String, String>): ByteArray {
        requirePerm("http")
        return HostTransfers.post(url, body, contentType, headers)
    }

    override suspend fun download(url: String, dest: File, headers: Map<String, String>, progress: (Long, Long?) -> Unit): Long {
        requirePerm("http")
        return HostTransfers.download(url, dest, headers, progress)
    }

    override suspend fun extract(archive: File, dest: File, password: String?, progress: (Long, Long?) -> Unit): List<File> {
        requirePerm("archive")
        return HostTransfers.extract(archive, dest, password, progress)
    }

    override val work: PodWork
        get() {
            requirePerm("work")
            val app = checkNotNull(app) { "no app: work needs the launcher's process" }
            return PodWorkRunner.forPod(app.zl, podId) { error ->
                CoroutineScope(Dispatchers.IO).launch { runCatching { app.zl.core.call { it.recordPodCrash(podId, "Task failed: $error") } } }
            }
        }

    override suspend fun platformRoot(platformId: String): String? {
        requirePerm("folders")
        return app?.zl?.core?.call { it.platformFolder(platformId) }?.path
    }
}
