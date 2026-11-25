import org.jetbrains.kotlin.gradle.tasks.KotlinCompile
import java.util.zip.ZipEntry
import java.util.zip.ZipOutputStream
import java.io.FileOutputStream
import java.io.FileInputStream

plugins {
    id("com.android.library")
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.compose)
}

android {
    namespace = "dev.zlauncher.podspike"
    compileSdk = 35
    defaultConfig {
        minSdk = 29
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    buildFeatures { compose = true }
}

kotlin { compilerOptions { jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17) } }

dependencies {
    compileOnly(platform(libs.compose.bom))
    compileOnly(libs.compose.ui)
    compileOnly(libs.compose.foundation)
    compileOnly(libs.compose.material3)
    compileOnly(libs.compose.icons.extended)
}

val appClasses = files(
    rootProject.projectDir.resolve("app/build/tmp/kotlin-classes/ossRelease"),
    rootProject.projectDir.resolve("app/build/intermediates/javac/ossRelease/compileOssReleaseJavaWithJavac/classes")
)

tasks.withType<KotlinCompile>().configureEach {
    libraries.from(appClasses)
}

val packageSpikePod = tasks.register("packageSpikePod") {
    dependsOn("compileReleaseKotlin")
    val outputDir = layout.buildDirectory.dir("outputs/pod").get().asFile
    val zlpodFile = File(outputDir, "sample.spike.zlpod")
    outputs.file(zlpodFile)

    doLast {
        outputDir.mkdirs()
        val ndkDir = File(System.getenv("ANDROID_NDK_HOME") ?: "${System.getenv("ANDROID_HOME")}/ndk/27.2.12479018")
        val clang = File(ndkDir, "toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android33-clang")
        val nativeDir = File(outputDir, "lib/arm64-v8a").apply { mkdirs() }
        val nativeSo = File(nativeDir, "libspike_native.so")

        // 1. Compile C native library
        exec {
            commandLine(
                clang.absolutePath,
                "-shared", "-fPIC", "-O2",
                "-o", nativeSo.absolutePath,
                projectDir.resolve("src/main/c/spike_native.c").absolutePath
            )
        }

        // 2. Run d8
        val dexDir = File(outputDir, "dex").apply { mkdirs() }
        val sdkDir = File(System.getenv("ANDROID_HOME") ?: "${System.getProperty("user.home")}/Android/Sdk")
        val d8 = File(sdkDir, "build-tools/35.0.0/d8")
        val androidJar = File(sdkDir, "platforms/android-35/android.jar")
        val classes = fileTree(layout.buildDirectory.dir("tmp/kotlin-classes/release")).matching {
            include("dev/zlauncher/podspike/*.class")
        }.files

        exec {
            val args = mutableListOf(
                d8.absolutePath,
                "--min-api", "29",
                "--lib", androidJar.absolutePath,
                "--classpath", rootProject.projectDir.resolve("app/build/tmp/kotlin-classes/ossRelease").absolutePath,
                "--output", dexDir.absolutePath
            )
            classes.forEach { args.add(it.absolutePath) }
            commandLine(args)
        }

        // 3. Package .zlpod zip
        val dexFile = File(dexDir, "classes.dex")
        val manifestFile = projectDir.resolve("manifest.json")
        ZipOutputStream(FileOutputStream(zlpodFile)).use { zos ->
            fun addFile(file: File, entryPath: String) {
                zos.putNextEntry(ZipEntry(entryPath))
                FileInputStream(file).use { it.copyTo(zos) }
                zos.closeEntry()
            }
            addFile(manifestFile, "manifest.json")
            addFile(dexFile, "classes.dex")
            addFile(nativeSo, "lib/arm64-v8a/libspike_native.so")
        }
        println("Generated spike pod: ${zlpodFile.absolutePath} (${zlpodFile.length()} bytes)")
    }
}
