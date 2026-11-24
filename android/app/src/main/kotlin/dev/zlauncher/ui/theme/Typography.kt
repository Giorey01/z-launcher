package dev.zlauncher.ui.theme

import androidx.compose.material3.Typography
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import dev.zlauncher.R

/** The interface font: Poppins (SIL Open Font License, see NOTICE). */
val Poppins = FontFamily(
    Font(R.font.poppins_regular, FontWeight.Normal),
    Font(R.font.poppins_medium, FontWeight.Medium),
    Font(R.font.poppins_semibold, FontWeight.SemiBold),
    Font(R.font.poppins_bold, FontWeight.Bold),
)

/** Material typography with Poppins everywhere, so plain `Text` already looks right. */
val ZTypography: Typography = Typography().let { base ->
    fun TextStyle.poppins() = copy(fontFamily = Poppins)
    Typography(
        displayLarge = base.displayLarge.poppins(),
        displayMedium = base.displayMedium.poppins(),
        displaySmall = base.displaySmall.poppins(),
        headlineLarge = base.headlineLarge.poppins(),
        headlineMedium = base.headlineMedium.poppins(),
        headlineSmall = base.headlineSmall.poppins(),
        titleLarge = base.titleLarge.poppins(),
        titleMedium = base.titleMedium.poppins(),
        titleSmall = base.titleSmall.poppins(),
        bodyLarge = base.bodyLarge.poppins(),
        bodyMedium = base.bodyMedium.poppins(),
        bodySmall = base.bodySmall.poppins(),
        labelLarge = base.labelLarge.poppins(),
        labelMedium = base.labelMedium.poppins(),
        labelSmall = base.labelSmall.poppins(),
    )
}
