package dev.zlauncher.nav

import android.app.Application
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import dev.zlauncher.R
import dev.zlauncher.ZLauncherApp
import dev.zlauncher.core.nativecore.Candidate
import dev.zlauncher.core.nativecore.CoreException
import dev.zlauncher.core.nativecore.GameEntry
import dev.zlauncher.core.nativecore.Plan
import dev.zlauncher.launch.LaunchFailure
import dev.zlauncher.launch.toMessage
import dev.zlauncher.ui.pad.FocusMemory
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.launch

/** The root state of the launcher: which screen is on top, launching games and apps, and messages. */
class AppViewModel(application: Application) : AndroidViewModel(application) {
    /** The shared application object. */
    private val zl = application as ZLauncherApp

    /** What is shown before the Home. */
    var boot by mutableStateOf(Boot.Loading)
        private set

    /** Screens above the Home, the top one last. */
    val stack = mutableStateListOf<Screen>()

    /** The game waiting for an emulator pick, while [Screen.Picker] is shown. */
    var choice by mutableStateOf<Choice?>(null)
        private set

    /** The message on screen, if any. */
    var message by mutableStateOf<UiMessage?>(null)

    /** Incremented every time the launcher comes back to the foreground; screens restore their focus on it. */
    var resumeTick by mutableIntStateOf(0)
        private set

    /** Requests for the activity. */
    private val eventChannel = Channel<UiEvent>(Channel.BUFFERED)

    /** The requests as a flow the activity collects. */
    val events = eventChannel.receiveAsFlow()

    /** One focus memory per screen id. */
    private val memories = HashMap<String, FocusMemory>()

    init {
        viewModelScope.launch {
            boot = try {
                if (zl.core.call { it.onboardingDone() }) Boot.Ready else Boot.Onboarding
            } catch (e: CoreException) {
                message = e.toMessage()
                Boot.Onboarding
            }
        }
    }

    /** The focus memory of the screen [id]; it survives the screen leaving the composition. */
    fun memory(id: String): FocusMemory = memories.getOrPut(id) { FocusMemory() }

    /** The screen on top, or `null` when the Home is. */
    val top: Screen? get() = stack.lastOrNull()

    /** Opens a screen above the current one. */
    fun open(screen: Screen) {
        if (top != screen) stack.add(screen)
    }

    /** Goes back one screen; `false` when the Home is already shown (BACK never leaves the Home). */
    fun back(): Boolean {
        val screen = stack.removeLastOrNull() ?: return false
        if (screen == Screen.Picker) choice = null
        return true
    }

    /** Returns to the Home from anywhere (the system Home button while the launcher is the Home). */
    fun goHome() {
        stack.clear()
        choice = null
    }

    /** The launcher came back to the foreground. */
    fun onResumed() {
        resumeTick++
    }

    /** The setup finished: show the Home. */
    fun finishOnboarding() {
        goHome()
        boot = Boot.Ready
    }

    /** Runs the setup again (from the Home). */
    fun reopenSetup() {
        goHome()
        boot = Boot.Onboarding
    }

    /** Shows why a launch failed. */
    fun report(failure: LaunchFailure) {
        message = failure.toMessage()
    }

    /** Starts an installed app. */
    fun launchApp(packageName: String) {
        eventChannel.trySend(UiEvent.StartApp(packageName))
    }

    /** Plans the launch of [game] in the core and starts it, or asks which emulator to use. */
    fun launchGame(game: GameEntry) {
        viewModelScope.launch {
            try {
                val packages = zl.apps.list().map { it.packageName }
                handle(game, zl.core.call { it.planLaunch(game.id, packages) })
            } catch (e: CoreException) {
                message = e.toMessage()
            }
        }
    }

    /** Acts on a plan: start, ask, or say that nothing can launch the game. */
    private fun handle(game: GameEntry, plan: Plan) {
        when (plan) {
            is Plan.Launch -> eventChannel.trySend(UiEvent.StartGame(plan.intent))
            is Plan.NeedsChoice -> if (plan.candidates.isEmpty()) {
                message = UiMessage(R.string.msg_no_emulator, listOf(game.platformName))
            } else {
                choice = Choice(game, plan.candidates)
                memories.remove("picker")
                open(Screen.Picker)
            }
        }
    }

    /** The user picked an emulator for the waiting game: remember it for the platform and launch. */
    fun choose(candidate: Candidate) {
        val pending = choice ?: return
        viewModelScope.launch {
            try {
                zl.core.call { it.setPlatformDefault(pending.game.platformId, candidate.profile) }
                back()
                launchGame(pending.game)
            } catch (e: CoreException) {
                message = e.toMessage()
            }
        }
    }
}
