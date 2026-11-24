# Specifica: temi in formato Cocoon

Specifica scritta in parole nostre (clean-room) dai documenti pubblici della wiki di Cocoon
(`cocoon-shell.com/wiki`: themes, theme-json-reference, theme-builder, icon-overlays,
wallpapers, audio-and-music, jingles; lette il 24 novembre 2026) e da cinque temi ZIP esportati
dalla community. Nessun codice né asset di Cocoon è stato guardato o copiato.

Z-Launcher usa questo formato come **formato proprio**: legge i temi Cocoon senza conversione e
scrive i suoi nello stesso layout, con le estensioni sotto la chiave `zl` (sezione 8).

## 1. Pacchetti e layer

Un tema è composto da **cinque layer**, che possono arrivare da pacchetti diversi:

| Layer | Cartella | Contenuto |
| --- | --- | --- |
| `base` | radice del pacchetto | `theme.json` (colori, impostazioni, musica), `wallpapers/`, `preview.png` |
| `icon_overlays` | `icon_overlays/<piattaforma>/` | `mask.png`, `overlay.png` |
| `smart_folders` | `smart_folders/<tipo>/`, `smart_folders/by_platform/<piattaforma>/` | `icon.png`, `hero.png`, `logo.png` |
| `sounds` | `sounds/` | un file per azione (sezione 5) |
| `music` | `music/` | tracce citate da `music_playlist` o `music_time_schedule` |

- Un tema base cita i pacchetti degli altri layer in `asset_refs`. Nei ZIP esportati i pacchetti
  citati **non sono inclusi**: arrivano a parte. Un riferimento senza pacchetto installato non è
  un errore, è un avviso ("pacchetto mancante").
- Precedenza: file dell'utente > layer scelto esplicitamente > tema base > default dell'app.
  Ogni pezzo ricade da solo (manca `hero.png` di una cartella → si usa il successivo nella
  catena, non si scarta la cartella).
- Nomi di cartella in minuscolo; l'id piattaforma è quello delle cartelle ROM (`gba`, `snes`,
  `psx`, …; elenco in `docs/reference/cocoon-wiki.md`).

## 2. Struttura dello ZIP

```
themes/<id>/theme.json
themes/<id>/wallpapers/main.<ext>
themes/<id>/wallpapers/external.<ext>
```

- `<id>` è uno slug del nome più un suffisso esadecimale di 6 cifre (`wiive-95ab15`). È l'id
  stabile del pacchetto: due versioni dello stesso tema hanno lo stesso id.
- Lo ZIP può contenere più cartelle sotto `themes/`. Va accettato anche uno ZIP con `theme.json`
  direttamente in radice o in un'unica cartella senza `themes/` (tolleranza, non errore).
- Sicurezza all'import: niente percorsi assoluti né `..`, niente link simbolici, limite di
  dimensione totale ed estratta (valori in `zl-config`).

## 3. `theme.json`

JSON stretto (niente commenti, niente virgole finali). Tutti i campi sono facoltativi tranne
`name`; un campo assente **lascia invariata** l'impostazione corrente dell'utente. Le chiavi
sconosciute si conservano e producono un avviso, mai un errore (i temi nuovi possono avere campi
che non conosciamo ancora).

### 3.1 Metadati

`name` (obbligatorio), `author`, `version` (stringa, es. `"1.1"`), `description` (può contenere
`\r\n`), `credits`, `website`.

### 3.2 `theme_mode`

`light` | `dark` | `oled` | `system`, **senza distinzione di maiuscole** (i temi reali usano il
minuscolo). Non vincola i colori: un tema `dark` può avere sfondo chiaro.

### 3.3 `color_scheme`

Oggetto di colori. Chiavi note:

`background_gradient_start`, `background_gradient_end`, `card_gradient_start`,
`card_gradient_end`, `text_primary`, `text_secondary`, `icon_tint`, `tile_background`,
`tile_border`, `toggle_off_gradient_start`, `toggle_off_gradient_end`,
`toggle_thumb_gradient_start`, `toggle_thumb_gradient_end`, `drop_shadow`,
`inner_shadow_light`, `inner_shadow_dark`, `success`, `warning`, `divider`,
`accent_gradient_start`, `accent_gradient_end`, `accent_glow`, `selection_gradient_start`,
`selection_gradient_end`.

Se mancano i colori di selezione, il cursore usa il gradiente d'accento.

**Sintassi dei colori** (la wiki dichiara `#RRGGBB` e `#AARRGGBB`, ma i temi reali usano altro):

| Forma | Esempio reale | Lettura |
| --- | --- | --- |
| `#RRGGBB` | `#34beed` | opaco |
| `#RRGGBBAA` | `#ffffff66` | ordine CSS, alfa in fondo |
| `rgba(r,g,b,a)` | `rgba(0,0,0,0.5)` | r,g,b 0–255, a 0–1 |
| `rgb(r,g,b)` | — | opaco (tolleranza) |
| `#RGB` | — | tolleranza |

Le 8 cifre si leggono in **ordine CSS** (`#RRGGBBAA`): nel tema Wiive `inner_shadow_light` vale
`#ffffff66`, che come `#AARRGGBB` sarebbe un giallo opaco, mentre come CSS è il bianco al 40%
che ci si aspetta da un'ombra chiara; il Theme Builder è un editor web. **Da confermare** con un
tema esportato dall'app con alfa diverso da `ff` nei primi due caratteri. Maiuscole e spazi dentro
`rgba( … )` sono tollerati. Un colore illeggibile è un avviso e la chiave viene ignorata.

### 3.4 Sfondi

`wallpaper_main` (schermo principale, sotto) e `wallpaper_external` (schermo esterno, sopra):
nome di file dentro `wallpapers/`, con o senza estensione. Formati visti: `png`, `webp`, `gif`,
`mp4`, `webm`. Se il campo manca si cerca `main` o `bottom` per il principale, `external` o
`top` per l'esterno, con qualsiasi estensione supportata. Un file citato e assente è un errore
del validatore.

### 3.5 `settings`

Applicate una volta, quando l'utente sceglie di applicare le impostazioni del tema.

| Chiave | Tipo | Valori |
| --- | --- | --- |
| `icon_roundness` | numero | 0.0–0.5 |
| `icon_scale` | numero | 0.5–1.0 |
| `hover_scale` | numero | visti 0.9 e 1; intervallo non documentato |
| `hero_display_style` | enum | `VIGNETTE`, `FULLSCREEN`, `NONE` |
| `hero_logo_scale`, `hero_folder_logo_scale` | numero | visti 0.35–0.65 |
| `hero_gradient_style` | enum | `NONE`, `BLACK_GRADIENT` |
| `hero_vignette_size` | numero | non documentato |
| `icon_style` | enum | `COLORED`, `ACCENT`, `FLAT` |
| `surface_material` | enum | `SOLID`, `GLASS` |
| `glass_blur` | numero | 0–10 |
| `glass_refraction`, `glass_highlight`, `glass_rim_strength`, `glass_tint` | numero | non documentati |
| `glass_on_tiles` | booleano | |
| `cursor_style` | enum | `ANIMATED`, `OUTLINED`, `ALTERNATIVE_ANIMATED`, `ALTERNATIVE` |
| `highlight_thickness` | numero | 0.5–1.0 (visto `0.65000004`: float da esportazione) |
| `dock_icon_shape` | enum | `CIRCLE`, `SQUIRCLE`, `ROUNDED_SQUARE`, `SQUARE` |
| `hide_dock_background`, `dock_on_top`, `corner_hints` | booleano | |

Enum senza distinzione di maiuscole. Un numero fuori intervallo è un avviso e viene **limitato**
all'intervallo; un enum sconosciuto è un avviso e la chiave viene ignorata.

### 3.6 Musica e volumi (al primo livello, non in `settings`)

- `music_mode`: `TIME` | `PLAYLIST` | `DISABLED`.
- `music_playlist`: percorsi relativi a `music/` separati da `||`.
- `music_time_schedule`: coppie `HH:MM|file`. Il separatore tra le coppie non è documentato:
  accettiamo `||` (come la playlist); **da confermare** su un pacchetto musica reale.
- `music_playback_mode`: `IN ORDER` | `SHUFFLE` (con lo spazio; tolleriamo `IN_ORDER`).
- `sfx_volume`, `music_volume`, `jingle_volume`: 0.0–1.0.

### 3.7 `asset_refs`

Lista di `{ id, name, category, version }`, con `category` tra `icon_overlays`,
`smart_folders`, `sounds`, `music`. Collega i pacchetti degli altri layer per id.

## 4. Icon overlay

`icon_overlays/<piattaforma>/mask.png` e/o `overlay.png`, stesse dimensioni, consigliato 512×512 o
più. La maschera è una forma bianca su trasparente: della copertina resta visibile solo dove la
maschera è opaca. L'overlay (cornice, decorazione, trasparente) si disegna **sopra** la copertina
mascherata. Ordine: copertina → maschera → overlay. La wiki non precisa se la maschera usa l'alfa
o la luminanza: usiamo l'alfa (**da confermare** su un pacchetto reale).

## 5. Suoni

File in `sounds/`, nome = azione, estensione tra `wav`, `mp3`, `ogg`, `m4a`, `flac`. Le 22 azioni:

`navigate`, `select`, `back`, `folder_open`, `folder_close`, `launch`, `error`, `notification`,
`discord_open`, `discord_close`, `screen_swap`, `grid_zoom_in`, `grid_zoom_out`,
`reorder_pickup`, `reorder_place`, `open_edit`, `pop`, `slider_increase`, `slider_decrease`,
`saving`, `saving_end`, `retroachievements`.

Un'azione senza file usa il suono di default; un file con nome sconosciuto è un avviso.

## 6. Smart Folders

`smart_folders/<tipo>/{icon,hero,logo}.png`, varianti per piattaforma in
`smart_folders/by_platform/<piattaforma>/`. I nomi dei tipi documentati sono `favorites` e
`recent`; gli altri (più giocati, mai giocati, nuovi, giochi Android, emulatori) **vanno
ricavati** da un pacchetto reale. Fino ad allora il lettore accetta qualsiasi nome di cartella e
lo riporta nel risultato.

## 7. Validatore

Produce una lista di problemi strutturati (codice, percorso JSON o file, gravità), mai frasi:
ogni app li traduce. Gravità: **errore** (il tema non si carica: JSON non valido, `name` assente,
sfondo citato e mancante) e **avviso** (chiave sconosciuta, colore illeggibile, valore limitato,
pacchetto citato non installato, suono con nome sconosciuto).

## 8. Estensioni Z-Launcher

I temi scritti da Z-Launcher usano lo stesso layout e aggiungono, se servono, una chiave `zl` al
primo livello di `theme.json` con le impostazioni che Cocoon non ha. Un tema senza `zl` è un tema
Cocoon puro. Il contenuto di `zl` si definisce quando serve, con un numero di versione
(`"zl": { "format": 1, … }`).

## 9. Domande aperte

1. Ordine delle 8 cifre esadecimali (3.3).
2. Separatore di `music_time_schedule` (3.6).
3. Maschera per alfa o per luminanza (4).
4. Nomi delle cartelle dei tipi di Smart Folder (6).
5. Layout interno degli ZIP dei pacchetti `icon_overlays`, `smart_folders`, `sounds`, `music`:
   servono pacchetti reali esportati.
