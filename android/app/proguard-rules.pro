# The Rust core is reached through UniFFI over JNA, which finds classes, fields and methods by name at run
# time: keep JNA and the generated bindings whole.
-keep class com.sun.jna.** { *; }
-keepclassmembers class * extends com.sun.jna.** { *; }
-keep class dev.zlauncher.core.nativecore.** { *; }
-dontwarn java.awt.**
-dontwarn com.sun.jna.**

# Pod interfaces and contracts
-keep class dev.zlauncher.pods.** { *; }

# Launcher UI contracts used by Pods
-keep class dev.zlauncher.ui.theme.LocalZTheme* { *; }
-keep class dev.zlauncher.ui.theme.ZTheme { *; }
-keep class dev.zlauncher.ui.theme.ZThemeKt { *; }
-keep class dev.zlauncher.ui.pad.PadButtonKt { *; }
-keep class dev.zlauncher.ui.pad.PadTileKt { *; }
-keep class dev.zlauncher.ui.pad.TileSurface { *; }

# Compose APIs used by Pods
-keep class androidx.compose.runtime.** { *; }
-keep class androidx.compose.ui.** { *; }
-keep class androidx.compose.foundation.** { *; }
-keep class androidx.compose.material3.TextKt { *; }
# Material icons used by pods
-keep class androidx.compose.material.icons.Icons$Rounded { *; }
-keep class androidx.compose.material.icons.rounded.StarKt { *; }

# Kotlin stdlib and coroutines for dynamic dex compatibility
-keep class kotlin.** { *; }
-keep class kotlinx.coroutines.** { *; }
