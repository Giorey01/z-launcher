package dev.zlauncher

import dev.zlauncher.core.nativecore.Core
import dev.zlauncher.core.nativecore.CoreException
import dev.zlauncher.core.nativecore.ExtraValue
import dev.zlauncher.core.nativecore.LaunchIntent
import dev.zlauncher.core.nativecore.Plan
import dev.zlauncher.core.nativecore.ProfileRef
import dev.zlauncher.core.nativecore.RawLaunchRequest
import java.io.File
import java.nio.file.Files
import org.json.JSONArray
import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * The cross-target contract: `fixtures/launch/expected.json` is read here, through the real Rust library,
 * and must give the same outcomes the Rust test of `zl-platforms` asserts.
 */
class LaunchFixtureTest {
    /** The fixture document. */
    private val doc = JSONObject(File(checkNotNull(System.getProperty("zl.fixturesDir")), "launch/expected.json").readText())

    /** A core over a throwaway data folder. */
    private fun newCore(): Core = Core(Files.createTempDirectory("zl-core-test").toString())

    /** The package list of a case: a literal array or the name of a shared list. */
    private fun packages(case: JSONObject): List<String> {
        val value = case.get("installed_packages")
        val array = if (value is String) doc.getJSONArray(value) else value as JSONArray
        return List(array.length()) { array.getString(it) }
    }

    /** `null` for a missing or JSON-null key. */
    private fun JSONObject.optStringOrNull(key: String): String? = if (isNull(key)) null else getString(key)

    /** A profile reference from JSON. */
    private fun profile(json: JSONObject?): ProfileRef? =
        json?.let { ProfileRef(it.getString("emulator"), it.getString("profile"), it.optStringOrNull("core")) }

    /** The request of a case, in the shape the FFI takes. */
    private fun request(json: JSONObject): RawLaunchRequest {
        val context = json.getJSONObject("context")
        val uris = context.optJSONObject("rom_uris") ?: JSONObject()
        val tags = context.optJSONObject("tags") ?: JSONObject()
        val extensions = json.optJSONObject("extension_defaults") ?: JSONObject()
        return RawLaunchRequest(
            romPath = context.optString("rom_path"),
            fileUri = uris.optStringOrNull("file"),
            safUri = uris.optStringOrNull("saf"),
            safTree = uris.optStringOrNull("saf_tree"),
            providerUri = uris.optStringOrNull("provider"),
            platform = context.getString("platform"),
            platformDir = context.optStringOrNull("platform_dir"),
            tags = tags.keys().asSequence().associateWith { tags.getString(it) },
            core = context.optStringOrNull("core"),
            `package` = context.optStringOrNull("package"),
            gameOverride = profile(json.optJSONObject("game_override")),
            platformDefault = profile(json.optJSONObject("platform_default")),
            extensionDefaults = extensions.keys().asSequence().associateWith { profile(extensions.getJSONObject(it))!! },
        )
    }

    /** A profile reference as JSON, with the core only when there is one. */
    private fun ProfileRef.toJson(): JSONObject =
        JSONObject().put("emulator", emulator).put("profile", profile).also { json -> core?.let { json.put("core", it) } }

    /** An extra as the fixture spells it. */
    private fun extraJson(key: String, value: ExtraValue): JSONObject {
        val json = JSONObject().put("key", key)
        return when (value) {
            is ExtraValue.Text -> json.put("type", "string").put("value", value.value)
            is ExtraValue.Flag -> json.put("type", "bool").put("value", value.value)
            is ExtraValue.Number -> json.put("type", "int").put("value", value.value)
            is ExtraValue.TextList -> json.put("type", "string_array").put("value", JSONArray(value.value))
        }
    }

    /** The intent as JSON, leaving out empty lists and absent values (the fixture does the same). */
    private fun LaunchIntent.toJson(): JSONObject {
        val json = JSONObject().put("package", `package`)
        action?.let { json.put("action", it) }
        activity?.let { json.put("activity", it) }
        data?.let { json.put("data", it) }
        mime?.let { json.put("mime", it) }
        if (categories.isNotEmpty()) json.put("categories", JSONArray(categories))
        if (flags.isNotEmpty()) json.put("flags", JSONArray(flags))
        if (extras.isNotEmpty()) json.put("extras", JSONArray(extras.map { extraJson(it.key, it.value) }))
        clipData?.let { json.put("clip_data", JSONObject().put("uri", it.uri).put("mime", it.mime)) }
        if (uriGrants.isNotEmpty()) {
            json.put("uri_grants", JSONArray(uriGrants.map { JSONObject().put("uri", it.uri).put("read", it.read).put("write", it.write) }))
        }
        return json
    }

    /** The outcome as JSON, in the shape of the fixture's `expected`. */
    private fun Plan.toJson(): JSONObject = when (this) {
        is Plan.Launch -> JSONObject().put(
            "launch",
            JSONObject().put("intent", intent.toJson()).put("profile", profile.toJson()).put("by", by.name.lowercase()),
        )
        is Plan.NeedsChoice -> JSONObject().put("needs_choice", JSONObject().put("candidates", JSONArray(candidates.map { it.profile.toJson() })))
    }

    /** Drops empty arrays and nulls so a fixture that omits defaults compares equal. */
    private fun canonical(value: Any?): Any? = when (value) {
        is JSONObject -> JSONObject().also { out ->
            value.keys().forEach { key ->
                val inner = canonical(value.get(key))
                if (inner != null) out.put(key, inner)
            }
        }
        is JSONArray -> if (value.length() == 0) null else JSONArray(List(value.length()) { canonical(value.get(it)) })
        JSONObject.NULL -> null
        else -> value
    }

    /** Every case of the fixture gives the outcome the Rust test expects, byte for byte in its JSON form. */
    @Test
    fun launchCasesMatchTheRustOutcomes() {
        val core = newCore()
        val cases = doc.getJSONArray("cases")
        assertTrue("fixture lost cases", cases.length() >= 8)
        for (i in 0 until cases.length()) {
            val case = cases.getJSONObject(i)
            val plan = core.planRequest(packages(case), request(case.getJSONObject("request")))
            val expected = canonical(case.getJSONObject("expected")) as JSONObject
            val actual = canonical(plan.toJson()) as JSONObject
            assertTrue("case ${case.getString("name")}:\nexpected $expected\nactual   $actual", expected.similar(actual))
        }
    }

    /** Every error case fails with the code (and subject) the Rust test expects. */
    @Test
    fun errorCasesReportTheirCode() {
        val core = newCore()
        val cases = doc.getJSONArray("error_cases")
        assertTrue(cases.length() >= 5)
        for (i in 0 until cases.length()) {
            val case = cases.getJSONObject(i)
            val error = runCatching { core.planRequest(packages(case), request(case.getJSONObject("request"))) }.exceptionOrNull()
            assertTrue("case ${case.getString("name")}: $error", error is CoreException.Launch)
            val launch = error as CoreException.Launch
            assertEquals(case.getString("expected_error"), "${launch.code} ${launch.detail}".trim())
        }
    }

    /** Creates an empty file and its folders. */
    private fun touch(root: File, relative: String) {
        val file = File(root, relative)
        file.parentFile?.mkdirs()
        file.writeText("x")
    }

    /** The emulators of the device list are detected and the defaults proposed for the platforms in the library match. */
    @Test
    fun detectionOfTheDeviceMatchesTheFixture() {
        val case = doc.getJSONArray("detect_cases").getJSONObject(0)
        val core = newCore()
        val root = Files.createTempDirectory("zl-games").toFile()
        val present = mapOf(
            "ps2" to "Ps2/a.chd", "psp" to "psp/a.iso", "switch" to "Switch/Games/a.nsp", "gc" to "gc/a.iso",
            "n3ds" to "n3ds/a.3ds", "nds" to "nds/a.nds", "nes" to "nes/a.nes", "psx" to "psx/a.cue", "wiiu" to "wiiu/Game [1]/code/g.rpx",
        )
        present.values.forEach { touch(root, it) }
        core.addRoot(root.absolutePath, null)
        core.scan(object : dev.zlauncher.core.nativecore.ScanListener {
            override fun onEvent(event: dev.zlauncher.core.nativecore.ScanEvent) = Unit
        })
        val detection = core.detectInstalled(packages(case))
        val installed = case.getJSONObject("expected_installed")
        assertEquals(installed.length(), detection.installed.size)
        for (id in installed.keys()) {
            val found = detection.installed.firstOrNull { it.emulator == id }
            assertNotNull("$id not detected", found)
            assertEquals(installed.getJSONArray(id).let { a -> List(a.length()) { a.getString(it) } }, found!!.packages)
        }
        val defaults = case.getJSONObject("expected_defaults")
        val proposed = detection.proposed.associateBy { it.platformId }
        assertEquals(present.keys, proposed.keys)
        for (platform in present.keys) {
            assertEquals(platform, profile(defaults.getJSONObject(platform)), proposed.getValue(platform).candidate.profile)
        }
    }
}
