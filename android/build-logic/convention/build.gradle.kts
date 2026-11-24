plugins {
    `kotlin-dsl`
}

kotlin { jvmToolchain(17) }

dependencies {
    compileOnly(libs.kotlin.gradlePlugin)
    compileOnly(libs.detekt.gradlePlugin)
}

gradlePlugin {
    plugins {
        register("detekt") { id = "zl.detekt"; implementationClass = "DetektConventionPlugin" }
        register("jvmLibrary") { id = "zl.jvm.library"; implementationClass = "JvmLibraryConventionPlugin" }
    }
}
