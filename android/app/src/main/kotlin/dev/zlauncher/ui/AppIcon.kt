package dev.zlauncher.ui

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.produceState
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.Dp
import dev.zlauncher.ZLauncherApp

/** The icon of an installed app, decoded in the background (an empty box until it is ready). */
@Composable
fun AppIcon(packageName: String, size: Dp, modifier: Modifier = Modifier) {
    val apps = (LocalContext.current.applicationContext as ZLauncherApp).apps
    val bitmap by produceState<ImageBitmap?>(null, packageName) { value = apps.icon(packageName) }
    val box = modifier.size(size)
    val icon = bitmap
    if (icon == null) Box(box) else Image(icon, contentDescription = null, modifier = box)
}
