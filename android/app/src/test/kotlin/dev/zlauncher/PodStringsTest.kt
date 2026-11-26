package dev.zlauncher

import dev.zlauncher.pods.AppPodStrings
import java.io.File
import java.nio.file.Files
import org.junit.Assert.assertEquals
import org.junit.Test

/** The texts of a Pod from its `i18n/` folder (§8): app language, then English, then the key. */
class PodStringsTest {
    /** A Pod's `i18n/` folder with the given files. */
    private fun i18n(vararg files: Pair<String, String>): File {
        val dir = Files.createTempDirectory("zl-pod-i18n").toFile()
        for ((name, text) in files) File(dir, name).writeText(text)
        return dir
    }

    @Test
    fun languageThenEnglishThenKey() {
        val dir = i18n("it.json" to """{"title": "Titolo"}""", "en.json" to """{"title": "Title", "close": "Close"}""")
        val strings = AppPodStrings.load(dir, "it")
        assertEquals("Titolo", strings.get("title"))
        assertEquals("Close", strings.get("close"))
        assertEquals("missing.key", strings.get("missing.key"))
    }

    @Test
    fun unknownLanguageFallsBackToEnglish() {
        val dir = i18n("en.json" to """{"title": "Title"}""")
        assertEquals("Title", AppPodStrings.load(dir, "de").get("title"))
    }

    @Test
    fun badFileIsSkipped() {
        val dir = i18n("it.json" to "not json", "en.json" to """{"title": "Title"}""")
        assertEquals("Title", AppPodStrings.load(dir, "it").get("title"))
    }

    @Test
    fun noFolderGivesTheKeys() {
        assertEquals("title", AppPodStrings.load(null, "it").get("title"))
    }

    @Test
    fun argumentsAreFormatted() {
        val dir = i18n("en.json" to """{"count": "%1${'$'}d games"}""")
        assertEquals("3 games", AppPodStrings.load(dir, "en").get("count", 3))
    }
}
