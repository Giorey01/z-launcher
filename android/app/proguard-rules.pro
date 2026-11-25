# The Rust core is reached through UniFFI over JNA, which finds classes, fields and methods by name at run
# time: keep JNA and the generated bindings whole.
-keep class com.sun.jna.** { *; }
-keepclassmembers class * extends com.sun.jna.** { *; }
-keep class dev.zlauncher.core.nativecore.** { *; }
-dontwarn java.awt.**
-dontwarn com.sun.jna.**
