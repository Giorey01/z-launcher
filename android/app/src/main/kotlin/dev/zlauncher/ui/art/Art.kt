package dev.zlauncher.ui.art

import android.graphics.BitmapFactory
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.graphics.BlendMode
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.ColorMatrix
import androidx.compose.ui.graphics.CompositingStrategy
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.IntSize
import coil.compose.AsyncImage
import coil.request.ImageRequest
import dev.zlauncher.core.nativecore.OverlayArt
import java.io.File
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

/** An image file (PNG, JPG, WebP, animated GIF/WebP through the Coil decoders) that fills its box. */
@Composable
fun ArtImage(
    path: String?,
    modifier: Modifier = Modifier,
    contentScale: ContentScale = ContentScale.Fit,
    colorFilter: ColorFilter? = null,
    alignment: Alignment = Alignment.Center,
) {
    if (path == null) return
    val context = LocalContext.current
    val request = remember(path) { ImageRequest.Builder(context).data(File(path)).crossfade(false).build() }
    AsyncImage(request, null, modifier, alignment = alignment, contentScale = contentScale, colorFilter = colorFilter)
}

/** A still bitmap decoded off the main thread (masks and overlays are small PNGs); `null` while loading or when unreadable. */
@Composable
fun rememberArtBitmap(path: String?): ImageBitmap? {
    val state = produceState<ImageBitmap?>(null, path) {
        value = path?.let { withContext(Dispatchers.IO) { BitmapFactory.decodeFile(it)?.asImageBitmap() } }
    }
    return state.value
}

/** Colour matrix that removes the colour and dims a little: unfocused icons in the ACCENT icon style. */
val GreyMatrix: ColorFilter = ColorFilter.colorMatrix(ColorMatrix().apply { setToSaturation(0f) })

/**
 * A cover drawn the way the theme says: the image, cut by the platform's mask (alpha), the overlay
 * frame on top. Without a mask the cover gets plain rounded corners ([roundness] is the fraction of
 * the side, 0 to 0.5). [fallback] draws when the game has no cover.
 */
@Composable
fun CoverArt(
    cover: String?,
    overlay: OverlayArt?,
    roundness: Float,
    modifier: Modifier = Modifier,
    colorFilter: ColorFilter? = null,
    fallback: @Composable () -> Unit = {},
) {
    val mask = rememberArtBitmap(overlay?.mask)
    val frame = rememberArtBitmap(overlay?.overlay)
    val hasMask = overlay?.mask != null
    Box(modifier) {
        Box(
            Modifier.fillMaxSize()
                .then(if (hasMask) Modifier else Modifier.clip(RoundedCornerShape(percentOf(roundness))))
                .graphicsLayer(compositingStrategy = CompositingStrategy.Offscreen)
                .drawWithContent {
                    drawContent()
                    mask?.let { drawImage(it, dstSize = IntSize(size.width.toInt(), size.height.toInt()), blendMode = BlendMode.DstIn) }
                },
        ) {
            if (cover == null) fallback() else ArtImage(cover, Modifier.fillMaxSize(), ContentScale.Crop, colorFilter)
        }
        frame?.let { Image(it, null, Modifier.fillMaxSize(), contentScale = ContentScale.FillBounds, colorFilter = colorFilter) }
    }
}

/** Maximum corner roundness fraction allowed by [RoundedCornerShape]. */
private const val MAX_ROUNDNESS_FRACTION = 0.5f

/** Factor to convert fractional roundness to percentage. */
private const val PERCENT_SCALE = 100f

/** Corner size as a percentage of the side, clamped to what [RoundedCornerShape] takes. */
private fun percentOf(roundness: Float): Int = (roundness.coerceIn(0f, MAX_ROUNDNESS_FRACTION) * PERCENT_SCALE).toInt()
