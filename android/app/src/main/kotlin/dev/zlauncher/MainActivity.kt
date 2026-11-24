package dev.zlauncher

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.BackHandler
import androidx.activity.compose.setContent
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import dev.zlauncher.core.nativecore.coreVersion

/** The only activity: the launcher's home screen, which can also be the device Home. */
class MainActivity : ComponentActivity() {
    /** Draws the home screen. */
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent { MaterialTheme { HomeScreen() } }
    }
}

/** A minimal screen showing the version of the Rust core; BACK is swallowed so Home never exits. */
@Composable
private fun HomeScreen() {
    // As the device Home there is nothing behind this screen to go back to.
    BackHandler(enabled = true) {}
    Box(Modifier.fillMaxSize().background(Color.Black), contentAlignment = Alignment.Center) {
        Text(stringResource(R.string.core_version_label, coreVersion()), color = Color.White)
    }
}
