package dev.zlauncher.launch

import android.app.ActivityOptions
import android.content.ActivityNotFoundException
import android.content.Context
import android.content.Intent
import android.net.Uri
import dev.zlauncher.core.nativecore.LaunchIntent

/** Why a launch did not start; the screen shows a translated message for each. */
enum class LaunchFailure {
    /** The package or activity is not installed. */
    NotInstalled,

    /** Android refused: the launcher has no permission on a URI it must hand over. */
    Denied,

    /** Anything else the system rejected. */
    Failed,
}

/** Grants the URIs a launch needs and starts the activity. */
object LaunchExecutor {
    /**
     * Starts [spec] from [context], on [displayId] when given; returns why it failed, or `null` when the
     * activity started.
     */
    fun launch(context: Context, spec: LaunchIntent, displayId: Int? = null): LaunchFailure? = run {
        grant(context, spec)
        val options = displayId?.let { ActivityOptions.makeBasic().setLaunchDisplayId(it).toBundle() }
        context.startActivity(IntentBuilder.build(spec), options)
    }

    /** Starts the launcher activity of [packageName] (an app from the drawer or the dock). */
    fun launchApp(context: Context, packageName: String): LaunchFailure? = run {
        val intent = context.packageManager.getLaunchIntentForPackage(packageName) ?: throw ActivityNotFoundException(packageName)
        context.startActivity(intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
    }

    /** Runs [action] and maps the exceptions Android throws to a [LaunchFailure]. */
    private fun run(action: () -> Unit): LaunchFailure? = try {
        action()
        null
    } catch (_: ActivityNotFoundException) {
        LaunchFailure.NotInstalled
    } catch (_: SecurityException) {
        LaunchFailure.Denied
    } catch (_: IllegalArgumentException) {
        LaunchFailure.Failed
    }

    /** Grants the target package the URI permissions the profile lists, before the start. */
    private fun grant(context: Context, spec: LaunchIntent) {
        for (grant in spec.uriGrants) {
            var bits = 0
            if (grant.read) bits = bits or Intent.FLAG_GRANT_READ_URI_PERMISSION
            if (grant.write) bits = bits or Intent.FLAG_GRANT_WRITE_URI_PERMISSION
            context.grantUriPermission(spec.`package`, Uri.parse(grant.uri), bits)
        }
    }
}
