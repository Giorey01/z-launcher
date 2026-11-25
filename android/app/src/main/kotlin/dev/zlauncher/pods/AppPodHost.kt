package dev.zlauncher.pods

import android.content.Context
import dev.zlauncher.core.nativecore.GameDetails
import dev.zlauncher.core.nativecore.GameEntry
import dev.zlauncher.core.nativecore.PlatformEntry
import dev.zlauncher.core.nativecore.PlayStats
import dev.zlauncher.core.nativecore.ScanEvent
import dev.zlauncher.core.nativecore.ScanListener
import dev.zlauncher.core.nativecore.SoundAction
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.Screen
import dev.zlauncher.ui.theme.ZTheme
import java.io.File
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.asSharedFlow

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

/** Simple fallback strings provider returning the key or formatted string. */
class AppPodStrings : PodStrings {
    override fun get(key: String): String = key
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
    ) : this(podId, permissions, app, theme, AppPodStrings(), AppPodStorage(app.zl, podId))

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
        app?.zl?.core?.call { it.scan(QuietScan) }
    }

    override fun launch(game: GameEntry) {
        requirePerm("launch")
        app?.launchGame(game)
    }

    override suspend fun getSecret(key: String): String? {
        requirePerm("secrets")
        return null
    }

    override suspend fun setSecret(key: String, value: String?) {
        requirePerm("secrets")
    }

    override suspend fun httpGet(url: String): ByteArray {
        requirePerm("http")
        return ByteArray(0)
    }

    override suspend fun httpPost(url: String, body: ByteArray, contentType: String): ByteArray {
        requirePerm("http")
        return ByteArray(0)
    }

    override suspend fun platformRoot(platformId: String): String? {
        requirePerm("folders")
        return null
    }
}
