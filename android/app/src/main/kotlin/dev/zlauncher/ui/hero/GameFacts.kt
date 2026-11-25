package dev.zlauncher.ui.hero

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.produceState
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.zlauncher.ZLauncherApp
import dev.zlauncher.core.nativecore.CoreException
import dev.zlauncher.core.nativecore.GameDetails

/** Widest the facts card gets. */
private val FACTS_MAX_WIDTH = 720.dp

/** Lines of the description shown under the facts. */
private const val DESCRIPTION_LINES = 3

/** Digits of a year at the start of a release date. */
private const val YEAR_DIGITS = 4

/** Opacity of the card behind the text, so it reads over any hero image. */
private const val CARD_ALPHA = 0.55f

/**
 * What the library knows about a game, under its logo: year, developer and genre on one line, then the start of the
 * description. Nothing is drawn when the library has none of them (no placeholders).
 */
@Composable
fun GameFacts(gameId: Long, modifier: Modifier = Modifier) {
    val core = (LocalContext.current.applicationContext as ZLauncherApp).core
    val details = produceState<GameDetails?>(null, gameId) {
        value = try {
            core.call { it.gameDetails(gameId) }
        } catch (_: CoreException) {
            null
        }
    }.value ?: return
    val line = factsLine(details)
    val description = details.description?.trim()?.takeIf { it.isNotEmpty() }
    if (line == null && description == null) return
    Column(
        modifier.widthIn(max = FACTS_MAX_WIDTH)
            .background(Color.Black.copy(alpha = CARD_ALPHA), RoundedCornerShape(14.dp))
            .padding(horizontal = 20.dp, vertical = 14.dp),
        verticalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        line?.let { Text(it, color = Color.White, fontSize = 18.sp, fontWeight = FontWeight.SemiBold, maxLines = 1) }
        description?.let {
            Text(
                it,
                color = Color.White.copy(alpha = 0.85f),
                fontSize = 15.sp,
                maxLines = DESCRIPTION_LINES,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

/** "2011 · HAL Laboratory · Platformer" from the fields that are known, `null` when none is. */
private fun factsLine(d: GameDetails): String? {
    val year = d.releaseDate?.take(YEAR_DIGITS)?.takeIf { it.length == YEAR_DIGITS && it.all(Char::isDigit) }
    val parts = listOfNotNull(year, d.developer, d.genres).map { it.trim() }.filter { it.isNotEmpty() }
    return parts.takeIf { it.isNotEmpty() }?.joinToString(" · ")
}
