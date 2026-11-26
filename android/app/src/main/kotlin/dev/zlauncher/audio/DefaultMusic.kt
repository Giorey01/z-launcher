package dev.zlauncher.audio

import android.content.Context
import android.util.Log
import dev.zlauncher.core.nativecore.defaultMusicDirName
import java.io.File

/** Folder of the app's assets with the built-in tracks. */
private const val ASSET_DIR = "music"

/**
 * Copies the launcher's built-in tracks (assets `music/`) into the data folder, where the core
 * picks them up when the theme has no music of its own. A track already there with the same size
 * is left alone.
 */
object DefaultMusic {
    /** Copies the tracks into `dataDir`; a failure only leaves the launcher without its own music. */
    fun install(context: Context, dataDir: File) {
        val dir = File(dataDir, defaultMusicDirName())
        runCatching {
            dir.mkdirs()
            for (name in context.assets.list(ASSET_DIR).orEmpty()) {
                val dest = File(dir, name)
                val size = context.assets.openFd("$ASSET_DIR/$name").use { it.length }
                if (dest.length() == size) continue
                context.assets.open("$ASSET_DIR/$name").use { input -> dest.outputStream().use { input.copyTo(it) } }
            }
        }.onFailure { Log.w("ZMusic", "built-in music not copied", it) }
    }
}
