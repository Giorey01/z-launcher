package dev.zlauncher.nav

import androidx.annotation.StringRes
import dev.zlauncher.core.nativecore.Candidate
import dev.zlauncher.core.nativecore.GameEntry
import dev.zlauncher.core.nativecore.LaunchIntent

/** A screen above the Home. The Home itself is the root and is not in the stack. */
sealed interface Screen {
    /** The folder of one platform: its games. */
    data class Platform(
        /** Canonical platform id. */
        val id: String,
        /** English platform name. */
        val name: String,
    ) : Screen

    /** The list of installed apps. */
    data object Drawer : Screen

    /** The choice of an emulator for the game in [AppViewModel.choice]. */
    data object Picker : Screen

    /** The settings (the main part on the hero display in the dual-screen mode). */
    data object Settings : Screen

    /** The screen of a Pod of this build. */
    data class Pod(
        /** The Pod's id. */
        val id: String,
        /** The destination route within the Pod. */
        val route: dev.zlauncher.pods.PodRoute = dev.zlauncher.pods.PodRoute(),
    ) : Screen

    /** The Pod management and store screen (§11). */
    data object PodManagement : Screen

    /** The Info tab of a game: details, play time, trailer and screenshots. */
    data class GameInfo(
        /** The game. */
        val game: GameEntry,
    ) : Screen

    /** The consoles an Android app (a recomp or port) can be filed under. */
    data class ConsolePicker(
        /** Package of the app. */
        val packageName: String,
        /** Label of the app, used as the game's title. */
        val label: String,
    ) : Screen
}

/** A menu floating over the Home. At most one is open. */
sealed interface Overlay {
    /** The Y menu of a game: preview on the left, entries on the right. */
    data class GameMenu(
        /** The game the menu is about. */
        val game: GameEntry,
        /** Focus key of the tile that opened it, to give the focus back on close. */
        val originKey: String,
    ) : Overlay

    /** The Start menu: Apps, Sort, Settings. */
    data object StartMenu : Overlay

    /** The Y menu of an app in the drawer: pin to the dock, file under a console. */
    data class AppMenu(
        /** Package of the app. */
        val packageName: String,
        /** Label of the app. */
        val label: String,
        /** Whether the app is pinned to the dock. */
        val pinned: Boolean,
    ) : Overlay
}


/** What the hero area (the big screen of the dual mode, the top part of the single mode) shows. */
sealed interface HeroTarget {
    /** Nothing focused that has art: the wallpaper alone. */
    data object None : HeroTarget

    /** The focused platform folder. */
    data class Platform(
        /** Canonical platform id. */
        val id: String,
        /** English platform name. */
        val name: String,
    ) : HeroTarget

    /** The focused game. */
    data class Game(
        /** The game. */
        val game: GameEntry,
    ) : HeroTarget
}

/** What the app asks the activity to do (the activity owns the `Context` that starts things). */
sealed interface UiEvent {
    /** Start an emulator with this intent. */
    data class StartGame(
        /** The intent the core planned. */
        val intent: LaunchIntent,
        /** Display to start it on, `null` for the one the settings choose. */
        val displayId: Int? = null,
        /** The game, shown on the other display while it runs. */
        val game: GameEntry? = null,
    ) : UiEvent

    /** Start an installed app. */
    data class StartApp(
        /** Package of the app. */
        val packageName: String,
    ) : UiEvent
}

/** A translated message, shown for a few seconds. [args] fill the placeholders of the string. */
data class UiMessage(
    /** String resource. */
    @StringRes val res: Int,
    /** Values for the placeholders. */
    val args: List<String> = emptyList(),
)

/** A game waiting for the user to pick an emulator. */
data class Choice(
    /** The game to launch after the pick. */
    val game: GameEntry,
    /** Installed emulators that can launch it, preferred first. */
    val candidates: List<Candidate>,
    /** The pick is for this game only (from its Info), not the platform's default, and nothing is launched. */
    val forGame: Boolean = false,
)

/** What the app shows before the Home is ready. */
enum class Boot {
    /** Reading whether the setup was done. */
    Loading,

    /** The setup (onboarding) is shown. */
    Onboarding,

    /** The Home and its screens are shown. */
    Ready,
}
