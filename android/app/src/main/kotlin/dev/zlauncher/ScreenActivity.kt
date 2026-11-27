package dev.zlauncher

import android.app.ActivityOptions
import android.content.Context
import android.content.Intent
import android.hardware.display.DisplayManager
import android.os.Bundle
import android.view.Display
import android.view.KeyEvent
import android.view.WindowManager
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import dev.zlauncher.core.nativecore.SoundAction
import dev.zlauncher.display.DisplayRoles
import dev.zlauncher.display.Role
import dev.zlauncher.launch.LaunchExecutor
import dev.zlauncher.launch.LaunchFailure
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.nav.Boot
import dev.zlauncher.nav.NowPlaying
import dev.zlauncher.nav.Overlay
import dev.zlauncher.nav.Screen
import dev.zlauncher.nav.UiEvent
import dev.zlauncher.ui.ScreenRoot
import dev.zlauncher.ui.settings.SettingsKeys
import dev.zlauncher.ui.theme.ZLauncherTheme
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch
import java.lang.ref.WeakReference

/**
 * One window of the launcher. Which part of the UI it shows depends on the display it is on (see
 * [DisplayRoles]): the grid, the hero area, or both. All windows share one [AppViewModel], so what is focused
 * on one display shows on the other. Gamepad input is handled by the window that shows the grid: the other
 * window forwards its keys there.
 */
abstract class ScreenActivity : ComponentActivity() {
    /** The launcher's state, shared by every window. */
    protected val app: AppViewModel by lazy {
        ViewModelProvider(application as ZLauncherApp, ViewModelProvider.AndroidViewModelFactory(application))[AppViewModel::class.java]
    }

    /** What this window shows now. */
    var role by mutableStateOf(Role.SINGLE)
        private set

    /** Grows whenever a display is added, removed or changed, so the role is worked out again. */
    private var displaysTick by mutableIntStateOf(0)

    /** Reacts to displays coming and going (an HDMI screen, the second screen switching on). */
    private val displayListener = object : DisplayManager.DisplayListener {
        /** A display appeared. */
        override fun onDisplayAdded(displayId: Int) = bump()

        /** A display went away. */
        override fun onDisplayRemoved(displayId: Int) = bump()

        /** A display changed state or size. */
        override fun onDisplayChanged(displayId: Int) = bump()

        /** Re-evaluates the roles. */
        private fun bump() {
            displaysTick++
        }
    }

    /** Builds the content: the themed root that picks the part for this window's role. */
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        getSystemService(DisplayManager::class.java).registerDisplayListener(displayListener, null)
        hideSystemBars()
        setContent {
            val theme by app.theme.collectAsState()
            val settings = theme?.settings
            val context = LocalContext.current
            val displayId = display?.displayId ?: 0
            val computed = remember(settings, displaysTick) {
                if (settings == null) {
                    Role.SINGLE
                } else {
                    val dm = context.getSystemService(DisplayManager::class.java)
                    val candidates = DisplayRoles.candidates(dm, settings.externalDisplayDocking)
                    DisplayRoles.resolve(settings, candidates, displayId)
                }
            }
            LaunchedEffect(computed) { onRoleChanged(computed) }
            theme?.let {
                ZLauncherTheme(it) {
                    ScreenRoot(app, role, displayId)
                    // The second screen beside the game, darkened as the settings say (the touches go through).
                    val veil = 1f - it.settings.secondaryBrightness
                    if (displayId != Display.DEFAULT_DISPLAY && veil > 0f) {
                        Box(Modifier.fillMaxSize().background(Color.Black.copy(alpha = veil)))
                    }
                }
            }
        }
        lifecycleScope.launch { repeatOnLifecycle(Lifecycle.State.STARTED) { followEvents() } }
        lifecycleScope.launch {
            snapshotFlow { app.nowPlaying }.collect(::applyNowPlaying)
        }
    }

    /** Draws edge to edge and hides the system bars: the launcher uses the whole display. */
    private fun hideSystemBars() {
        WindowCompat.setDecorFitsSystemWindows(window, false)
        WindowInsetsControllerCompat(window, window.decorView).apply {
            hide(WindowInsetsCompat.Type.systemBars())
            systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
        }
    }

    /** Stores a new role; a window that shows the grid becomes the one that takes the input. */
    private fun onRoleChanged(next: Role) {
        role = next
        if (next != Role.HERO) inputHost = WeakReference(this)
        onRoleApplied(next)
    }

    /** Hook for the subclasses: the role of this window is now [role]. */
    protected open fun onRoleApplied(role: Role) = Unit

    /** Carries out the launch requests of the view model, while this window is the one that shows the grid. */
    private suspend fun followEvents() {
        snapshotFlowOfRole().collectLatest { r -> if (r != Role.HERO) app.events.collect(::carryOut) }
    }

    /** The role as a flow. */
    private fun snapshotFlowOfRole() = androidx.compose.runtime.snapshotFlow { role }

    /** Starts what the view model asked for and reports a failure. */
    private fun carryOut(event: UiEvent) {
        val failure: LaunchFailure? = when (event) {
            is UiEvent.StartGame -> startGame(event)
            is UiEvent.StartApp -> LaunchExecutor.launchApp(this, event.packageName)
        }
        failure?.let(app::report)
    }

    /**
     * Starts a game on the display the user picked, or on the one the settings choose; when that is another
     * display, this one shows the game as now playing.
     */
    private fun startGame(event: UiEvent.StartGame): LaunchFailure? {
        val target = event.displayId ?: launchDisplay()
        val failure = LaunchExecutor.launch(this, event.intent, target)
        if (failure == null && event.game != null) app.onGameStarted(event.game, event.intent.`package`)
        val elsewhere = target?.takeIf { it != currentDisplayId() }
        if (failure == null && event.game != null && elsewhere != null) app.startNowPlaying(event.game, elsewhere)
        return failure
    }

    /** The display the settings choose for games, `null` for this one. */
    private fun launchDisplay(): Int? {
        val settings = app.theme.value?.settings ?: return null
        val candidates = DisplayRoles.candidates(getSystemService(DisplayManager::class.java), settings.externalDisplayDocking)
        return DisplayRoles.launchDisplay(settings, candidates)
    }

    /** The id of the display this window is on. */
    private fun currentDisplayId(): Int = display?.displayId ?: Display.DEFAULT_DISPLAY

    /**
     * While this window shows the now playing screen it must not take the input focus, or a touch here would
     * move the gamepad away from the game. The dimming is only drawn (a black layer): a window brightness
     * override was tried on the Thor and Android applied it to the game's display, not to this one.
     */
    private fun applyNowPlaying(playing: NowPlaying?) {
        if (playing != null && playing.displayId != currentDisplayId()) {
            window.addFlags(WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE)
        } else {
            window.clearFlags(WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE)
        }
    }

    /** Back in the foreground (also after a game): screens put their focus back. */
    override fun onResume() {
        super.onResume()
        app.endNowPlaying(currentDisplayId())
        if (role != Role.HERO) inputHost = WeakReference(this)
        app.onResumed()
    }

    /** Stops listening to the displays. */
    override fun onDestroy() {
        getSystemService(DisplayManager::class.java).unregisterDisplayListener(displayListener)
        super.onDestroy()
    }

    /** Sends the keys of a window that only shows the hero to the window that shows the grid. */
    override fun dispatchKeyEvent(event: KeyEvent): Boolean {
        val host = inputHost?.get()
        if (host != null && host !== this && role == Role.HERO) {
            return host.dispatchKeyEvent(event)
        }
        val inSettings = app.top == Screen.Settings && !app.settings.editing
        val consumed = (inSettings && event.action == KeyEvent.ACTION_DOWN && SettingsKeys.handle(app, event)) ||
            (inSettings && event.action == KeyEvent.ACTION_UP && SettingsKeys.consumes(event))
        return if (consumed) true else super.dispatchKeyEvent(event)
    }

    /** Gamepad buttons the focus system does not use: B is back, Start opens the Start menu. */
    override fun onKeyDown(keyCode: Int, event: KeyEvent): Boolean {
        if (event.repeatCount > 0) return super.onKeyDown(keyCode, event)
        val handled = handleGamepadButton(keyCode)
        return if (handled) true else super.onKeyDown(keyCode, event)
    }

    /** Handles launcher gamepad shortcuts. */
    private fun handleGamepadButton(keyCode: Int): Boolean = when (keyCode) {
        KeyEvent.KEYCODE_BUTTON_B -> true.also { goBack() }
        KeyEvent.KEYCODE_BUTTON_START -> if (app.boot == Boot.Ready) true.also { toggleStartMenu() } else false
        KeyEvent.KEYCODE_BUTTON_L1 -> handleBumper(-1)
        KeyEvent.KEYCODE_BUTTON_R1 -> handleBumper(1)
        KeyEvent.KEYCODE_BUTTON_X -> handleX()
        else -> false
    }

    /** X toggles search when inside the app drawer. */
    private fun handleX(): Boolean = if (app.top is Screen.Drawer) {
        app.drawerSearching = !app.drawerSearching
        app.play(SoundAction.OPEN_EDIT)
        true
    } else false

    /** Steps platforms when in a folder, tabs in the Pod screen and in a game's sheet. */
    private fun handleBumper(delta: Int): Boolean = when {
        app.top is Screen.Platform && app.overlay == null -> {
            app.stepPlatform(delta)
            true
        }
        app.top is Screen.GameInfo -> {
            if (app.infoTabs.step(delta)) app.play(SoundAction.NAVIGATE)
            true
        }
        app.top == Screen.PodManagement -> {
            app.podManager.stepTab(delta)
            app.play(SoundAction.NAVIGATE)
            true
        }
        else -> false
    }

    /** B: close the menu or leave the screen (never the Home). */
    private fun goBack() {
        if (app.back()) app.play(SoundAction.BACK)
    }

    /** Start opens the Start menu on the Home and closes it when it is open. */
    private fun toggleStartMenu() {
        if (app.top != null) return
        app.overlay = if (app.overlay == Overlay.StartMenu) null else Overlay.StartMenu.also { app.play(SoundAction.OPEN_EDIT) }
    }

    /** Starts the companion window on the other display when the dual-screen mode needs one. */
    protected fun startCompanion(context: Context, displayId: Int, target: Class<out ScreenActivity>) {
        val options = ActivityOptions.makeBasic().setLaunchDisplayId(displayId).toBundle()
        context.startActivity(Intent(context, target).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK), options)
    }

    /** The window that handles the input (it shows the grid). */
    companion object {
        /** The window that shows the grid; the others forward their keys to it. */
        var inputHost: WeakReference<ScreenActivity>? = null
    }
}
