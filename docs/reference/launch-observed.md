# Intent di lancio osservati su dispositivo

Intent che hanno avviato davvero un gioco su un AYN Thor (Android 13), letti il 24 novembre 2026
dal log di lancio che un altro frontend scrive nella cartella dati scelta dall'utente e dal log di
sistema (`ActivityTaskManager: START`). Sono fatti di interoperabilità (come ogni emulatore accetta
un gioco), non codice. Delle extra si conoscono i nomi, non sempre i valori.

In tutti i casi la ROM era passata come **URI documento SAF** sotto un albero concesso dall'utente
(`content://com.android.externalstorage.documents/tree/primary%3AGames/document/primary%3AGames%2F<piattaforma>%2F<file>`),
e prima del lancio il frontend concedeva all'emulatore i permessi sull'albero. Il percorso reale
(`/storage/emulated/0/Games/...`) era comunque disponibile come seconda variabile.

| Emulatore | Package / activity | Action | Data | Extra |
| --- | --- | --- | --- | --- |
| RetroArch 64 bit | `com.retroarch.aarch64` / `com.retroarch.browser.retroactivity.RetroActivityFuture` | `VIEW` | nessuno | `ROM`, `LIBRETRO`, `CONFIGFILE`, `DATADIR`, `SDCARD`, `EXTERNAL`, `APK`, `IME` |
| Eden | `dev.eden.eden_emulator` / `org.yuzu.yuzu_emu.activities.EmulationActivity` | `android.nfc.action.TECH_DISCOVERED` | URI della ROM | nessuna |
| Eden Nightly | `dev.eden.eden_emulator.nightly` / stessa activity | come Eden | URI della ROM | nessuna |
| Azahar | `io.github.lime3ds.android` / `org.citra.citra_emu.activities.EmulationActivity` | `VIEW` | URI della ROM | nessuna |
| PPSSPP | `org.ppsspp.ppsspp` / `org.ppsspp.ppsspp.PpssppActivity` | `VIEW` | URI della ROM | nessuna |
| melonDS | `me.magnum.melonds` / `me.magnum.melonds.ui.emulator.EmulatorActivity` | `me.magnum.melonds.LAUNCH_ROM` | nessuno | `uri` |
| AetherSX2 | `xyz.aethersx2.android` / `xyz.aethersx2.android.EmulationActivity` | `MAIN` | nessuno | `bootPath` |
| Dolphin | `org.dolphinemu.dolphinemu` / `org.dolphinemu.dolphinemu.ui.main.MainActivity` | `MAIN` | nessuno | `AutoStartFile` |
| Cemu | `info.cemu.cemu` / `info.cemu.cemu.emulation.EmulationActivity` | `VIEW` | URI del file `code/*.rpx` | nessuna |

Altre osservazioni:

- Eden con l'action NFC: l'intent di sistema ha `flg=0x10010001` (nuovo task, niente animazione,
  permesso di lettura sull'URI) e la URI anche in `ClipData` con MIME `application/octet-stream`.
- Wii U (Cemu): il gioco è una cartella `<titolo> [ID]/` e si lancia il file `code/*.rpx` dentro.
- Switch: cartelle `Switch/Games/` e `Switch/Updates_DLC/`; gli aggiornamenti e i DLC non sono
  giochi da mostrare.
- Le cartelle piattaforma possono avere maiuscole (`Ps2`, `Wii`, `Switch`) e contenere
  `systeminfo.txt` di ES-DE: si ignora.
- Emulatori installati sul Thor di prova: RetroArch aarch64, Dolphin, PPSSPP, Eden, melonDS (anche
  `me.magnum.melondualds`), Azahar (`io.github.lime3ds.android`), Vita3K, Cemu, AetherSX2,
  ARMSX2 (`com.armsx2`), GameNative (`app.gamenative`), GameHub Lite (`gamehub.lite`), Moonlight
  Noir (`com.limelight.noir`), ES-DE, Daijishō.
