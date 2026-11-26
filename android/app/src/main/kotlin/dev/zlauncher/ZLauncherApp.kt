package dev.zlauncher

import android.app.Application
import android.os.Build
import androidx.lifecycle.ViewModelStore
import androidx.lifecycle.ViewModelStoreOwner
import coil.ImageLoader
import coil.ImageLoaderFactory
import coil.decode.GifDecoder
import coil.decode.ImageDecoderDecoder
import dev.zlauncher.apps.InstalledApps
import dev.zlauncher.apps.RecentApps
import dev.zlauncher.audio.DefaultMusic
import dev.zlauncher.audio.MusicPlayer
import dev.zlauncher.audio.MusicVisibility
import dev.zlauncher.audio.SoundFx
import dev.zlauncher.core.CoreProvider
import dev.zlauncher.core.KeystoreSecrets
import dev.zlauncher.ui.art.ArtResolver
import dev.zlauncher.ui.theme.ThemeStore
import dev.zlauncher.pods.PodLoader
import dev.zlauncher.pods.PodRegistry
import dev.zlauncher.sessions.SessionTracker
import java.io.File
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking

/**
 * The application: owns the objects every screen on every display shares (the Rust core, the installed-app
 * list, the theme, the art resolver, the sounds) and the view-model store, so the launcher's state
 * outlives its activities and is the same on both displays.
 */
class ZLauncherApp : Application(), ImageLoaderFactory, ViewModelStoreOwner {
    /** The Rust core, opened lazily on the first call. */
    lateinit var core: CoreProvider
        private set

    /** Launcher activities installed on the device and their icons. */
    lateinit var apps: InstalledApps
        private set

    /** The apps launched last. */
    lateinit var recent: RecentApps
        private set

    /** Answers about theme art, cached per theme. */
    lateinit var art: ArtResolver
        private set

    /** The interface sounds. */
    lateinit var sound: SoundFx
        private set

    /** The background music of the theme, playing while a launcher screen is visible. */
    lateinit var music: MusicPlayer
        private set

    /** The resolved theme shared by all screens. */
    lateinit var themeStore: ThemeStore
        private set

    /** Play sessions from the system's usage events. */
    lateinit var sessions: SessionTracker
        private set

    /** The Pods this build adds (none in the open-source build). */
    lateinit var pods: PodRegistry
        private set

    /** The store of the process-wide view models. */
    override val viewModelStore: ViewModelStore = ViewModelStore()

    /** Creates the shared objects. */
    override fun onCreate() {
        super.onCreate()
        core = CoreProvider(filesDir, KeystoreSecrets(this))
        apps = InstalledApps(packageManager)
        recent = RecentApps(this)
        art = ArtResolver(core)
        sound = SoundFx()
        DefaultMusic.install(this, filesDir)
        music = MusicPlayer()
        registerActivityLifecycleCallbacks(MusicVisibility(music))
        themeStore = ThemeStore(core, art, sound, music)
        sessions = SessionTracker(this, core)
        val activePodFile = File(filesDir, "active_pod")
        if (activePodFile.exists()) {
            val crashedScreenPodId = runCatching { activePodFile.readText().trim() }.getOrDefault("")
            activePodFile.delete()
            if (crashedScreenPodId.isNotEmpty()) {
                CoroutineScope(Dispatchers.IO).launch {
                    core.call { it.recordPodCrash(crashedScreenPodId, "Process died while Pod screen was open") }
                }
            }
        }
        reloadPods()
    }

    /** Reloads the Pod registry with active installed external pods (if enabled) and built-ins. */
    fun reloadPods() {
        val loader = PodLoader(this, File(filesDir, "pods"))
        val onCrash: (String, String) -> Unit = { id, reason ->
            CoroutineScope(Dispatchers.IO).launch {
                core.call { it.recordPodCrash(id, reason) }
            }
        }
        val activePods = runBlocking(Dispatchers.IO) {
            runCatching {
                if (core.call { it.externalPodsEnabled() }) {
                    core.call { it.listActivePods() }
                } else {
                    emptyList()
                }
            }.getOrDefault(emptyList())
        }
        val loadResult = loader.loadExternalPods(activePods, onCrash)
        pods = PodRegistry.load(classLoader, loadResult.externalPods, onCrash)
    }

    /** The image loader of the whole app: decodes animated GIF and WebP too. */
    override fun newImageLoader(): ImageLoader = ImageLoader.Builder(this)
        .components { add(if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) ImageDecoderDecoder.Factory() else GifDecoder.Factory()) }
        .build()
}
