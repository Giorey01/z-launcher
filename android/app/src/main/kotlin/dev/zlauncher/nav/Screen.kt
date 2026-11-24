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
}

/** What the app asks the activity to do (the activity owns the `Context` that starts things). */
sealed interface UiEvent {
    /** Start an emulator with this intent. */
    data class StartGame(
        /** The intent the core planned. */
        val intent: LaunchIntent,
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
