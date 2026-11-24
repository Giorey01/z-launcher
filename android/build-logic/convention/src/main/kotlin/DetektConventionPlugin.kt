import io.gitlab.arturbosch.detekt.extensions.DetektExtension
import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.kotlin.dsl.configure

/**
 * Static analysis for a module: the shared rule set in `config/detekt/detekt.yml`.
 *
 * A module with a `detekt-baseline.xml` next to its build file tolerates the findings recorded there
 * (legacy code not yet reshaped); a module without one must pass clean.
 */
class DetektConventionPlugin : Plugin<Project> {
    override fun apply(target: Project) = with(target) {
        pluginManager.apply("io.gitlab.arturbosch.detekt")
        extensions.configure<DetektExtension> {
            buildUponDefaultConfig = true
            config.setFrom(rootProject.file("config/detekt/detekt.yml"))
            val baselineFile = file("detekt-baseline.xml")
            if (baselineFile.exists()) baseline = baselineFile
            source.setFrom(files("src/main/kotlin", "src/test/kotlin"))
        }
    }
}
