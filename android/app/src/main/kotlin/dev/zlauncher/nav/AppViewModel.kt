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
import dev.zlauncher.core.nativecore.SessionEnd
import dev.zlauncher.core.nativecore.SoundAction
import dev.zlauncher.launch.LaunchFailure
import dev.zlauncher.launch.toMessage
import dev.zlauncher.ui.pad.FocusMemory
import dev.zlauncher.ui.settings.ArtDownload
import dev.zlauncher.ui.settings.SettingsModel
import dev.zlauncher.ui.settings.toMessage
import dev.zlauncher.ui.theme.ZTheme
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.launch

/** The root state of the launcher: which screen is on top, launching games and apps, and messages. */
class AppViewModel(application: Application) : AndroidViewModel(application) {
    /** The shared application object. */
    val zl = application as ZLauncherApp

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

    /** The state of the settings screens, shared by both displays. */
    val settings = SettingsModel(this)

    /** What the hero area shows: the focused platform or game. */
    var hero by mutableStateOf<HeroTarget>(HeroTarget.None)

    /** The platforms of the Home in order (id to name), set by the Home; LB and RB step through them in a folder. */
    var platforms by mutableStateOf<List<Pair<String, String>>>(emptyList())

    /** The floating menu over the Home, if one is open. */
    var overlay by mutableStateOf<Overlay?>(null)


    /** The game running on another display, shown on this one as the now playing screen; `null` when none. */
    var nowPlaying by mutableStateOf<NowPlaying?>(null)
        private set

    /** Whether the now playing screen is dimmed because nobody touched it for a while. */
    var nowPlayingDimmed by mutableStateOf(false)

    /** Whether search mode is active in the app drawer. */
    var drawerSearching by mutableStateOf(false)

    /** The theme every screen draws with, `null` until the first load. */
    val theme: StateFlow<ZTheme?> get() = zl.themeStore.theme

    /** Requests for the activity. */
    private val eventChannel = Channel<UiEvent>(Channel.BUFFERED)

    /** The requests as a flow the activity collects. */
    val events = eventChannel.receiveAsFlow()

    /** One focus memory per screen id. */
    private val memories = HashMap<String, FocusMemory>()

    init {
        viewModelScope.launch {
            reloadTheme()
            boot = try {
                if (zl.core.call { it.onboardingDone() }) Boot.Ready else Boot.Onboarding
            } catch (e: CoreException) {
                message = e.toMessage()
                Boot.Onboarding
            }
        }
    }

    /** Reads the active theme again (after any change of settings, theme or art). */
    suspend fun reloadTheme() {
        try {
            zl.themeStore.reload()
        } catch (e: CoreException) {
            message = e.toMessage()
        }
    }

    /** Plays the sound of [action] (nothing when the theme has none or effects are off). */
    fun play(action: SoundAction) = zl.sound.play(action)

    /** The focus memory of the screen [id]; it survives the screen leaving the composition. */
    fun memory(id: String): FocusMemory = memories.getOrPut(id) { FocusMemory() }

    /** The screen on top, or `null` when the Home is. */
    val top: Screen? get() = stack.lastOrNull()

    /** Manages external and built-in pods, store browsing and installation. */
    val podManager = dev.zlauncher.pods.PodManager(this)

    /** Opens a screen above the current one. */
    fun open(screen: Screen) {
        if (screen == Screen.Settings) settings.reset()
        if (screen == Screen.PodManagement) podManager.reload()
        if (top != screen) stack.add(screen)
        overlay = null
    }

    /** Opens a Pod from a URL (deep link or direct action). */
    fun openPodUrl(url: String) {
        open(Screen.PodManagement)
        podManager.handleDeepLink(url)
    }

    /** Goes back: closes the open menu first, else one screen; `false` when the Home is already shown. */
    fun back(): Boolean {
        if (overlay != null || drawerSearching) {
            overlay = null
            drawerSearching = false
            return true
        }
        // A dialog of the Pod screen or a settings subpage closes before the screen does.
        val handledInside = (top == Screen.PodManagement && podManager.dismissDialog()) || (top == Screen.Settings && settings.pop())
        val screen = if (handledInside) null else stack.removeLastOrNull()
        if (screen == Screen.Picker) choice = null
        return handledInside || screen != null
    }

    /** Returns to the Home from anywhere (the system Home button while the launcher is the Home). */
    fun goHome() {
        stack.clear()
        overlay = null
        choice = null
        drawerSearching = false
    }

    /** The launcher came back to the foreground: the game that was running is counted, then screens refresh. */
    fun onResumed() {
        resumeTick++
        viewModelScope.launch {
            try {
                if (zl.sessions.end() is SessionEnd.Recorded) resumeTick++
            } catch (e: CoreException) {
                message = e.toMessage()
            }
        }
    }

    /** Favourites, hidden games, the dock and apps filed under consoles. */
    val library = LibraryActions(this)

    /** Asks every screen to read its lists again (after a change to the library). */
    fun refresh() {
        resumeTick++
    }

    /** [game] started through [packageName]: its play time starts counting. */
    fun onGameStarted(game: GameEntry, packageName: String) {
        viewModelScope.launch {
            try {
                zl.sessions.begin(game.id, packageName)
            } catch (e: CoreException) {
                message = e.toMessage()
            }
        }
    }

    /** [game] started on [displayId]: the other displays show it until the launcher is back there. */
    fun startNowPlaying(game: GameEntry, displayId: Int) {
        nowPlayingDimmed = false
        nowPlaying = NowPlaying(game, displayId)
    }

    /** The launcher is in front again on [displayId]: the game there has ended or was left. */
    fun endNowPlaying(displayId: Int) {
        if (nowPlaying?.displayId != displayId) return
        nowPlaying = null
        nowPlayingDimmed = false
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

    /** Starts an installed app and remembers it as recent. */
    fun launchApp(packageName: String) {
        zl.recent.push(packageName)
        play(SoundAction.LAUNCH)
        eventChannel.trySend(UiEvent.StartApp(packageName))
    }

    /** In a platform folder: replaces it with the previous ([step] -1) or next (1) platform of the Home. */
    fun stepPlatform(step: Int) {
        val current = top as? Screen.Platform ?: return
        val index = platforms.indexOfFirst { it.first == current.id }.takeIf { it >= 0 } ?: return
        val (id, name) = platforms[(index + step).mod(platforms.size)]
        stack[stack.lastIndex] = Screen.Platform(id, name)
        play(SoundAction.NAVIGATE)
    }

    /** Opens the emulator picker for [game]'s platform, listing the installed emulators that can launch it. */
    fun chooseEmulator(game: GameEntry) = openPicker(game, forGame = false)

    /** Opens the emulator picker for [game] alone: the pick overrides the platform's default for this game. */
    fun chooseGameEmulator(game: GameEntry) = openPicker(game, forGame = true)

    /** Opens the picker of the installed emulators that can launch [game]. */
    private fun openPicker(game: GameEntry, forGame: Boolean) {
        viewModelScope.launch {
            try {
                val packages = zl.apps.list().map { it.packageName }
                val row = zl.core.call { it.platformEmulators(packages) }.firstOrNull { it.platformId == game.platformId }
                val candidates = row?.candidates.orEmpty()
                if (candidates.isEmpty()) {
                    message = UiMessage(R.string.msg_no_emulator, listOf(game.platformName))
                } else {
                    choice = Choice(game, candidates, forGame)
                    memories.remove("picker")
                    open(Screen.Picker)
                }
            } catch (e: CoreException) {
                message = e.toMessage()
            }
        }
    }

    /** Plans the launch of [game] in the core and starts it on [displayId] (or here), or asks which emulator to use. */
    fun launchGame(game: GameEntry, displayId: Int? = null) {
        viewModelScope.launch {
            try {
                val packages = zl.apps.list().map { it.packageName }
                handle(game, zl.core.call { it.planLaunch(game.id, packages) }, displayId)
            } catch (e: CoreException) {
                message = e.toMessage()
            }
        }
    }

    /** Acts on a plan: start, ask, or say that nothing can launch the game. */
    private fun handle(game: GameEntry, plan: Plan, displayId: Int?) {
        when (plan) {
            is Plan.Launch -> {
                play(SoundAction.LAUNCH)
                eventChannel.trySend(UiEvent.StartGame(plan.intent, displayId, game))
            }
            is Plan.NeedsChoice -> if (plan.candidates.isEmpty()) {
                play(SoundAction.ERROR)
                message = UiMessage(R.string.msg_no_emulator, listOf(game.platformName))
            } else {
                choice = Choice(game, plan.candidates)
                memories.remove("picker")
                open(Screen.Picker)
            }
        }
    }

    /**
     * Downloads the missing cover, hero and logo of [game] from SteamGridDB, then reloads the lists so the new
     * art shows; the result is a message.
     */
    fun downloadArt(game: GameEntry) {
        viewModelScope.launch {
            message = UiMessage(R.string.msg_sgdb_working)
            message = try {
                val download = ArtDownload()
                val report = zl.core.call { it.scrapeImages(null, game.id, download) }
                resumeTick++
                report.toMessage()
            } catch (e: CoreException) {
                e.toMessage()
            }
        }
    }

    /**
     * The user picked an emulator for the waiting game: remember it for the platform and launch, or, for a pick made
     * for this game alone, remember it for the game.
     */
    fun choose(candidate: Candidate) {
        val pending = choice ?: return
        viewModelScope.launch {
            try {
                if (pending.forGame) {
                    zl.core.call { it.setGameEmulator(pending.game.id, candidate.profile) }
                    back()
                    message = UiMessage(R.string.msg_game_emulator_set, listOf(candidate.emulatorName, pending.game.displayTitle))
                    return@launch
                }
                zl.core.call { it.setPlatformDefault(pending.game.platformId, candidate.profile) }
                back()
                launchGame(pending.game)
            } catch (e: CoreException) {
                message = e.toMessage()
            }
        }
    }
}

/** A game running on another display. */
data class NowPlaying(
    /** The game. */
    val game: GameEntry,
    /** The display it runs on. */
    val displayId: Int,
)
