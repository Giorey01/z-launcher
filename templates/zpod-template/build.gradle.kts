plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.compose)
    alias(libs.plugins.detekt)
}

android {
    namespace = "dev.example.pod"
    compileSdk = 35

    defaultConfig {
        minSdk = 29
        consumerProguardFiles("consumer-rules.pro")
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    buildFeatures {
        compose = true
    }
}

kotlin {
    compilerOptions {
        jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17)
    }
}

detekt {
    config.setFrom(files("detekt.yml"))
    buildUponDefaultConfig = true
}

dependencies {
    // Dipendenze fornite dalla piattaforma Z-Launcher Pod SDK
    compileOnly(project(":pod-api"))
    compileOnly(project(":pod-ui"))
    compileOnly(platform(project(":pod-platform")))

    compileOnly(libs.compose.ui)
    compileOnly(libs.compose.foundation)
    compileOnly(libs.compose.material3)
    compileOnly(libs.kotlinx.coroutines.android)
    compileOnly(libs.androidx.core.ktx)

    detektPlugins(libs.detekt.formatting)
}

// Task per preparare l'albero di staging prima della firma .zlpod
tasks.register<Copy>("stagePod") {
    dependsOn("assembleRelease")
    from("src/main/manifest.json")
    from("src/main/assets") { into("assets") }
    from("src/main/i18n") { into("i18n") }
    into(layout.buildDirectory.dir("outputs/pod-staging"))
}
