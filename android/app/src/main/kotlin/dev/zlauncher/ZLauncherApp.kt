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
import dev.zlauncher.audio.SoundFx
import dev.zlauncher.core.CoreProvider
import dev.zlauncher.core.KeystoreSecrets
import dev.zlauncher.ui.art.ArtResolver
import dev.zlauncher.ui.theme.ThemeStore
import dev.zlauncher.pods.PodRegistry
import dev.zlauncher.sessions.SessionTracker

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
        themeStore = ThemeStore(core, art, sound)
        sessions = SessionTracker(this, core)
        pods = PodRegistry.load(classLoader)
    }

    /** The image loader of the whole app: decodes animated GIF and WebP too. */
    override fun newImageLoader(): ImageLoader = ImageLoader.Builder(this)
        .components { add(if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) ImageDecoderDecoder.Factory() else GifDecoder.Factory()) }
        .build()
}
