package dev.zlauncher.ui.home

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.produceState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.zlauncher.ZLauncherApp
import dev.zlauncher.core.nativecore.FolderArt
import dev.zlauncher.core.nativecore.GameEntry
import dev.zlauncher.core.nativecore.IconStyle
import dev.zlauncher.core.nativecore.OverlayArt
import dev.zlauncher.core.nativecore.PlatformEntry
import dev.zlauncher.ui.art.ArtImage
import dev.zlauncher.ui.art.CoverArt
import dev.zlauncher.ui.art.GreyMatrix
import dev.zlauncher.ui.pad.PadTile
import dev.zlauncher.ui.pad.TileSurface
import dev.zlauncher.ui.theme.LocalZTheme
import dev.zlauncher.ui.theme.ZTheme

/** Where the cursor box sits relative to a platform cell: inset by this much of the cell. */
private val PLATFORM_CURSOR = (-11).dp

/** Where the cursor box sits relative to a game cell. */
private val GAME_CURSOR = (-6).dp

/** Fraction of a cell the platform icon takes (the rest is room for the cursor). */
private const val PLATFORM_ICON_FRACTION = 0.82f

/** Fraction of a cell the game cover takes. */
private const val GAME_COVER_FRACTION = 0.78f

/** The art of one platform folder, loaded through the core (cached per theme). */
@Composable
private fun rememberPlatformArt(id: String, theme: ZTheme): FolderArt? {
    val art = (LocalContext.current.applicationContext as ZLauncherApp).art
    return produceState<FolderArt?>(null, id, theme.revision) { value = art.platform(id) }.value
}

/** The cover mask and overlay of one platform, loaded through the core (cached per theme). */
@Composable
fun rememberOverlay(platformId: String, theme: ZTheme): OverlayArt? {
    val art = (LocalContext.current.applicationContext as ZLauncherApp).art
    return produceState<OverlayArt?>(null, platformId, theme.revision) { value = art.overlay(platformId) }.value
}

/**
 * The folder of one platform: the icon from the theme's Smart Folder pack (animated GIF/WebP supported), with
 * no text label. In the ACCENT icon style the unfocused icons are grey and the focused one has its colours;
 * FLAT draws them in one colour. A platform without art shows its name on a card.
 */
@Composable
fun PlatformTile(
    platform: PlatformEntry,
    onOpen: () -> Unit,
    modifier: Modifier = Modifier,
    onFocused: () -> Unit = {},
) {
    val theme = LocalZTheme.current
    val art = rememberPlatformArt(platform.id, theme)
    val shape = RoundedCornerShape(theme.settings.iconRoundness.dpPercent())
    PadTile(
        onOpen,
        modifier.fillMaxWidth().aspectRatio(1f),
        shape = shape,
        surface = TileSurface.NONE,
        cursorOutset = PLATFORM_CURSOR,
        onFocused = onFocused
    ) { focused ->
        val icon = art?.icon
        Box(Modifier.align(Alignment.Center).fillMaxSize(PLATFORM_ICON_FRACTION)) {
            val filter = if (theme.settings.iconStyle == IconStyle.ACCENT && !focused) GreyMatrix else null
            if (icon != null) {
                ArtImage(icon, Modifier.fillMaxSize(), colorFilter = filter)
            } else {
                NameCard(platform.name)
            }
        }
    }
}

/** Maximum corner roundness fraction allowed by [RoundedCornerShape]. */
private const val MAX_ROUNDNESS_FRACTION = 0.5f

/** One hundred, for percentages. */
private const val PERCENT = 100

/** Rounds a 0 to 0.5 roundness to the percent of a side that [RoundedCornerShape] takes. */
private fun Float.dpPercent(): Int = (coerceIn(0f, MAX_ROUNDNESS_FRACTION) * PERCENT).toInt()

/** The name of a platform on a card: what is drawn when the theme has no icon for it. */
@Composable
private fun BoxScope.NameCard(name: String) {
    val theme = LocalZTheme.current
    val cardShape = RoundedCornerShape(16.dp)
    Box(
        Modifier
            .fillMaxSize()
            .clip(cardShape)
            .background(theme.cardBrush())
            .border(1.dp, theme.tileBorder, cardShape),
        contentAlignment = Alignment.Center
    ) {
        Text(
            name,
            Modifier.padding(8.dp),
            color = theme.textPrimary,
            fontSize = 15.sp,
            fontWeight = FontWeight.SemiBold,
            textAlign = TextAlign.Center,
            maxLines = 3,
            overflow = TextOverflow.Ellipsis
        )
    }
}

/**
 * A game in the grid: its cover, cut and framed by the platform's icon overlay; the Y button opens the
 * menu through [onMenu]. A game without a cover shows its title.
 */
@Composable
fun GameTile(
    game: GameEntry,
    onLaunch: () -> Unit,
    onMenu: () -> Unit,
    modifier: Modifier = Modifier,
    onFocused: () -> Unit = {},
) {
    val theme = LocalZTheme.current
    val overlay = rememberOverlay(game.platformId, theme)
    PadTile(
        onLaunch,
        modifier.fillMaxWidth().aspectRatio(1f),
        onSecondary = onMenu,
        surface = TileSurface.NONE,
        cursorOutset = GAME_CURSOR,
        onFocused = onFocused
    ) { focused ->
        CoverArt(
            cover = game.icon,
            overlay = overlay,
            roundness = theme.settings.iconRoundness,
            modifier = Modifier.align(Alignment.Center).fillMaxSize(GAME_COVER_FRACTION),
            colorFilter = if (theme.settings.iconStyle == IconStyle.ACCENT && !focused) GreyMatrix else null,
        ) { TitleFallback(game.displayTitle) }
    }
}

/** The title of a game on a card, for games without a cover. */
@Composable
private fun TitleFallback(title: String) {
    val theme = LocalZTheme.current
    val shape = RoundedCornerShape(theme.settings.iconRoundness.dpPercent())
    Box(
        Modifier
            .fillMaxSize()
            .clip(shape)
            .background(theme.cardBrush())
            .border(1.5.dp, Color.White.copy(alpha = 0.3f), shape),
        contentAlignment = Alignment.Center
    ) {
        Text(
            title,
            Modifier.padding(6.dp),
            color = theme.textPrimary,
            fontSize = 11.sp,
            fontWeight = FontWeight.Medium,
            textAlign = TextAlign.Center,
            maxLines = 4,
            overflow = TextOverflow.Ellipsis
        )
    }
}

/** An empty slot of the grid: a faint rounded outline where no game is. */
@Composable
fun EmptySlot(modifier: Modifier = Modifier) {
    val shape = RoundedCornerShape(LocalZTheme.current.settings.iconRoundness.dpPercent())
    Box(modifier.fillMaxWidth().aspectRatio(1f), contentAlignment = Alignment.Center) {
        Box(
            Modifier
                .fillMaxSize(GAME_COVER_FRACTION)
                .clip(shape)
                .background(Color.Black.copy(alpha = 0.22f))
                .border(2.5.dp, Color.Black.copy(alpha = 0.35f), shape)
        )
    }
}
