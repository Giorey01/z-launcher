# Cocoon: interfaccia osservata su dispositivo

Osservazioni fatte usando Cocoon 3.11 su un AYN Thor (dual-screen) il 24 novembre 2026, solo
guardando lo schermo e navigando col controller. Descrivono il comportamento da raggiungere, con
un'identità visiva nostra. Gli screenshot restano fuori dalla repo (contengono asset di terzi).

## Ruoli degli schermi (Thor)

- **Schermo piccolo sotto (1080×1240):** la griglia e il dock. È lo schermo "principale" del
  launcher.
- **Schermo grande sopra (1920×1080):** wallpaper esterno e **hero** dell'elemento selezionato:
  per una cartella piattaforma, il logo/immagine della Smart Folder più un riquadro info
  ("Virtual Console · Nintendo DS · Released: 2004"); per un gioco, il logo del gioco grande sul
  wallpaper. Ci sono anche le pillole di stato: in alto a sinistra avatar e amici online, in alto
  a destra notifiche, Wi-Fi, batteria e ora; in basso a destra un indicatore con un pulsante info.
- Cocoon usa due activity, una per schermo (`MainActivity` e `ExternalDisplayActivity`).
- Le Impostazioni si aprono sullo schermo grande; quello piccolo mostra l'icona grande della
  categoria a fuoco e la versione.

## Home e griglia

- Dock in cima allo schermo della griglia: una fila di icone di app piccole e tonde.
- Griglia a 3 colonne di cartelle piattaforma (icone animate dal pacchetto Smart Folder). Il
  cursore è fatto di quattro angoli arrotondati luminosi che si animano attorno all'elemento.
- Con lo stile icona "Accent" gli elementi non selezionati sono desaturati e quello selezionato
  è a colori.
- Aprendo una cartella: intestazione con il nome della cartella, il tipo di ordinamento a destra
  ("Manual") e i suggerimenti dei bumper ai lati; griglia a 4–5 colonne. Le sottocartelle della
  cartella ROM diventano cartelle (icona cartella azzurra). Le celle vuote si vedono come
  riquadri tratteggiati.
- Copertine dei giochi quadrate, con cornice dell'icon overlay; sullo schermo sopra compare il
  logo del gioco.

## Menu

- **Y su un gioco:** pannello con l'anteprima del gioco a sinistra e le voci Launch › (sottomenu
  per scegliere lo schermo), Edit, Hide, Favorite, Add to Home. Alcune voci mostrano a destra il
  glifo del tasto scorciatoia.
- **A su un gioco** lo lancia subito.
- **Start:** Apps, New ›, Sort ›, Search, Edit Grid, Cocoon Settings.
- **RB:** pannello stato sullo schermo grande: toggle (Wi-Fi, Bluetooth, Non disturbare, un toggle
  di schermo, impostazioni), slider luminosità, elenco notifiche dove compaiono anche gli eventi
  del launcher ("Scraping interrupted, 56 di 64", "Incompatible Player: ROM format not supported",
  "4 new games found").
- **LB:** pannello social: tab Steam / Android, "Friends online" con avatar, nome e gioco in corso.

## Cassetto app e ricerca

- "All apps": una riga di app recenti, poi tutte in ordine alfabetico a 5 colonne, indice A–Z
  verticale a destra, pulsante ricerca in alto.
- La ricerca usa una **tastiera propria pensata per il controller** (QWERTY a tasti grandi,
  Paste, Accents con le varianti della lettera su una riga, maiuscole, ?1, conferma), con
  risultati live sopra.

## Impostazioni

Colonna di categorie a sinistra (Search, Personalization, Library & data, Integrations, System &
Advance, About) e lista a destra; ogni riga ha icona, titolo, sottotitolo e un controllo
(freccia per una sottopagina, menu a tendina, slider con − e +, interruttore, campioni di
colore). La lista è circolare: dal fondo si torna in cima.

Personalization, in ordine: Theme ›, Accent Color (11 campioni), Cursor style (tendina), Highlight
thickness (slider in %), Surface material (Glass/Solid), Glass Options ›, Icon style (Accent),
Wallpapers ›, Grid ›, App icon style ›, **App icon pack** (pacchetti di icone Android per le app),
Hero Settings ›, Dock style ›, Sound settings ›, Jingles settings ›, Animation style ›, Now playing
style ›, Fullscreen folders (interruttore), Show battery percentage (interruttore), Always show
corner hints (interruttore).

## Log della cartella dati

Cocoon scrive nella cartella dati log leggibili dall'utente (`launch_debug.log`,
`scan_debug.log`, log dell'applicazione dei temi). È una buona idea anche per noi: un log di
lancio con template, variabili risolte, intent e permessi concessi rende i problemi degli
emulatori diagnosticabili dall'utente (vedi `launch-observed.md`).
