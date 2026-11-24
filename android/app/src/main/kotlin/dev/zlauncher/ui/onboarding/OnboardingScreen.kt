package dev.zlauncher.ui.onboarding

import android.app.Activity
import android.app.role.RoleManager
import android.content.Context
import android.content.Intent
import android.net.Uri
import androidx.activity.compose.BackHandler
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.platform.LocalContext
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.LifecycleEventEffect
import androidx.lifecycle.viewmodel.compose.viewModel
import dev.zlauncher.core.nativecore.safTreeForPath
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.storage.Storage

/** The setup (RF-29): a sequence of steps, all reachable with the controller; B goes back one step. */
@Composable
fun OnboardingScreen(app: AppViewModel, vm: OnboardingViewModel = viewModel()) {
    val ui by vm.ui.collectAsState()
    val context = LocalContext.current
    LaunchedEffect(Unit) { vm.restart() }
    LifecycleEventEffect(Lifecycle.Event.ON_RESUME) {
        vm.refreshPermission()
        vm.setRoleHeld(isHomeRoleHeld(context))
    }
    BackHandler(enabled = true) { vm.back() }
    val picker = rememberFolderPicker(vm)
    val role = rememberRoleRequest(vm)
    when (ui.step) {
        Step.Welcome -> WelcomeStep(vm::next)
        Step.Permission -> PermissionStep(ui.allFiles, ui.error, { openAllFilesSettings(context) }, vm::next)
        Step.Folders -> FoldersStep(ui, { path -> picker(path?.let(::pickerHint)) }, vm::next)
        Step.Mode -> ModeStep(vm::chooseMode)
        Step.Scan -> ScanStep(ui, vm::startScan, vm::next)
        Step.Emulators -> EmulatorsStep(ui, vm::detect, vm::acceptDefaults, vm::next)
        Step.HomeRole -> HomeRoleStep(ui.roleHeld, { role(context) }, vm::next)
        Step.Cocoon -> CocoonStep(ui.cocoon?.path.orEmpty(), ui.importing, vm::importCocoon, vm::skipCocoon)
        Step.Done -> DoneStep { vm.finish(app::finishOnboarding) }
    }
}

/** The tree URI the folder picker opens at for [path], when it is on a storage volume. */
private fun pickerHint(path: String): Uri? = safTreeForPath(path)?.let(Uri::parse)

/** Opens the all-files screen of this app, or the general one when the device has no per-app screen. */
private fun openAllFilesSettings(context: Context) {
    try {
        context.startActivity(Storage.allFilesSettingsIntent(context))
    } catch (_: android.content.ActivityNotFoundException) {
        context.startActivity(Storage.allFilesFallbackIntent())
    }
}

/** Whether this app is the device's Home. */
private fun isHomeRoleHeld(context: Context): Boolean =
    context.getSystemService(RoleManager::class.java).isRoleHeld(RoleManager.ROLE_HOME)

/**
 * The system folder picker: returns a function that opens it at an optional start location. A granted folder
 * is kept across restarts (persistable permission) and handed to the view model.
 */
@Composable
private fun rememberFolderPicker(vm: OnboardingViewModel): (Uri?) -> Unit {
    val context = LocalContext.current
    val launcher = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocumentTree()) { uri ->
        if (uri != null) {
            context.contentResolver.takePersistableUriPermission(uri, Intent.FLAG_GRANT_READ_URI_PERMISSION)
            vm.addPickedTree(uri.toString())
        }
    }
    return { start -> launcher.launch(start) }
}

/** Asks the system to make this app the Home; whatever the answer, the step is left. */
@Composable
private fun rememberRoleRequest(vm: OnboardingViewModel): (Context) -> Unit {
    val launcher = rememberLauncherForActivityResult(ActivityResultContracts.StartActivityForResult()) { result ->
        vm.setRoleHeld(result.resultCode == Activity.RESULT_OK)
        vm.next()
    }
    return { context ->
        val manager = context.getSystemService(RoleManager::class.java)
        if (manager.isRoleAvailable(RoleManager.ROLE_HOME)) launcher.launch(manager.createRequestRoleIntent(RoleManager.ROLE_HOME)) else vm.next()
    }
}
