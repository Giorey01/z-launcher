pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}
dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "zpod-sample"

// In sviluppo locale include direttamente z-launcher se presente nella cartella a fianco
val zlauncherDir = file("../z-launcher/android")
if (zlauncherDir.exists()) {
    includeBuild(zlauncherDir) {
        dependencySubstitution {
            substitute(module("dev.zlauncher:pod-api")).using(project(":pod-api"))
            substitute(module("dev.zlauncher:pod-ui")).using(project(":pod-ui"))
            substitute(module("dev.zlauncher:pod-platform")).using(project(":pod-platform"))
        }
    }
}
