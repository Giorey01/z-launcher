package dev.zlauncher

import android.app.Application
import dev.zlauncher.apps.InstalledApps
import dev.zlauncher.core.CoreProvider

/** The application: owns the objects every screen shares (the Rust core, the installed-app list). */
class ZLauncherApp : Application() {
    /** The Rust core, opened lazily on the first call. */
    lateinit var core: CoreProvider
        private set

    /** Launcher activities installed on the device and their icons. */
    lateinit var apps: InstalledApps
        private set

    /** Creates the shared objects. */
    override fun onCreate() {
        super.onCreate()
        core = CoreProvider(filesDir)
        apps = InstalledApps(packageManager)
    }
}
