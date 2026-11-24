# Cocoon: comportamento documentato (riferimento clean-room)

Appunti in parole nostre dalla wiki pubblica di Cocoon (`cocoon-shell.com/wiki`, 30 pagine lette il
24 novembre 2026). Servono a fissare il livello di personalizzazione e di funzioni da raggiungere,
non come testo da riusare. I temi hanno la loro specifica: `docs/spec/cocoon-theme.md`.

## Libreria e piattaforme

- La **cartella è la piattaforma**: il nome della cartella ROM è l'id (`gba`, `psx`, …). Una
  piattaforma può avere più cartelle ROM; opzione "scansiona sottocartelle".
- L'elenco delle piattaforme viene da un indice JSON scaricato al primo avvio e aggiornabile
  ("Refetch Platforms", aggiorna anche i player).
- Player (emulatore) di default per piattaforma; override per gioco.
- Multi-disco: `.m3u` accanto ai dischi, un nome di file per riga, una sola voce in libreria.
  CUE collegati automaticamente ai BIN.
- Formati comuni: `zip`, `7z`, `bin+cue`, `iso`, `chd`, `m3u`, `gdi`, `pbp`, `sfc`/`smc`, `nes`,
  `gba`, `nds`, `z64`/`n64`/`v64`, `p8.png` (PICO-8).
- "Smart Subfolders": le sottocartelle di una cartella ROM diventano cartelle nella Home.
- Giochi nascosti recuperabili dalle impostazioni.

### Id delle piattaforme

Nintendo: `nes` `fds` `satellaview` `snes` `snesmsu1` `n64` `gc` `wii` `wiiu` `wiiware` `switch`
`gb` `gbc` `gba` `nds` `ndsi` `n3ds` `virtualboy` `pokemini` `gameandwatch`.
Sony: `psx` `ps2` `ps3` `psp` `pspminis` `psvita`.
Sega: `genesis` `genesismsu` `mastersystem` `gamegear` `sg-1000` `sega32x` `segacd` `saturn`
`dreamcast` `pico` `naomi`.
Atari: `atari2600` `atari5200` `atari7800` `atarijaguar` `atarijaguarcd` `atarilynx` `atarist`.
SNK: `neogeo` `neogeocd` `ngp` `ngpc`.
NEC: `tg16` `tg-cd` `supergrafx` `pcfx` `pc60` `pc88` `pc98`.
Arcade: `mame` `fbneo` `cps1` `cps2` `cps3` `atomiswave`.
Computer: `amstradcpc` `apple2` `bbcmicro` `c64` `amiga` `pet` `plus4` `vic20` `dos` `fmtowns`
`msx` `oric` `x1` `x68000` `to8` `ti83` `windows` `zx81` `zxspectrum` `palm` `adobeflash`.
Altre console: `3do` `arcadia` `cassette` `supercassette` `colecovision` `elektor` `channelf`
`intellivision` `vc4000` `j2me` `odyssey2` `megaduck` `ngage` `cdimono1` `videopac` `uzebox`
`vectrex` `supervision` `wonderswan` `wonderswancolor` `zeebo`.
Fantasy e motori: `pico8` `tic80` `chip8` `arduboy` `lowresnx` `wasm4` `cannonball` `cavestory`
`doom` `flashback` `quake` `quake2` `rpgmaker` `scummvm` `ports`.
Media e streaming: `ebook` `videos` `moonlight` `steam` `xbox` `xbox360` `xcloud`.

La wiki non dà estensioni né emulatori consigliati per piattaforma: li costruiamo noi in
`data/platforms.json` (fonti: Daijishō MIT, ES-DE MIT).

## File tag

File di testo nella cartella della piattaforma; il nome del file senza estensione è il nome del
gioco; ogni riga è `[chiave] valore`.

| Estensione | Cartella | Chiave | Valore |
| --- | --- | --- | --- |
| `.steamappid` | `steam/` | `steamappid` | App ID Steam |
| `.localgameid` | `windows/` | `localgameid` | package Android |
| `.dpt` | `scummvm/` | `scummvm_id` | id ScummVM |
| `.dpt` | `psvita/` | `vita_game_id` | id del gioco Vita |
| `.moonlight` | `moonlight/` | `moonlight_id` | nome dell'app su Moonlight |
| `.xcloud` | `xcloud/` | `titleId` | id del titolo Xbox |

`.dpt` è condiviso: si distingue dalla chiave presente (e dalla cartella). I file tag supportano
artwork, preferiti e cartelle come gli altri giochi.

## Home, griglia, cartelle

- La Home contiene giochi, cartelle, app, scorciatoie ai Pod, Impostazioni e widget.
- Start → Edit Grid: sposta uno o più elementi, ridimensiona giochi e widget con Y, togli una
  scorciatoia, posa negli spazi vuoti (richiede l'ordinamento manuale).
- Start → New: aggiungi giochi, cartelle, app, widget.
- Zoom con pinch o con una scorciatoia del controller; **un layout salvato per livello di zoom**.
- Griglia: Page Mode (pagine o scorrimento continuo), righe e colonne, etichette su una riga.
  Da una a cinque righe; con una riga, stili Standard, Carousel, Cover Flow.
- Ordinamenti: Manuale, A–Z, Giocati di recente, Più giocati, Data di aggiunta, Piattaforma.
- Cartelle: nome, aspetto (colore, simbolo o icone di piattaforma, artwork e logo personali),
  annidabili, con breadcrumb; display di lancio per cartella; aperte flottanti o a schermo intero.
- Smart Folder per regola (contenuto non modificabile a mano): Preferiti, Giochi Android,
  Emulatori, Piattaforma, Genere, Personalizzata. Al primo avvio: modalità Smart Folders (una
  cartella per piattaforma) o Simple (tutti i giochi in Home); cambiarla azzera il layout.
- Dock fino a 5 scorciatoie (dimensione, forma, posizione, sfondo, auto-nascondi); cassetto app con
  ricerca; un'app può essere "trattata come gioco" (ore tracciate, piattaforma, artwork).

## Widget

Orologio (analogico, digitale, flip), Giocati di recente, Lettore musicale (sessione media
Android), Ultimo giocato, Gioco casuale (da libreria o cartella), Ore della settimana, Picnic
(screenshot), RetroAchievements, Immagine fissata (anche GIF), Spazio vuoto; più i widget Android.

## Scheda gioco e modifica

- Info: descrizione, dati di uscita, screenshot e video, HowLongToBeat, achievement, ore e sessioni.
- Y → Edit: titolo, descrizione, dati, sessioni; icona, hero, logo, screenshot, video; emulatore
  e display di lancio; preferito; jingle; abbinamento RetroAchievements.
- "Scrape Game" per un gioco; "Edit Game Artwork" per una sola immagine.

## Metadati

Fonti in ordine configurabile e disattivabili: SteamGridDB (chiave), ScreenScraper (account),
LaunchBox DB, IGDB, HowLongToBeat. Modalità: solo mancanti di base, mancanti scelti, forza tutto;
"sostituisci campi protetti" spento di default protegge le modifiche manuali. Media: box art e
logo, hero e screenshot a parte, video a parte. Import da ES-DE (titoli, descrizioni, statistiche,
media) con priorità scelta; export `gamelist.xml`.

## Sessioni e Pod

- Tracciamento con Usage Access; disattivabile per emulatore o app; sessioni modificabili a mano.
- Now Playing sull'altro schermo durante il gioco, con stile scelto.
- Pod: **Log** (storico, statistiche, immagini da condividere), **Picnic** (screenshot per gioco,
  preferiti, condivisione), **store dei temi**.

## Audio

- Effetti: on/off, volume, vibrazione; ogni effetto sostituibile con un file, ripristinabile.
- Musica: per ora del giorno o playlist (in ordine o casuale), volume proprio; si ferma quando il
  launcher va in background o parte un gioco; cede ad altre app che riproducono media.
- **Jingle**: suono breve quando ci si ferma su un gioco. Priorità gioco > piattaforma > globale.
  Repository della community su GitHub (`owner/repo`) con un `index.json`: `name`, poi una chiave
  per id piattaforma con una lista di `{ name, file, regex? }`. Volume proprio, normalizzazione,
  dissolvenza sulle clip lunghe, musica abbassata durante il jingle.

## Controller

A conferma/apre, B indietro, Y menu dell'elemento, X scambia schermi (o espande l'info sul
singolo schermo), Start menu Start, bumper per pannello social (sinistro) e stato (destro), D-pad
e stick sinistro per muoversi; nelle impostazioni i bumper cambiano scheda e sinistra/destra
regolano i valori. Suggerimenti negli angoli (Corner Hints). Tastiera: frecce, Invio, Esc. Touch:
tocco, pressione lunga = menu, pinch = zoom.

## Schermi

- Dual-screen: Home su uno, info del gioco sull'altro; ruoli invertibili; X scambia.
- Display di lancio: per questa volta (Y → Launch), per gioco, per cartella, per app.
- Display esterno: Automatico, Sempre agganciato (un unico layout), Mai.
- Uno sfondo per schermo. Modalità a schermo singolo con divisione hero/griglia regolabile.

## Pannelli e integrazioni

Pannello stato (controlli del dispositivo, notifiche, cambio schermo attivo); pannello social
(amici Discord e Steam, cosa giocano, messaggi Steam, conversazioni Android con risposta).
Discord Rich Presence. RetroAchievements con chiave web API; Steam con login via QR.

## Backup

Backup a scelta per parti: dati dei giochi (info, storico, sessioni, Picnic), aspetto (temi,
sfondi, suoni, impostazioni visive), layout e cartelle, fonti e player. Sta nella cartella dati
scelta al primo avvio; restore selettivo; migrazione copiando la cartella dati.

## Impostazioni (mappa)

Personalizzazione · Libreria e dati · Integrazioni · Sistema e avanzate (schermi, sessioni,
lingua, backup, reset del layout, log, riavvio, reset dei dati). Ricerca tra le impostazioni.
