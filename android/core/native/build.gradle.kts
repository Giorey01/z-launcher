import java.util.Properties
import org.gradle.api.tasks.Exec

// The Rust core (crates/zl-ffi) as Kotlin: the UniFFI bindings, generated from the built library,
// and the build of the library itself:
//   cargoNdkArm64   libzl_ffi.so for arm64-v8a through cargo-ndk, packed by :app (jniLibs)
//   cargoHost       the same library for this machine, for the bindings and the JVM unit tests
//   generateUniffi  Kotlin bindings from the host library (library mode) into build/generated
// A pure JVM module: the bindings only need JNA's classes to compile. At runtime the app brings
// JNA's AAR (with libjnidispatch.so); the JVM tests bring its jar and point jna.library.path at
// the host library (see app/build.gradle.kts).
plugins {
    alias(libs.plugins.zl.jvm.library)
}

dependencies {
    compileOnly(libs.jna)
    // The generated bindings may suspend through kotlinx.coroutines; the app brings it at run time.
    compileOnly(libs.kotlinx.coroutines.core)
}

val workspaceDir = rootProject.projectDir.resolve("..")
val ffiCrate = workspaceDir.resolve("crates/zl-ffi")
val profile = "release-ffi"
val libName = "zl_ffi"

fun tool(name: String): String {
    val cargoHome = System.getenv("CARGO_HOME") ?: "${System.getProperty("user.home")}/.cargo"
    return File(cargoHome, "bin/$name").takeIf { it.canExecute() }?.absolutePath ?: name
}

fun sdkDir(): File {
    val fromEnv = System.getenv("ANDROID_HOME") ?: System.getenv("ANDROID_SDK_ROOT")
    if (fromEnv != null) return File(fromEnv)
    val props = Properties().also { p -> rootProject.file("local.properties").takeIf { it.exists() }?.inputStream()?.use(p::load) }
    return File(props.getProperty("sdk.dir") ?: error("ANDROID_HOME or sdk.dir is needed to find the NDK"))
}

/** The newest NDK under the SDK (ANDROID_NDK_HOME wins). */
fun ndkDir(): File = System.getenv("ANDROID_NDK_HOME")?.let(::File)
    ?: sdkDir().resolve("ndk").listFiles()?.filter { it.isDirectory }?.maxByOrNull { it.name }
    ?: error("no NDK under ${sdkDir()}/ndk")

// Everything the library is made of: its sources and the workspace manifests.
val rustInputs = fileTree(workspaceDir) {
    include("Cargo.toml", "Cargo.lock", "crates/zl-ffi/**", "crates/zl-config/**")
    exclude("**/target/**")
}

val jniLibsDir = layout.buildDirectory.dir("jniLibs")
val generatedDir = layout.buildDirectory.dir("generated/uniffi")
val hostLibDir = layout.buildDirectory.dir("hostLib")

val cargoNdkArm64 = tasks.register<Exec>("cargoNdkArm64") {
    group = "rust"
    description = "lib$libName.so for arm64-v8a (cargo-ndk)"
    workingDir = workspaceDir
    inputs.files(rustInputs)
    outputs.dir(jniLibsDir)
    // Android 15 devices with 16 KB pages: the NDK before r28 does not align for it by default.
    environment("RUSTFLAGS", "-C link-arg=-Wl,-z,max-page-size=16384")
    // Symbols are kept in the host build (the bindings are read from its metadata) but not in the library that ships.
    environment("CARGO_PROFILE_RELEASE_FFI_STRIP", "symbols")
    doFirst { environment("ANDROID_NDK_HOME", ndkDir().absolutePath) }
    commandLine(tool("cargo"), "ndk", "-t", "arm64-v8a", "-o", jniLibsDir.get().asFile.absolutePath, "build", "--profile", profile, "-p", "zl-ffi")
    doLast {
        // Only the core's library ships, not the .so of any dependency that declares itself cdylib.
        fileTree(jniLibsDir).matching { include("**/*.so"); exclude("**/lib$libName.so") }.forEach { it.delete() }
    }
}

val cargoHost = tasks.register<Exec>("cargoHost") {
    group = "rust"
    description = "lib$libName.so for this machine (bindings, JVM tests)"
    workingDir = workspaceDir
    inputs.files(rustInputs)
    outputs.dir(hostLibDir)
    commandLine(tool("cargo"), "build", "--profile", profile, "-p", "zl-ffi")
    doLast {
        copy {
            from(workspaceDir.resolve("target/$profile")) { include("lib$libName.so", "lib$libName.dylib", "$libName.dll") }
            into(hostLibDir)
        }
    }
}

val generateUniffi = tasks.register<Exec>("generateUniffi") {
    group = "rust"
    description = "Kotlin bindings from the host library (UniFFI library mode)"
    dependsOn(cargoHost)
    workingDir = workspaceDir
    inputs.files(rustInputs)
    outputs.dir(generatedDir)
    doFirst { generatedDir.get().asFile.deleteRecursively() }
    commandLine(
        tool("cargo"), "run", "--profile", profile, "-q", "-p", "zl-ffi", "--features", "bindgen", "--bin", "uniffi-bindgen", "--",
        "generate", "--library", hostLibDir.get().asFile.resolve("lib$libName.so").absolutePath,
        "--language", "kotlin", "--no-format", "--out-dir", generatedDir.get().asFile.absolutePath,
        "--config", ffiCrate.resolve("uniffi.toml").absolutePath,
    )
}

sourceSets["main"].kotlin.srcDir(generatedDir)
tasks.named("compileKotlin") { dependsOn(generateUniffi) }
