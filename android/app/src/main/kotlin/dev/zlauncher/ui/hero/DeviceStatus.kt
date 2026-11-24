package dev.zlauncher.ui.hero

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import android.os.BatteryManager
import android.text.format.DateFormat
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import java.util.Date

/** What the status indicators show. */
data class DeviceStatus(
    /** Battery level in percent, `-1` when unknown. */
    val battery: Int,
    /** The battery is charging. */
    val charging: Boolean,
    /** The device is on Wi-Fi. */
    val wifi: Boolean,
    /** The time of day, as the user's locale writes it. */
    val time: String,
)

/** Default battery scale used when the system does not report one. */
private const val DEFAULT_BATTERY_SCALE = 100

/** Factor to convert fractional battery level to percentage. */
private const val BATTERY_PERCENT_FACTOR = 100

/** Reads the status now. */
private fun readStatus(context: Context): DeviceStatus {
    val battery = context.registerReceiver(null, IntentFilter(Intent.ACTION_BATTERY_CHANGED))
    val level = battery?.getIntExtra(BatteryManager.EXTRA_LEVEL, -1) ?: -1
    val scale = battery?.getIntExtra(BatteryManager.EXTRA_SCALE, DEFAULT_BATTERY_SCALE) ?: DEFAULT_BATTERY_SCALE
    val plugged = (battery?.getIntExtra(BatteryManager.EXTRA_PLUGGED, 0) ?: 0) != 0
    val connectivity = context.getSystemService(ConnectivityManager::class.java)
    val caps = connectivity.getNetworkCapabilities(connectivity.activeNetwork)
    return DeviceStatus(
        battery = if (level < 0) -1 else level * BATTERY_PERCENT_FACTOR / scale,
        charging = plugged,
        wifi = caps?.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) == true,
        time = DateFormat.getTimeFormat(context).format(Date()),
    )
}

/** The status, refreshed on every battery change and every minute tick. */
@Composable
fun rememberDeviceStatus(): DeviceStatus {
    val context = LocalContext.current
    var status by remember { mutableStateOf(readStatus(context)) }
    DisposableEffect(context) {
        val receiver = object : BroadcastReceiver() {
            /** Any of the subscribed broadcasts: read again. */
            override fun onReceive(c: Context, intent: Intent) {
                status = readStatus(c)
            }
        }
        val filter = IntentFilter().apply {
            addAction(Intent.ACTION_BATTERY_CHANGED)
            addAction(Intent.ACTION_TIME_TICK)
            addAction(Intent.ACTION_TIME_CHANGED)
            addAction(Intent.ACTION_TIMEZONE_CHANGED)
        }
        context.registerReceiver(receiver, filter)
        onDispose { context.unregisterReceiver(receiver) }
    }
    return status
}
