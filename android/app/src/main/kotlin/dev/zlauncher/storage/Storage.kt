package dev.zlauncher.storage

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Environment
import android.os.storage.StorageManager
import android.provider.Settings
import java.io.File

/** What the storage steps of the onboarding need from Android: the all-files permission and the volumes. */
object Storage {
    /** Whether the app may read every file (`MANAGE_EXTERNAL_STORAGE`, the main storage mode of the launcher). */
    fun hasAllFilesAccess(): Boolean = Environment.isExternalStorageManager()

    /** The system screen where the user grants "All files access" to this app. */
    fun allFilesSettingsIntent(context: Context): Intent =
        Intent(Settings.ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION, Uri.parse("package:${context.packageName}"))

    /** The general all-files screen, for devices that do not open the per-app one. */
    fun allFilesFallbackIntent(): Intent = Intent(Settings.ACTION_MANAGE_ALL_FILES_ACCESS_PERMISSION)

    /**
     * Folders worth offering as a ROM root: the [suggested] one when it exists, then the root of every
     * mounted volume that is not the internal storage (microSD, USB). Volumes are only listed from Android 11,
     * where `StorageVolume.getDirectory` exists.
     */
    fun candidateFolders(context: Context, suggested: String): List<String> {
        val folders = mutableListOf<String>()
        if (File(suggested).isDirectory) folders += suggested
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            val manager = context.getSystemService(StorageManager::class.java)
            manager.storageVolumes
                .filter { !it.isPrimary }
                .mapNotNull { it.directory?.absolutePath }
                .filter { File(it).isDirectory }
                .forEach { folders += it }
        }
        return folders
    }
}
