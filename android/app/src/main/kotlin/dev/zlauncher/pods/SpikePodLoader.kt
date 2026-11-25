package dev.zlauncher.pods

import android.content.Context
import android.util.Log
import dalvik.system.DexClassLoader
import org.json.JSONObject
import java.io.File
import java.io.FileOutputStream
import java.util.zip.ZipFile

/**
 * Temporary loader for the Fase 1 SPIKE (RF-73).
 * Loads a `.zlpod` package from external storage into the app's private files, makes `classes.dex`
 * strictly READ-ONLY (required on Android 14+), loads it via [DexClassLoader] (parent = launcher classLoader),
 * loads native libraries from `lib/arm64-v8a/`, and instantiates the entry [PodProvider].
 */
object SpikePodLoader {
    private const val TAG = "SpikePodLoader"

    fun loadSpikePod(context: Context, baseRegistry: PodRegistry): PodRegistry {
        val candidates = listOfNotNull(
            context.getExternalFilesDir(null)?.resolve("sample.spike.zlpod"),
            File("/data/local/tmp/sample.spike.zlpod"),
            File("/sdcard/Download/sample.spike.zlpod"),
            File(context.filesDir, "sample.spike.zlpod")
        )

        val packageFile = candidates.firstOrNull { it.isFile && it.canRead() }
        if (packageFile == null) {
            Log.d(TAG, "No spike pod package found in candidates: ${candidates.map { it.absolutePath }}")
            return baseRegistry
        }

        Log.i(TAG, "Found spike pod package at ${packageFile.absolutePath} (size: ${packageFile.length()} bytes)")
        return try {
            val podDir = File(context.filesDir, "pods/sample.spike/active")
            val markerFile = File(podDir, ".source_stamp")
            val currentStamp = "${packageFile.lastModified()}_${packageFile.length()}"

            // If new package or not extracted, extract cleanly
            if (!podDir.exists() || markerFile.takeIf { it.exists() }?.readText() != currentStamp) {
                Log.i(TAG, "Extracting ${packageFile.name} to private dir ${podDir.absolutePath}...")
                podDir.deleteRecursively()
                podDir.mkdirs()

                ZipFile(packageFile).use { zip ->
                    for (entry in zip.entries()) {
                        if (entry.isDirectory) continue
                        val outFile = File(podDir, entry.name)
                        outFile.parentFile?.mkdirs()
                        zip.getInputStream(entry).use { input ->
                            FileOutputStream(outFile).use { output ->
                                input.copyTo(output)
                            }
                        }
                    }
                }
                markerFile.writeText(currentStamp)
            }

            // 1. Classes.dex must be made strictly READ-ONLY (Android 14 requirement)
            val dexFile = File(podDir, "classes.dex")
            check(dexFile.isFile) { "Missing classes.dex in ${packageFile.name}" }

            val readOnlySuccess = dexFile.setReadOnly()
            val canWrite = dexFile.canWrite()
            Log.i(TAG, "Dex file: ${dexFile.absolutePath}, setReadOnly=$readOnlySuccess, canWrite=$canWrite")

            // 2. Native library path
            val nativeLibDir = File(podDir, "lib/arm64-v8a")
            Log.i(TAG, "Native library dir: ${nativeLibDir.absolutePath} (exists=${nativeLibDir.exists()})")

            // 3. DexClassLoader with launcher classloader as parent
            val codeCacheDir = context.codeCacheDir
            val classLoader = DexClassLoader(
                dexFile.absolutePath,
                codeCacheDir.absolutePath,
                nativeLibDir.absolutePath,
                context.classLoader
            )

            // 4. Read entry class from manifest.json
            val manifestFile = File(podDir, "manifest.json")
            val manifestJson = JSONObject(manifestFile.readText())
            val entryClass = manifestJson.getString("entry")
            Log.i(TAG, "Manifest entry class: $entryClass")

            // 5. Instantiate entry class
            val loadedClass = classLoader.loadClass(entryClass)
            val provider = loadedClass.getDeclaredConstructor().newInstance() as PodProvider
            val loadedPods = provider.pods()
            Log.i(TAG, "Successfully loaded ${loadedPods.size} pod(s) from $entryClass: ${loadedPods.map { it.id }}")

            PodRegistry.from(baseRegistry.pods + loadedPods, baseRegistry.problems)
        } catch (t: Throwable) {
            Log.e(TAG, "Failed to load spike pod", t)
            baseRegistry
        }
    }
}
