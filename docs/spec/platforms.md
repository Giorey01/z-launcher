# Database di piattaforme, emulatori e profili di lancio

Due file JSON in `data/`, incorporati nell'APK (`include_str!` nel crate `zl-platforms`) e
aggiornabili da una cartella su disco o da un URL senza un nuovo APK: vince la copia con `version`
più alta, purché `schema` coincida con quello che la build capisce (oggi `1`). Una copia rotta,
di altro schema o incoerente con l'altro file non viene usata e l'app resta sulla copia incorporata.

I file sono generati da `scripts/convert_esde.py` (fonte: ES-DE, MIT, vedi `NOTICE`) più i dati
osservati su dispositivo (`docs/reference/launch-observed.md`). Non si modificano a mano: si corregge
il convertitore e si rigenera.

```bash
python3 -I scripts/convert_esde.py es_find_rules.xml es_systems.xml data
```

## `platforms.json`

Campi di primo livello: `schema`, `version`, `sources`, `platforms`.

Ogni piattaforma:

| Campo | Significato |
| --- | --- |
| `id` | nome cartella Cocoon in minuscolo (`psx`, `snes`, `tg16`...). Gli id non Cocoon hanno `cocoon: false` |
| `name` | nome in inglese |
| `extensions` | estensioni dei file di gioco, minuscole, senza punto; anche multi-parte (`p8.png`). Per le piattaforme a file tag sono le estensioni dei tag |
| `aliases` | altri nomi accettati ovunque ci sia un id (id ES-DE come `sfc`, `megadrive`, `pcengine`) |
| `multi_disc` | i giochi sono più dischi legati da un `.m3u` |
| `tag_files` | `{ext, key}`: estensione del file tag e chiave che porta (`steamappid`, `localgameid`, `scummvm_id`, `vita_game_id`, `moonlight_id`, `titleId`) |

Gli id si confrontano **senza distinguere le maiuscole** (sul dispositivo si vedono cartelle `Ps2`,
`Wii`, `Switch`): `PlatformDb::platform(nome)` abbassa le maiuscole e risolve gli alias.
I giochi che sono cartelle (Wii U, Xbox 360, Switch) non hanno estensione di file: il chiamante
passa il file da lanciare (per Wii U il `code/*.rpx`).

## `emulators.json`

Primo livello: `schema`, `version`, `sources`, `default_preference`, `preferences`, `emulators`.

**Preferenze (dati, non codice).** `preferences[platform]` è una lista ordinata di
`{emulator, core?}` e decide il default proposto all'onboarding: gli emulatori standalone per i
sistemi pesanti (ps2, psp, gc, wii, switch, dreamcast, n3ds, nds...), RetroArch per quelli
retro. Le piattaforme senza lista usano `default_preference` (RetroArch) e poi gli altri emulatori
installati in ordine alfabetico di id.

Ogni emulatore: `id`, `name`, `kind` (`standalone` | `retroarch`), `packages`, `platforms`,
`profiles`, e per RetroArch `cores`.

- `packages`: `{package, activity?, verified}`. Più voci coprono varianti e fork (RetroArch
  `aarch64`/`ra32`/base, Dolphin, PPSSPP e Gold, melonDS e il suo fork). La prima installata si
  usa. L'activity è sempre il nome di classe completo. Pacchetti "mimetizzati" di ES-DE (build
  camuffate come app di benchmark o giochi famosi) sono esclusi: darebbero falsi positivi.
- `cores`: id piattaforma → lista di `{id, name, file}` in ordine di preferenza (il primo è il
  default). `file` è il nome della libreria nella cartella dei core di RetroArch.
- `profiles`: modi di lanciare.

### Profilo

| Campo | Significato |
| --- | --- |
| `id`, `label`, `platforms` | identità e piattaforme lanciate |
| `rom_access` | forma in cui serve il ROM: `path` (`{rom}`), `file_uri` (`{rom_uri}`), `content_uri` (`{rom_provider}`, FileProvider nostro), `saf` (`{rom_saf}`), `none` (guidato dai tag) |
| `uses_core` | il profilo usa `{core}`/`{core_path}` (solo RetroArch) |
| `intent` | descrizione dell'intent, sotto |
| `verified` | una fonte conferma l'intent |
| `source` | `es-de`, `es-de+device`, `manual` |
| `verified_on` | data in cui un gioco è partito davvero su un dispositivo |
| `note` | differenze dalla fonte o dubbi |

`intent`: `action`, `category[]`, `data`, `mime`, `extras[]`, `flags[]`, `clip_data`, `uri_grants[]`.

- Extra tipizzati: `{key, type: "string"|"bool"|"int"|"string_array", value}`. Per `int` il valore
  è un numero o un template che dopo la sostituzione deve essere un intero.
- `flags` ammessi: `clear_task`, `clear_top`, `new_task`, `no_animation`,
  `grant_read_uri_permission`, `grant_write_uri_permission`.
- `clip_data`: `{uri, mime}`, ClipData da allegare (serve a Eden: l'URI sta anche lì).
- `uri_grants`: `[{uri, read, write, optional}]`: permessi che il launcher concede al pacchetto di
  destinazione con `grantUriPermission` **prima** del lancio. `optional` salta la concessione se
  il chiamante non ha la variabile (di solito `{rom_saf_tree}`).

### Variabili

`{rom}` percorso, `{rom_uri}` `file://`, `{rom_saf}` documento SAF, `{rom_saf_tree}` albero SAF
concesso dall'utente, `{rom_provider}` `content://` del FileProvider, `{rom_name}` nome senza
l'ultima estensione, `{rom_file}` nome del file, `{rom_dir}` cartella del file, `{platform}`,
`{platform_dir}` cartella della piattaforma, `{package}` pacchetto installato scelto, `{core}` file
del core, `{core_path}` percorso assoluto del core (`/data/data/<package>/cores/<core>`),
`{tags.NOME}` tag del gioco. Una variabile sconosciuta è un errore **al caricamento** del database
(`bad_template`) e al lancio (`unknown_variable`); un valore che il chiamante non ha dato è
`missing_variable`/`missing_tag`, mai una stringa vuota.

Il caricamento controlla anche: id unici, piattaforme esistenti, flag noti, `rom_access`
coerente con le variabili usate, uso dei core solo in RetroArch, preferenze che puntano a emulatori
e core esistenti.

## Ordine di scelta (RF-08)

1. override del gioco; 2. default della piattaforma; 3. regola per estensione (suffisso più lungo:
`a.p8.png` prova `p8.png` poi `png`); 4. se resta un solo profilo installato lo si usa, altrimenti
`NeedsChoice { candidates }` e si chiede all'utente (lista vuota: nessun emulatore installato).
Un riferimento a un emulatore non installato o a un profilo che non lancia la piattaforma viene
saltato. I candidati sono ordinati secondo `preferences`; RetroArch dà un candidato per core.

## Differenze tra ES-DE e il dispositivo

Dove l'intent osservato (AYN Thor, Android 13, 24 novembre 2026) differisce da ES-DE, vince
l'osservato; i profili toccati hanno `source: es-de+device` e `verified_on`.

| Emulatore | ES-DE | Osservato (adottato) |
| --- | --- | --- |
| RetroArch | nessuna action | `VIEW`; extra `DATADIR`, `SDCARD`, `EXTERNAL`, `APK`, `IME` esistono ma non se ne conoscono i valori, quindi non sono nel profilo |
| Eden / Eden Nightly | `data` = URI del FileProvider | URI SAF, flag nuovo task + niente animazione + lettura URI, ClipData `application/octet-stream` |
| Azahar | nessuna action | `VIEW`; restano i flag `clear_task`/`clear_top` di ES-DE |
| Dolphin | `TvMainActivity` + categoria LEANBACK | `ui.main.MainActivity`, `MAIN`, nessuna categoria |
| Cemu | nessuna action | `VIEW` sul file `code/*.rpx` |
| PPSSPP, melonDS, AetherSX2 (pacchetto di NetherSX2) | uguale | confermato |

I flag di Azahar e di AetherSX2 non sono stati osservati. Tutti i lanci osservati passavano il ROM
come URI SAF sotto un albero concesso: per questo ogni profilo `saf` ha `grant_read_uri_permission`
e due `uri_grants` (documento e albero). I permessi di scrittura non sono stati osservati e non
sono concessi.

## Cosa non è verificato

- Moonlight (`com.limelight`, `com.limelight.noir`): ES-DE non ha Moonlight; il profilo `open-app`
  apre solo l'app (`verified: false`). Il lancio di un'app per nome (`moonlight_id`) va ancora
  scoperto su dispositivo.
- xCloud: nessun emulatore, nessun profilo.
- RetroArch Plus (`com.retroarch.plus`): pacchetto aggiunto da noi con la stessa activity delle altre
  build, `verified: false`.
- Fork di Yuzu/Citron/Sudachi: ES-DE non li elenca (Eden e Kenji-NX sì); non inventati.
- ES-DE usa `%INJECT%` (contenuto del file) per Steam, ScummVM, Vita3K: li abbiamo mappati sui tag
  Cocoon (`steamappid`, `localgameid`, `scummvm_id`, `vita_game_id`); `ps3_serial` e `ps4_id` sono
  chiavi nostre per i titoli PS3/PS4, non di Cocoon. Il significato esatto del valore (per ScummVM
  l'URI completo o l'id) è quello di ES-DE e va provato su dispositivo.
- Alcuni intent MAME4droid (Dragon, Oric) e idTech4A++ non sono convertiti (usano contenuti di file
  accanto al ROM).

## Codici di errore

`DbError`: `io`, `parse` (file, riga, colonna), `schema_mismatch`, `Invalid{code, subject}` con
`duplicate_emulator_id`, `duplicate_profile_id`, `bad_or_duplicate_platform_name`, `bad_extension`,
`bad_package`, `unknown_platform`, `unknown_flag`, `bad_template`, `rom_access_mismatch`,
`core_use_mismatch`, `unknown_preference_emulator`, `unknown_preference_core`.
`LaunchError`: `unknown_platform`, `unknown_emulator`, `unknown_profile`, `unknown_package`,
`profile_platform_mismatch`, `unknown_variable`, `bad_template`, `missing_variable`, `missing_tag`,
`invalid_integer`, `no_core`, `unknown_core`.

## Fixture

`fixtures/launch/expected.json`: casi (pacchetti installati + richiesta) → `Outcome` atteso
(`launch{intent, profile, by}` o `needs_choice{candidates}`), errori attesi, e la lista di
pacchetti del dispositivo di prova con emulatori rilevati e default proposti. Li legge il test
Rust e li leggerà Kotlin tramite la FFI.

## URI SAF dei ROM

Gli emulatori ricevono il ROM come documento SAF sotto la cartella concessa dall'utente
(`ACTION_OPEN_DOCUMENT_TREE`). Il core lo costruisce da solo (`zl_platforms::saf`), senza Android:

- id dell'albero = parte dopo `/tree/` dell'URI concesso, decodificata (`primary:Games`);
- id del documento = id dell'albero + `/` + percorso del file relativo alla cartella dell'albero
  (`primary:Games/Ps2/God of War (Europe, Australia) (En,Fr,De,Es,It).iso`);
- URI = `content://<autorità>/tree/<albero codificato>/document/<documento codificato>`.

La codifica è quella di `Uri.encode` di Android: byte UTF-8 come `%XX` maiuscoli, tranne lettere,
cifre e `_ - ! . ~ ' ( ) *`. Quindi `(` e `)` restano, mentre `,`, spazio, `[`, `:` e `/` interni
all'id diventano `%2C`, `%20`, `%5B`, `%3A`, `%2F`. La cartella dell'albero sul file system si
ricava dall'id (`primary:` è `/storage/emulated/0`, `ABCD-1234:` è `/storage/ABCD-1234`); se il
volume non è riconoscibile si usa il percorso della cartella radice salvata.
`{rom_uri}` è `file://` + percorso codificato; `{rom_provider}` non è ancora disponibile.
