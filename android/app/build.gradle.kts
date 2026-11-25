import java.util.Properties

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.compose)
    alias(libs.plugins.zl.detekt)
}

android {
    namespace = "dev.zlauncher"
    compileSdk = 35
    defaultConfig {
        applicationId = "dev.zlauncher"
        minSdk = 29
        targetSdk = 35
        versionCode = 1
        versionName = "0.1.0"
        // Only the languages the app is translated into.
        resourceConfigurations += listOf("en", "it")
        // The Rust core (core/native) is built for arm64 only.
        ndk { abiFilters += "arm64-v8a" }
    }
    // `oss` is the default build. `full` adds nothing yet: the dimension exists so a build with
    // extra modules can add its own sources and dependencies to that flavor without touching `oss`.
    flavorDimensions += "distribution"
    productFlavors {
        create("oss") { dimension = "distribution"; isDefault = true }
        create("full") { dimension = "distribution" }
    }
    // Release signing: the keystore stays outside the repository. Its properties file (storeFile, storePassword,
    // keyAlias, keyPassword) is read from ZL_KEYSTORE_PROPERTIES or ~/.config/z-launcher/keystore.properties;
    // without it the release build is left unsigned.
    val keystoreFile = (System.getenv("ZL_KEYSTORE_PROPERTIES") ?: "${System.getProperty("user.home")}/.config/z-launcher/keystore.properties")
        .let(::file)
    if (keystoreFile.isFile) {
        val props = Properties().apply { keystoreFile.inputStream().use(::load) }
        signingConfigs {
            create("release") {
                storeFile = file(props.getProperty("storeFile"))
                storePassword = props.getProperty("storePassword")
                keyAlias = props.getProperty("keyAlias")
                keyPassword = props.getProperty("keyPassword")
            }
        }
        buildTypes.getByName("release").signingConfig = signingConfigs.getByName("release")
    }
    buildTypes.getByName("release") {
        // R8 drops the unused code (most of the Material icon set); see proguard-rules.pro for what must stay.
        isMinifyEnabled = true
        isShrinkResources = true
        proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    buildFeatures { compose = true }
    // libzl_ffi.so, built by :core:native (cargo-ndk) before the native libraries are merged.
    sourceSets.getByName("main").jniLibs.srcDir(project(":core:native").layout.buildDirectory.dir("jniLibs"))
}

tasks.configureEach {
    if (name.startsWith("merge") && name.endsWith("JniLibFolders")) dependsOn(":core:native:cargoNdkArm64")
}

kotlin { compilerOptions { jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17) } }

dependencies {
    implementation(project(":core:native"))
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.activity.compose)
    implementation(platform(libs.compose.bom))
    implementation(libs.compose.ui)
    implementation(libs.compose.foundation)
    implementation(libs.compose.material3)
    implementation(libs.compose.icons.extended)
    implementation(libs.coil.compose)
    implementation(libs.coil.gif)
    implementation(libs.media3.exoplayer)
    // Store trailers are streamed; newer store pages only offer an HLS playlist.
    implementation(libs.media3.exoplayer.hls)
    implementation(libs.media3.ui)
    implementation(libs.androidx.lifecycle.viewmodel.compose)
    implementation(libs.androidx.lifecycle.runtime.compose)
    implementation(libs.kotlinx.coroutines.android)
    // JNA with its Android runtime (libjnidispatch.so): the UniFFI bindings of the Rust core load through it.
    implementation(libs.jna) { artifact { type = "aar" } }
    testImplementation(libs.junit)
    testImplementation(libs.jna)
    testImplementation(libs.kotlinx.coroutines.core)
    testImplementation(libs.org.json)
}

// JVM tests run against the real Rust library built for this machine (cargo build, release-ffi):
// the bindings find it through jna.library.path. `cargo` and the Rust toolchain must be installed.
tasks.withType<Test>().configureEach {
    dependsOn(":core:native:cargoHost")
    systemProperty("jna.library.path", project(":core:native").layout.buildDirectory.dir("hostLib").get().asFile.absolutePath)
    // The workspace manifest, so the test compares against the real version instead of a copy of it.
    inputs.dir(rootProject.projectDir.resolve("../fixtures"))
    // The shared fixtures every target reads (Rust tests read the same files).
    systemProperty("zl.fixturesDir", rootProject.projectDir.resolve("../fixtures").absolutePath)
    systemProperty("zl.workspaceManifest", rootProject.projectDir.resolve("../Cargo.toml").absolutePath)
}
