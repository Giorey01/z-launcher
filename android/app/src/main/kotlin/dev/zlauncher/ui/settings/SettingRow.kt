package dev.zlauncher.ui.settings

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector

/** The control at the right of a settings row. */
sealed interface Control {
    /** No control (an action row or a label). */
    data object None : Control

    /** An arrow: the row opens a subpage. */
    data object Chevron : Control

    /** A pill showing the chosen option; A opens the list of [options]. */
    data class Dropdown(
        /** Texts of the options. */
        val options: List<String>,
        /** Index of the chosen option. */
        val selected: Int,
        /** Called with the new index. */
        val onSelect: (Int) -> Unit,
    ) : Control

    /** A slider with minus and plus; Left and Right change it by [step]. */
    data class Slider(
        /** Current value. */
        val value: Float,
        /** Smallest value. */
        val min: Float,
        /** Largest value. */
        val max: Float,
        /** Size of one step. */
        val step: Float,
        /** The label at the left of the track. */
        val label: String,
        /** Called with the new value. */
        val onChange: (Float) -> Unit,
    ) : Control

    /** A switch. */
    data class Switch(
        /** Whether it is on. */
        val on: Boolean,
        /** Called with the new state. */
        val onChange: (Boolean) -> Unit,
    ) : Control

    /** A row of colour swatches; Left and Right move the choice, A confirms. */
    data class Swatches(
        /** The colours; `null` is the theme's own accent. */
        val colors: List<Color?>,
        /** Index of the chosen swatch. */
        val selected: Int,
        /** Called with the new index. */
        val onSelect: (Int) -> Unit,
    ) : Control

    /** A text at the right (a value that is only shown, or a check mark with [text] `✓`). */
    data class Value(
        /** The text. */
        val text: String,
    ) : Control
}

/** One row of a settings list. [onActivate] runs on A for rows without an inline control (chevrons, actions, values). */
data class SettingRow(
    /** Stable id, used to keep the focus on the same row when the list is rebuilt. */
    val id: String,
    /** Icon at the left. */
    val icon: ImageVector,
    /** Title. */
    val title: String,
    /** Smaller text under the title. */
    val subtitle: String? = null,
    /** The control at the right. */
    val control: Control = Control.None,
    /** Y on the row (for example to remove a folder). */
    val onSecondary: (() -> Unit)? = null,
    /** A on the row. */
    val onActivate: (() -> Unit)? = null,
    /** A picture drawn instead of the icon (a theme preview). */
    val preview: String? = null,
)
