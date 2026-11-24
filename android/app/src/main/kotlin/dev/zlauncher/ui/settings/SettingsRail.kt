package dev.zlauncher.ui.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Folder
import androidx.compose.material.icons.rounded.Groups
import androidx.compose.material.icons.rounded.Info
import androidx.compose.material.icons.rounded.Palette
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material.icons.rounded.Settings
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.zlauncher.R
import dev.zlauncher.nav.AppViewModel
import dev.zlauncher.ui.hero.StatusPill
import dev.zlauncher.ui.pad.FocusCursor
import dev.zlauncher.ui.theme.LocalBackdrop
import dev.zlauncher.ui.theme.LocalZTheme
import dev.zlauncher.ui.theme.ZPanel

/** The icon of a category matching Cocoon. */
fun Category.icon(): ImageVector = when (this) {
    Category.PERSONALIZATION -> Icons.Rounded.Palette
    Category.LIBRARY -> Icons.Rounded.Folder
    Category.INTEGRATIONS -> Icons.Rounded.Groups
    Category.SYSTEM -> Icons.Rounded.Settings
    Category.ABOUT -> Icons.Rounded.Info
}

/** The name of a category. */
@Composable
fun Category.title(): String = stringResource(
    when (this) {
        Category.PERSONALIZATION -> R.string.st_cat_personalization
        Category.LIBRARY -> R.string.st_cat_library
        Category.INTEGRATIONS -> R.string.st_cat_integrations
        Category.SYSTEM -> R.string.st_cat_system
        Category.ABOUT -> R.string.st_cat_about
    },
)

/** The one-line description of a category. */
@Composable
fun Category.subtitle(): String = stringResource(
    when (this) {
        Category.PERSONALIZATION -> R.string.st_cat_personalization_sub
        Category.LIBRARY -> R.string.st_cat_library_sub
        Category.INTEGRATIONS -> R.string.st_cat_integrations_sub
        Category.SYSTEM -> R.string.st_cat_system_sub
        Category.ABOUT -> R.string.st_cat_about_sub
    },
)

/** Search pill at the top of the category rail matching Cocoon. */
@Composable
fun SearchPill(modifier: Modifier = Modifier) {
    val theme = LocalZTheme.current
    Box(
        modifier = modifier
            .fillMaxWidth()
            .height(44.dp)
            .background(theme.textPrimary.copy(alpha = 0.08f), RoundedCornerShape(22.dp))
            .padding(horizontal = 16.dp),
        contentAlignment = Alignment.CenterStart,
    ) {
        Text(
            text = stringResource(R.string.st_search),
            color = theme.textPrimary.copy(alpha = 0.5f),
            fontSize = 15.sp,
            fontWeight = FontWeight.Medium,
        )
        Icon(
            imageVector = Icons.Rounded.Search,
            contentDescription = null,
            tint = theme.textPrimary.copy(alpha = 0.4f),
            modifier = Modifier.size(18.dp).align(Alignment.CenterEnd),
        )
    }
}

/**
 * A category item of the rail. The selected one is a glass card with its description and a title in the accent colour;
 * the cursor brackets wrap it while the rail has the cursor. The others are plain rows.
 */
@Composable
fun RailItem(category: Category, selected: Boolean, cursor: Boolean) {
    val theme = LocalZTheme.current
    val shape = RoundedCornerShape(18.dp)
    val content: @Composable BoxScope.() -> Unit = {
        Row(Modifier.fillMaxSize().padding(start = 15.dp), verticalAlignment = Alignment.CenterVertically) {
            val iconTint = if (selected) theme.accentStart else theme.iconTint.copy(alpha = 0.6f)
            Icon(category.icon(), null, Modifier.size(30.dp), tint = iconTint)
            Column(Modifier.padding(start = 16.dp)) {
                val titleColor = if (selected) theme.accentStart else theme.textPrimary.copy(alpha = 0.75f)
                Text(category.title(), color = titleColor, fontSize = 16.sp, fontWeight = FontWeight.SemiBold, maxLines = 1)
                if (selected) {
                    Text(
                        category.subtitle(),
                        color = theme.textPrimary.copy(alpha = 0.55f),
                        fontSize = 12.sp,
                        fontWeight = FontWeight.Medium,
                        maxLines = 1,
                    )
                }
            }
        }
    }
    if (selected) {
        Box(Modifier.fillMaxWidth().height(73.dp)) {
            ZPanel(Modifier.fillMaxSize(), shape, content = content)
            if (cursor) FocusCursor(Modifier.matchParentSize(), 5.dp)
        }
    } else {
        Box(Modifier.fillMaxWidth().height(60.dp), content = content)
    }
}

/**
 * The grid display while the settings are open (dual-screen mode): a centered settings emblem,
 * the app name and version pill under it, and the status pill in the top-right corner.
 */
@Composable
fun SettingsAside(app: AppViewModel) {
    val theme = LocalZTheme.current
    CompositionLocalProvider(LocalBackdrop provides null) {
        Box(Modifier.fillMaxSize().background(theme.backgroundBrush())) {
            val asideModifier = Modifier
                .padding(start = 16.dp, end = 16.dp, top = 66.dp, bottom = 8.dp)
                .fillMaxSize()
                .background(theme.textPrimary.copy(alpha = 0.10f), RoundedCornerShape(20.dp))
            Box(asideModifier) {
                Column(
                    Modifier.align(Alignment.Center),
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.spacedBy(22.dp),
                ) {
                    Icon(
                        imageVector = Icons.Rounded.Settings,
                        contentDescription = null,
                        modifier = Modifier.size(160.dp),
                        tint = theme.textPrimary.copy(alpha = 0.5f),
                    )
                    VersionCard(app)
                }
            }
            StatusPill(Modifier.align(Alignment.TopEnd).padding(top = 10.dp, end = 16.dp))
        }
    }
}

/** The pill with the app name and version. */
@Composable
private fun VersionCard(app: AppViewModel) {
    val theme = LocalZTheme.current
    val version = app.zl.packageManager.getPackageInfo(app.zl.packageName, 0).versionName.orEmpty()
    ZPanel(Modifier, RoundedCornerShape(18.dp), elevation = 6.dp) {
        Column(Modifier.padding(horizontal = 26.dp, vertical = 14.dp), horizontalAlignment = Alignment.CenterHorizontally) {
            Text(stringResource(R.string.app_name), color = theme.textPrimary.copy(alpha = 0.85f), fontSize = 19.sp, fontWeight = FontWeight.Bold)
            Text(
                stringResource(R.string.st_about_version, version),
                color = theme.textPrimary.copy(alpha = 0.6f),
                fontSize = 12.sp,
                fontWeight = FontWeight.SemiBold,
            )
        }
    }
}
