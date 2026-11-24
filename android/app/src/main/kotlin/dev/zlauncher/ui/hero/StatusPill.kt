package dev.zlauncher.ui.hero

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.BatteryChargingFull
import androidx.compose.material.icons.rounded.BatteryFull
import androidx.compose.material.icons.rounded.SignalWifi4Bar
import androidx.compose.material.icons.rounded.SignalWifiOff
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.zlauncher.ui.theme.LocalZTheme
import dev.zlauncher.ui.theme.ZPanel

/** The pill in the corner of a screen: Wi-Fi, battery with its percentage, and the clock, over a themed surface. */
@Composable
fun StatusPill(modifier: Modifier = Modifier) {
    val theme = LocalZTheme.current
    val status = rememberDeviceStatus()
    ZPanel(modifier, RoundedCornerShape(24.dp), elevation = 4.dp) {
        Row(
            Modifier.padding(horizontal = 14.dp, vertical = 10.dp),
            horizontalArrangement = Arrangement.spacedBy(10.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Icon(
                if (status.wifi) Icons.Rounded.SignalWifi4Bar else Icons.Rounded.SignalWifiOff,
                null,
                Modifier.size(22.dp),
                tint = theme.accentStart
            )
            if (status.battery >= 0) {
                val batteryIcon = if (status.charging) Icons.Rounded.BatteryChargingFull else Icons.Rounded.BatteryFull
                Icon(batteryIcon, null, Modifier.size(22.dp), tint = theme.accentStart)
                if (theme.settings.showBatteryPercentage) {
                    Text(
                        "${status.battery}%",
                        color = theme.textPrimary.copy(alpha = 0.8f),
                        fontSize = 13.sp,
                        fontWeight = FontWeight.SemiBold
                    )
                }
            }
            Text(
                status.time,
                color = theme.textPrimary.copy(alpha = 0.85f),
                fontSize = 17.sp,
                fontWeight = FontWeight.SemiBold
            )
        }
    }
}
