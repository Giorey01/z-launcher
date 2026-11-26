# The Rust core is reached through UniFFI over JNA, which finds classes, fields and methods by name at run
# time: keep JNA and the generated bindings whole.
-keep class com.sun.jna.** { *; }
-keepclassmembers class * extends com.sun.jna.** { *; }
-keep class dev.zlauncher.core.nativecore.** { *; }
-dontwarn java.awt.**
-dontwarn com.sun.jna.**

# Pods are found with ServiceLoader (META-INF/services/dev.zlauncher.pods.PodProvider): keep the providers and
# their no-argument constructors, which nothing calls directly.
-keep class * implements dev.zlauncher.pods.PodProvider { <init>(); }

# Pods (docs/spec/pods.md §9): a Pod's dex is compiled against the Pod SDK and the libraries the
# launcher provides (pod-platform). It is loaded at run time, so R8 cannot see what it uses: the
# whole contract and those libraries keep their names and members, or a Pod fails with
# ClassNotFoundException or NoSuchMethodError in release builds only. The Material icon set is not
# kept (it is most of the size, and the Pod style rule forbids it).
-keep class dev.zlauncher.pods.** { *; }
-keep class dev.zlauncher.ui.** { *; }
-keep class kotlin.** { *; }
-keep class kotlinx.coroutines.** { *; }
-keep class androidx.compose.runtime.** { *; }
-keep class androidx.compose.ui.** { *; }
-keep class androidx.compose.foundation.** { *; }
-keep class androidx.compose.animation.** { *; }
-keep class androidx.compose.material3.** { *; }
-keep class androidx.lifecycle.** { *; }
-keep class androidx.core.** { *; }
-keep class androidx.activity.** { *; }
-keep class coil.** { *; }
-keep class okhttp3.** { *; }
-keep class okio.** { *; }
