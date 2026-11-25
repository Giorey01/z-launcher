plugins {
    `java-platform`
}

javaPlatform {
    allowDependencies()
}

dependencies {
    api(platform(libs.compose.bom))

    constraints {
        api(libs.kotlinx.coroutines.android)
        api(libs.kotlinx.coroutines.core)
        api(libs.coil.compose)
        api(libs.androidx.core.ktx)
        api(libs.androidx.lifecycle.runtime.compose)
        api(libs.androidx.lifecycle.viewmodel.compose)
    }
}
