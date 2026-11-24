# Z-Launcher

An open source, gamepad-first Android home screen for emulation handhelds, with a cozy console look.

Z-Launcher scans your ROM folders, shows your games with artwork, launches them in the right
emulator and core, tracks play time, and can be themed down to the sounds and the icon frames of
every platform. It reads themes made for Cocoon without conversion.

Status: early development. Nothing to install yet.

## Layout

- `crates/`: the Rust core. All the logic (library, platforms, launch profiles, themes,
  metadata, sessions, saves) lives here; the Android app reaches it through UniFFI.
- `android/`: the Android app. Compose UI and what only Android can do (intents, permissions,
  usage stats, audio, notifications, widgets).
- `data/`: platform and emulator databases, updatable without a new APK.
- `fixtures/`: shared test cases read by the Rust and the Kotlin tests.
- `docs/`: specifications (`docs/spec/`) and reference notes (`docs/reference/`).

## Build

```bash
bash scripts/gate.sh          # Rust: format, lints, tests
cd android && ./gradlew assembleDebug
```

## License

Apache-2.0. See `LICENSE`.
