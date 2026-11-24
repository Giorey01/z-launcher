package dev.zlauncher.ui.settings

import android.view.KeyEvent
import dev.zlauncher.core.nativecore.SoundAction
import dev.zlauncher.nav.AppViewModel
import kotlin.math.roundToInt

/** Keys the settings handle themselves; B is left to the activity (it goes back). */
private val HANDLED = setOf(
    KeyEvent.KEYCODE_DPAD_UP, KeyEvent.KEYCODE_DPAD_DOWN, KeyEvent.KEYCODE_DPAD_LEFT, KeyEvent.KEYCODE_DPAD_RIGHT,
    KeyEvent.KEYCODE_BUTTON_A, KeyEvent.KEYCODE_DPAD_CENTER, KeyEvent.KEYCODE_ENTER, KeyEvent.KEYCODE_BUTTON_Y,
    KeyEvent.KEYCODE_BUTTON_L1, KeyEvent.KEYCODE_BUTTON_R1,
)

/**
 * The controller of the settings screens. The lists are not Compose focus targets (they may be drawn on a display
 * that gets no input), so the buttons drive [SettingsModel] directly and both displays redraw from it.
 */
object SettingsKeys {
    /** Whether the settings take this key (so its key-up must not reach anything else either). */
    fun consumes(event: KeyEvent): Boolean = event.keyCode in HANDLED

    /** Applies a key press to the settings; `true` when it was used. */
    fun handle(app: AppViewModel, event: KeyEvent): Boolean {
        if (!consumes(event)) return false
        val theme = app.theme.value
        if (theme != null) {
            val model = app.settings
            val rows = model.rows(theme)
            val current = rows.getOrNull(model.row.coerceIn(0, (rows.size - 1).coerceAtLeast(0)))
            when (event.keyCode) {
                KeyEvent.KEYCODE_DPAD_UP -> vertical(app, rows, current, -1)
                KeyEvent.KEYCODE_DPAD_DOWN -> vertical(app, rows, current, 1)
                KeyEvent.KEYCODE_DPAD_LEFT -> horizontal(app, current, -1)
                KeyEvent.KEYCODE_DPAD_RIGHT -> horizontal(app, current, 1)
                KeyEvent.KEYCODE_BUTTON_L1 -> bumper(app, -1)
                KeyEvent.KEYCODE_BUTTON_R1 -> bumper(app, 1)
                KeyEvent.KEYCODE_BUTTON_Y -> current?.onSecondary?.invoke()
                else -> if (event.repeatCount == 0) activate(app, current)
            }
        }
        return true
    }

    /** LB and RB switch the category. */
    private fun bumper(app: AppViewModel, delta: Int) {
        app.settings.step(delta)
        app.play(SoundAction.NAVIGATE)
    }

    /** Up and down: move through the open dropdown, the rail, or the rows (wrapping at the ends). */
    private fun vertical(app: AppViewModel, rows: List<SettingRow>, current: SettingRow?, delta: Int) {
        val model = app.settings
        app.play(SoundAction.NAVIGATE)
        val dropdown = (current?.control as? Control.Dropdown)
        when {
            model.dropdown >= 0 && dropdown != null -> model.dropdown = (model.dropdown + delta).mod(dropdown.options.size)
            model.inRail -> model.step(delta)
            rows.isNotEmpty() -> model.row = (model.row + delta).mod(rows.size)
        }
    }

    /** Left and right: adjust sliders, swatches and dropdowns, or switch between the rail and the list. */
    private fun horizontal(app: AppViewModel, current: SettingRow?, delta: Int) {
        val model = app.settings
        val control = current?.control
        when {
            model.inRail -> if (delta > 0) model.inRail = false
            control is Control.Slider -> slide(app, control, delta)
            control is Control.Swatches -> control.onSelect((control.selected + delta).coerceIn(0, control.colors.size - 1))
            control is Control.Dropdown && model.dropdown < 0 -> control.onSelect((control.selected + delta).mod(control.options.size))
            delta < 0 -> if (model.pages.isEmpty()) model.inRail = true else model.pop()
        }
    }

    /** Moves a slider by one step and plays the matching sound. */
    private fun slide(app: AppViewModel, slider: Control.Slider, delta: Int) {
        val next = (((slider.value + delta * slider.step) / slider.step).roundToInt() * slider.step).coerceIn(slider.min, slider.max)
        if (next == slider.value) return
        app.play(if (delta > 0) SoundAction.SLIDER_INCREASE else SoundAction.SLIDER_DECREASE)
        slider.onChange(next)
    }

    /** A: open a dropdown or pick its highlighted option, toggle a switch, or run the row's action. */
    private fun activate(app: AppViewModel, current: SettingRow?) {
        val model = app.settings
        if (model.inRail) return model.inRail.let { model.inRail = false }
        app.play(SoundAction.SELECT)
        when (val control = current?.control) {
            is Control.Dropdown -> if (model.dropdown >= 0) {
                control.onSelect(model.dropdown)
                model.dropdown = -1
            } else {
                model.dropdown = control.selected
            }
            is Control.Switch -> control.onChange(!control.on)
            else -> current?.onActivate?.invoke()
        }
    }
}
