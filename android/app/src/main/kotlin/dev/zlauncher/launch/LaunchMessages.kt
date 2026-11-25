package dev.zlauncher.launch

import dev.zlauncher.R
import dev.zlauncher.core.nativecore.CoreException
import dev.zlauncher.nav.UiMessage

/** Variables whose absence means the app has no access to the ROM through a granted folder. */
private val FOLDER_ACCESS_VARIABLES = setOf("rom_saf", "rom_saf_tree")

/** The translated message for a launch that did not start. */
fun LaunchFailure.toMessage(): UiMessage = when (this) {
    LaunchFailure.NotInstalled -> UiMessage(R.string.msg_launch_not_installed)
    LaunchFailure.Denied -> UiMessage(R.string.msg_launch_denied)
    LaunchFailure.Failed -> UiMessage(R.string.msg_launch_failed)
}

/** The translated message for an error code of the core. */
fun CoreException.toMessage(): UiMessage = when (this) {
    is CoreException.Launch ->
        if (code == "missing_variable" && detail in FOLDER_ACCESS_VARIABLES) {
            UiMessage(R.string.msg_need_folder_access)
        } else {
            UiMessage(R.string.msg_launch_error_code, listOf(code, detail))
        }
    is CoreException.Database, is CoreException.DatabaseTooNew -> UiMessage(R.string.msg_database)
    is CoreException.Data -> UiMessage(R.string.msg_data, listOf(code))
    is CoreException.NotFound -> UiMessage(R.string.msg_not_found)
    is CoreException.InvalidInput -> UiMessage(R.string.msg_invalid_input)
    is CoreException.Theme -> UiMessage(R.string.msg_theme_import, listOf(code))
    is CoreException.DockFull -> UiMessage(R.string.msg_dock_full)
    is CoreException.Io -> UiMessage(R.string.msg_invalid_input)
    is CoreException.Pod -> UiMessage(R.string.msg_pod, listOf(issues.joinToString(", ")))
}

