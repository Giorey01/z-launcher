package dev.zlauncher.ui.hero

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.produceState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.zlauncher.ZLauncherApp
import dev.zlauncher.core.nativecore.FolderArt
import dev.zlauncher.core.nativecore.GameEntry
import dev.zlauncher.core.nativecore.HeroDisplayStyle
import dev.zlauncher.core.nativecore.HeroGradientStyle
import dev.zlauncher.nav.HeroTarget
import dev.zlauncher.ui.art.ArtImage
import dev.zlauncher.ui.theme.LocalZTheme

/** Width of the platform hero picture as a fraction of the area, at `hero_folder_logo_scale` 1. */
private const val FOLDER_HERO_WIDTH = 0.42f

/** Width of the game logo as a fraction of the area, at `hero_logo_scale` 1. */
private const val GAME_LOGO_WIDTH = 0.68f

/** Width of the info box as a fraction of the area. */
private const val INFO_WIDTH = 0.206f

/**
 * The hero area: for a platform folder the Smart Folder hero picture with an info box (logo and name) under it;
 * for a game its hero image behind (shape and fade from the settings) and its logo large and centred.
 * Draws nothing for [HeroTarget.None], so the wallpaper shows.
 */
@Composable
fun HeroArea(target: HeroTarget, modifier: Modifier = Modifier) {
    Box(modifier.fillMaxSize()) {
        when (target) {
            HeroTarget.None -> Unit
            is HeroTarget.Platform -> PlatformHero(target.id, target.name)
            is HeroTarget.Game -> GameHero(target.game)
        }
    }
}

/** The theme art of a platform folder through the core cache. */
@Composable
private fun rememberFolderArt(id: String): FolderArt? {
    val art = (LocalContext.current.applicationContext as ZLauncherApp).art
    val revision = LocalZTheme.current.revision
    return produceState<FolderArt?>(null, id, revision) { value = art.platform(id) }.value
}

/** A platform: picture in the middle and an info box with its logo and name under it. */
@Composable
private fun PlatformHero(id: String, name: String) {
    val theme = LocalZTheme.current
    val art = rememberFolderArt(id)
    BoxWithConstraints(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
        val areaWidth = maxWidth
        val areaHeight = maxHeight
        Column(
            Modifier
                .width(areaWidth * FOLDER_HERO_WIDTH * theme.settings.heroFolderLogoScale * 2f)
                .padding(bottom = 8.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            art?.hero?.let { ArtImage(it, Modifier.fillMaxWidth().height(areaHeight * 0.34f), ContentScale.Fit) }
            InfoBox(name, art?.logo, Modifier.width(areaWidth * INFO_WIDTH * 1.35f))
        }
    }
}

/** The info card of a platform: its logo at left if present, name at right. */
@Composable
private fun InfoBox(name: String, logo: String?, modifier: Modifier = Modifier) {
    Row(
        modifier
            .height(46.dp)
            .clip(RoundedCornerShape(8.dp))
            .background(Color.White.copy(alpha = 0.95f))
            .padding(4.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        if (logo != null) {
            ArtImage(logo, Modifier.fillMaxHeight().width(56.dp).padding(horizontal = 4.dp), ContentScale.Fit)
        }
        Text(
            name,
            color = Color.Black,
            fontSize = 12.sp,
            fontWeight = FontWeight.Bold,
            maxLines = 1,
            modifier = Modifier.padding(horizontal = 8.dp),
        )
    }
}

/** A game: hero image behind (when the style wants it) and the logo large in the centre. */
@Composable
private fun GameHero(game: GameEntry) {
    val theme = LocalZTheme.current
    Box(Modifier.fillMaxSize()) {
        val style = theme.settings.heroDisplayStyle
        if (style != HeroDisplayStyle.NONE) game.hero?.let { HeroBackdrop(it, style) }
        if (theme.settings.heroGradientStyle == HeroGradientStyle.BLACK_GRADIENT && game.hero != null) {
            Box(Modifier.fillMaxSize().background(Brush.verticalGradient(listOf(Color.Transparent, Color.Black.copy(alpha = 0.55f)))))
        }
        BoxWithConstraints(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            val logo = game.logo
            val areaWidth = maxWidth
            if (logo != null) {
                ArtImage(logo, Modifier.width(areaWidth * GAME_LOGO_WIDTH * theme.settings.heroLogoScale).fillMaxHeight(0.5f), ContentScale.Fit)
            } else {
                Text(
                    text = game.displayTitle,
                    modifier = Modifier.width(areaWidth * 0.6f),
                    color = Color.White,
                    fontSize = 30.sp,
                    fontWeight = FontWeight.Bold,
                    textAlign = TextAlign.Center,
                    maxLines = 3,
                )
            }
        }
    }
}

/** The hero image of a game, full-bleed; the vignette style fades its edges into the wallpaper. */
@Composable
private fun HeroBackdrop(path: String, style: HeroDisplayStyle) {
    val theme = LocalZTheme.current
    Box(Modifier.fillMaxSize()) {
        ArtImage(path, Modifier.fillMaxSize(), ContentScale.Crop)
        if (style == HeroDisplayStyle.VIGNETTE) {
            val strength = theme.settings.heroVignetteSize.coerceIn(0f, 1f)
            Box(Modifier.fillMaxSize().background(Brush.radialGradient(0.55f to Color.Transparent, 1f to Color.Black.copy(alpha = 0.35f + 0.5f * strength))))
        }
    }
}
