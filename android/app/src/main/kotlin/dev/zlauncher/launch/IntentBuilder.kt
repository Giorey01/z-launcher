package dev.zlauncher.launch

import android.content.ClipData
import android.content.ClipDescription
import android.content.Intent
import android.net.Uri
import dev.zlauncher.core.nativecore.ExtraValue
import dev.zlauncher.core.nativecore.LaunchIntent

/** Intent flags by the names the core's profiles use (`docs/spec/platforms.md`). */
private val FLAGS_BY_NAME = mapOf(
    "clear_task" to Intent.FLAG_ACTIVITY_CLEAR_TASK,
    "clear_top" to Intent.FLAG_ACTIVITY_CLEAR_TOP,
    "new_task" to Intent.FLAG_ACTIVITY_NEW_TASK,
    "no_animation" to Intent.FLAG_ACTIVITY_NO_ANIMATION,
    "grant_read_uri_permission" to Intent.FLAG_GRANT_READ_URI_PERMISSION,
    "grant_write_uri_permission" to Intent.FLAG_GRANT_WRITE_URI_PERMISSION,
)

/** Turns the core's [LaunchIntent] record into an Android [Intent]; it decides nothing, it only maps. */
object IntentBuilder {
    /** The Android flag bits of the flag names (an unknown name is a bug in the data and is ignored). */
    fun flagBits(names: List<String>): Int = names.fold(0) { bits, name -> bits or (FLAGS_BY_NAME[name] ?: 0) }

    /** Builds the intent: component, action, data and type, categories, typed extras, flags and ClipData. */
    fun build(spec: LaunchIntent): Intent {
        val intent = Intent()
        spec.action?.let { intent.action = it }
        val activity = spec.activity
        if (activity != null) intent.setClassName(spec.packageName(), activity) else intent.setPackage(spec.packageName())
        spec.categories.forEach { intent.addCategory(it) }
        applyData(intent, spec)
        spec.extras.forEach { putExtra(intent, it.key, it.value) }
        intent.addFlags(flagBits(spec.flags))
        spec.clipData?.let { clip ->
            val description = ClipDescription("", arrayOf(clip.mime))
            intent.clipData = ClipData(description, ClipData.Item(Uri.parse(clip.uri)))
        }
        return intent
    }

    /** Sets the data URI and the MIME type together (setting one clears the other). */
    private fun applyData(intent: Intent, spec: LaunchIntent) {
        val data = spec.data?.let(Uri::parse)
        when {
            data != null && spec.mime != null -> intent.setDataAndType(data, spec.mime)
            data != null -> intent.data = data
            spec.mime != null -> intent.type = spec.mime
        }
    }

    /** Puts one typed extra. */
    private fun putExtra(intent: Intent, key: String, value: ExtraValue) {
        when (value) {
            is ExtraValue.Text -> intent.putExtra(key, value.value)
            is ExtraValue.Flag -> intent.putExtra(key, value.value)
            is ExtraValue.Number -> putNumber(intent, key, value.value)
            is ExtraValue.TextList -> intent.putExtra(key, value.value.toTypedArray())
        }
    }

    /** An integer extra goes as an `int` when it fits (what emulators read) and as a `long` otherwise. */
    private fun putNumber(intent: Intent, key: String, number: Long) {
        if (number in Int.MIN_VALUE..Int.MAX_VALUE) intent.putExtra(key, number.toInt()) else intent.putExtra(key, number)
    }

    /** The package of the record (the generated field is named `package`, a Kotlin keyword). */
    private fun LaunchIntent.packageName(): String = `package`
}
