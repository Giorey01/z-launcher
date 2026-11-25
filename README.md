# Z-Launcher

An open source, gamepad-first Android home screen for emulation handhelds, with a cozy console look.

Z-Launcher scans your ROM folders, shows your games with artwork, and launches them in the right
emulator and core. It can be themed down to the sounds and the icon frames of every platform, and it
reads themes made for Cocoon without conversion.

## Status: 0.1.0, first preview

Works today (tested on an AYN Thor, dual screen, Android 13):

- Setup with the controller: storage access, ROM folders, emulators found on the device.
- Library scan with ES-DE style folders; platforms as Smart Folders; games launched through the
  installed emulators (RetroArch, melonDS, Dolphin, PPSSPP, Eden, Azahar, AetherSX2 and more).
- Dual screen: grid below, hero above; games start on the top screen while the bottom one shows the
  game and dims after a while (touch to wake). Single screen mode too.
- Themes: Cocoon theme ZIPs and asset packs, wallpapers, glass or solid surfaces, accent colours,
  cursor styles, interface sounds.
- Artwork: covers, heroes and logos imported from ES-DE and Cocoon media folders, or downloaded from
  SteamGridDB with your own free API key. Game info (year, developer, genre, description) from the
  ES-DE gamelists.

Not there yet: play time, favourites and hidden games, achievements, save management, search
beyond the app drawer.

Requirements: Android 10 or later, arm64.

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
cd android && ./gradlew assembleOssDebug   # needs cargo-ndk and the Android NDK
```

## License

Apache-2.0. See `LICENSE`.
