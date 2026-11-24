package dev.zlauncher

import android.content.Intent
import android.os.Bundle
import android.view.KeyEvent
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.viewModels
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import dev.zlauncher.launch.LaunchExecutor
import dev.zlauncher.launch.LaunchFailure
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.Boot
import dev.zlauncher.nav.Screen
import dev.zlauncher.nav.UiEvent
import dev.zlauncher.ui.LauncherRoot
import dev.zlauncher.ui.theme.ZLauncherTheme
import kotlinx.coroutines.launch

/** The only activity: the launcher's Home, which can also be the device Home. */
class MainActivity : ComponentActivity() {
    /** Navigation, launching and messages. */
    private val app: AppViewModel by viewModels()

    /** Draws the UI and carries out the launch requests of the view model. */
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent { ZLauncherTheme { LauncherRoot(app) } }
        lifecycleScope.launch {
            repeatOnLifecycle(Lifecycle.State.STARTED) { app.events.collect(::carryOut) }
        }
    }

    /** Starts what the view model asked for and reports a failure. */
    private fun carryOut(event: UiEvent) {
        val failure: LaunchFailure? = when (event) {
            is UiEvent.StartGame -> LaunchExecutor.launch(this, event.intent)
            is UiEvent.StartApp -> LaunchExecutor.launchApp(this, event.packageName)
        }
        failure?.let(app::report)
    }

    /** Back in the foreground (also after a game): screens put their focus back. */
    override fun onResume() {
        super.onResume()
        app.onResumed()
    }

    /** The system Home button while the launcher is already open: back to the Home screen. */
    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        if (intent.action == Intent.ACTION_MAIN) app.goHome()
    }

    /** Gamepad buttons the focus system does not use: B is back, Start opens the app drawer. */
    override fun onKeyDown(keyCode: Int, event: KeyEvent): Boolean = when {
        event.repeatCount > 0 -> super.onKeyDown(keyCode, event)
        keyCode == KeyEvent.KEYCODE_BUTTON_B -> true.also { onBackPressedDispatcher.onBackPressed() }
        keyCode == KeyEvent.KEYCODE_BUTTON_START && app.boot == Boot.Ready -> true.also { app.open(Screen.Drawer) }
        else -> super.onKeyDown(keyCode, event)
    }
}
