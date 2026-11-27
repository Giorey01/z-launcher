# Roadmap

Requisiti funzionali del launcher. **S.** è la settimana prevista (8 settimane: release 0.1 alla
fine della 4, 1.0 alla fine dell'8). **Stato**: vuoto = da fare. Gli ID non si rinumerano: un
requisito spostato cambia area e settimana, non numero.

Ordine delle settimane:

| S. | Tema |
| --- | --- |
| 1–2 | Chiudere libreria e metadati |
| 3 | Home e controller |
| 4 | Temi e audio |
| 5 | Slot per i Pod; Pod integrati Achievements e Saves |
| 6 | Dual-screen, widget, Pod Media |
| 7 | Steam e PC |
| 8 | Pannelli, backup, sync |

## Launcher

| ID | Area | Requisito | S. | Stato |
| --- | --- | --- | --- | --- |
| RF-01 | Libreria | Scansione delle cartelle per piattaforma, filtrata per estensione, su memoria interna, microSD e USB | 1 | Fatto su memoria interna (provato sul Thor); microSD e USB da provare |
| RF-02 | Libreria | Multi-disco (`.m3u`, una cartella per gioco, `(Disc N)`) raccolto in una sola voce | 1 | Fatto (`.m3u`, `.cue`/`.gdi`, `(Disc N)`, dischi nudi in una cartella) |
| RF-03 | Libreria | Rescan incrementale per data di modifica; i giochi spariti vengono segnalati, non cancellati | 1 | Parziale: rescan per impronta di percorso, dimensione e data; spariti segnalati e ripristinati; manca l'elenco dei giochi spariti nella UI |
| RF-04 | Libreria | Lettura dei file tag `[tag] valore` (`.steamappid`, `.localgameid`, `.dpt`, `.moonlight`, `.xcloud`) | 1 | Fatto (le cinque estensioni, UTF-16, errori per riga) |
| RF-05 | Libreria | Hash delle ROM in background, anche dentro zip, 7z e CHD (per abbinare i metadati; l'hash di RetroAchievements è RF-53) | 2 | Parziale: CRC32, MD5 e SHA1 in background (file, zip, 7z, header CHD; anche senza header per NES, SNES, Lynx, 7800, FDS); RAR no; non mostrati nella scheda |
| RF-06 | Libreria | Elenco delle app Android installate, con categoria giochi | 1 | Fatto |
| RF-75 | Libreria | App Android che sono giochi di una console (recomp, port) nella cartella di quella console, avviate come app | 2 | Fatto: `.localgameid` in ogni console, "Sposta in una console" dal cassetto |
| RF-76 | Libreria | Update, DLC e secondi dump di un gioco Switch uniti al gioco base | 1 | Fatto (per title id; un add-on senza base resta un gioco) |
| RF-07 | Lancio | Database JSON di piattaforme, emulatori e profili di lancio, aggiornabile senza nuovo APK | 1 | Fatto |
| RF-08 | Lancio | Ordine di scelta dell'emulatore: gioco, poi sistema, poi estensione, poi chiedi | 1 | Fatto |
| RF-09 | Lancio | Variabili nei profili, come `{rom}`, `{core}` e `{tags.NOME}` | 1 | Fatto |
| RF-10 | Lancio | Rilevamento degli emulatori installati e proposta automatica nell'onboarding | 1 | Fatto |
| RF-11 | Lancio | Lancio di giochi PC con profili per GameNative, GameHub, Winlator e DroidDeck | 7 | Parziale: profili di tutti e quattro, giochi Steam e Windows in una sola cartella "PC"; DroidDeck non provato su dispositivo |
| RF-12 | Lancio | Gestore emulatori: segnala gli aggiornamenti e scarica le release ufficiali | 7 |  |
| RF-13 | Organizzazione | Smart Folders: Preferiti, Recenti, Più giocati, Mai giocati, Nuovi, App, Emulatori, Per piattaforma | 2 | Fatto (anche Più giocati, Nuovi, App, Emulatori) |
| RF-14 | Organizzazione | Simple Mode, con tutti i giochi nella Home | 2 | Fatto |
| RF-15 | Organizzazione | Cartelle manuali e lo stesso gioco in più cartelle | 2 | Fatto |
| RF-16 | Organizzazione | Ricerca e ordinamenti; selezione multipla con sposta e rimuovi | 2 | Fatto (ricerca fuzzy, cinque ordinamenti per cartella o globali, selezione multipla con sposta, togli, nascondi) |
| RF-17 | Organizzazione | Nascondi gioco, preferito, valutazione | 2 | Fatto |
| RF-18 | Metadati | Orchestratore con fonti in ordine scelto: libretro-db, ScreenScraper, SteamGridDB, IGDB, LaunchBox, HLTB | 2 | Parziale: SteamGridDB (immagini, chiave dell'utente, scaricate da sole per i giochi nuovi dopo ogni scansione), Steam Store (dettagli e screenshot), ES-DE; niente orchestratore |
| RF-19 | Metadati | Tre modalità: solo mancanti, campi scelti, forza tutto | 2 | Parziale: solo mancanti (copertina, hero, logo) |
| RF-20 | Metadati | Campi protetti: una modifica manuale non viene sovrascritta | 2 | Parziale: campi protetti nel core; nella UI si modificano solo titolo e valutazione |
| RF-21 | Metadati | Import dei media già scaricati da ES-DE | 2 | Fatto (media e `gamelist.xml`) |
| RF-22 | Metadati | Modifica di nome, descrizione e artwork, con scelta tra le alternative | 2 | Parziale: modifica del titolo; descrizione e scelta dell'artwork no |
| RF-23 | Home | Griglia con zoom continuo e un layout salvato per ogni livello | 3 |  |
| RF-24 | Home | Page Mode con righe e colonne per pagina e frecce di scorrimento | 3 | Parziale: solo l'impostazione, senza effetto |
| RF-25 | Home | Stili a riga singola: Standard, Carousel, Cover Flow | 3 | Fatto |
| RF-26 | Home | Riordino manuale (prendi e posa) | 3 | Fatto (nelle cartelle manuali) |
| RF-27 | Home | Dock in basso e cassetto delle app | 1 | Fatto (dock con pin da cassetto con Y) |
| RF-28 | Home | Ruolo di Home del sistema | 1 | Fatto (richiesta del ruolo nel setup; l'esito dipende dall'utente) |
| RF-29 | Home | Onboarding: permessi, cartelle, modalità, emulatori | 1 | Fatto |
| RF-30 | Scheda gioco | Tab Info: descrizione, screenshot, trailer, badge HLTB, scelta di emulatore e display | 2 | Parziale: descrizione, dati, ore, screenshot, trailer in streaming, emulatore per gioco; HLTB e display no |
| RF-31 | Scheda gioco | Tab Activity: ore, sessioni, media, grafico settimanale | 3 | Fatto (totali, media, elenco, grafico delle ultime 12 settimane); non provato su dispositivo |
| RF-35 | Controller | Navigazione completa da D-Pad e stick, senza punti morti | 1 | Parziale: D-pad, A, B, Start provati; stick sinistro non provato |
| RF-36 | Controller | Mappa di default: A conferma, B indietro, Y menu, X scambia schermi, LB social, RB stato, Start impostazioni | 3 | Parziale: A, B, Y, Start (menu), LB/RB tra le piattaforme; X, pannelli e Start→impostazioni no |
| RF-37 | Controller | Glifi Nintendo, Xbox e PlayStation, automatici o manuali, e Corner Hints | 3 | Parziale: Corner Hints con le sigle; glifi per marca no |
| RF-38 | Controller | Remapping con profili per controller | 3 |  |
| RF-39 | Controller | Tastiera, touch e pinch come alternative | 3 | Parziale: tastiera a schermo per il pad, Invio, touch; pinch no |
| RF-40 | Sessioni | Tracciamento delle sessioni dall'uso delle app, al secondo, con pausa e ripresa | 3 | Fatto (stima dal lancio al ritorno senza accesso ai dati di utilizzo); ore Steam importate a parte |
| RF-41 | Sessioni | Modifica manuale delle sessioni ed esclusione di un'app | 3 | Fatto |
| RF-42 | Sessioni | Pod Log: totali, calendario, classifica | 3 | Fatto (Pod integrato `builtin.log` con totali, classifica giochi e avvio) |
| RF-43 | Sessioni | Now Playing sullo schermo secondario | 6 | Fatto (schermata Now Playing sull'altro display, con attenuazione) |
| RF-44 | Temi | Formato temi proprio e import dei temi Cocoon senza conversione manuale | 4 | Fatto (lettura del formato e import dalle cartelle di Cocoon) |
| RF-45 | Temi | Cinque layer componibili da pacchetti diversi; i file dell'utente hanno la precedenza | 4 | Fatto |
| RF-46 | Temi | Sfondi statici e animati (webp, png, gif, mp4), separati per schermo principale ed esterno | 4 | Fatto (immagini con Coil, video in loop muto); non provato su dispositivo |
| RF-47 | Temi | Superficie solida o in vetro, con parametri; ripiego dove la sfocatura non c'è | 4 | Fatto (sfocatura da API 31, tinta sotto) |
| RF-48 | Temi | Quattro stili di cursore con gradiente proprio; maschere e cornici delle icone per piattaforma | 4 | Fatto |
| RF-49 | Temi | Validatore con errori comprensibili | 4 | Parziale: validatore nel core e nella FFI; gli errori non si vedono nell'app |
| RF-50 | Temi | Theme Hub: catalogo da un indice su GitHub, anteprima, installazione, import ZIP | 4 | Fatto (Pod integrato `builtin.themehub` con catalogo e anteprime) |
| RF-51 | Audio | Effetti sonori a bassa latenza per le 22 azioni | 4 | Parziale: suoni propri sintetizzati per le 22 azioni, sostituibili dal tema, sul volume dei media; latenza non misurata |
| RF-52 | Audio | Musica di sottofondo del launcher: playlist, per ora del giorno o spenta; jingle per gioco; sfumatura quando parte un gioco | 4 | Parziale: playlist in ordine o casuale, brano per ora del giorno, musica propria se il tema non ne ha, pausa quando parte un gioco; jingle e sfumatura no |
| RF-61 | Dual-screen | Schermo sopra con wallpaper, hero e logo del gioco selezionato | 6 | Fatto (hero, logo e sfondo esterno); non provato su dispositivo |
| RF-62 | Dual-screen | Docking automatico, sempre o mai; scambio con X; display preferito per gioco, cartella o app | 6 | Parziale: modalità schermo, inversione dei ruoli, docking esterno; scambio con X e display preferito per gioco no |
| RF-63 | Dual-screen | Display esterno con fino a 5 righe | 6 | Parziale: da 1 a 5 righe solo nella modalità a schermo singolo |
| RF-77 | Dual-screen | Luminosità del secondo schermo indipendente dal primo | 6 | Fatto (20–100%, velo sullo schermo) |
| RF-64 | Widget | Orologio, Recenti, Ultimo giocato, Musica, Immagine, Log, Gioco casuale nella griglia | 6 |  |
| RF-65 | Widget | Host di AppWidget Android di terze parti | 6 |  |
| RF-66 | Widget | Pod Picnic: screenshot per gioco | 6 |  |
| RF-67 | Steam e PC | Libreria Steam, ore e artwork con la Web API ufficiale | 7 | Parziale: giochi posseduti e ore importati con la chiave dell'utente; artwork da SteamGridDB |
| RF-68 | Pannelli | Pannello stato (RB): Wi-Fi, Bluetooth, Non disturbare, luminosità, volume, notifiche | 3 |  |
| RF-69 | Pannelli | Pannello social (LB): amici Steam e Discord in sola lettura, conversazioni Android con risposta rapida | 8 |  |
| RF-71 | Sistema | Backup e ripristino di tutta la configurazione | 8 |  |
| RF-72 | Sistema | Sync in LAN verso il PC di salvataggi, screenshot e backup | 8 |  |
| RF-74 | Sistema | Interfaccia in italiano e inglese | 1 | Fatto |

## Estensioni (contratto dei Pod)

Ciò che il launcher deve offrire perché le funzioni più grandi stiano nei Pod. Il contratto è in
`docs/spec/pods.md`.

| ID | Area | Requisito | S. | Stato |
| --- | --- | --- | --- | --- |
| RF-73 | Estensioni | Pod esterni: una build terza può aggiungere Pod che usano tema, suoni e libreria del launcher | 4 | Fatto: contratto v2 (`Pod`, `PodHost`, `PodSlot`), `PodLoader` dex dinamico, store, firma Ed25519, `pod-platform`, `zpod-template`; API 2 con archivi (zip, 7z, RAR), download con ripresa e HTTP |
| RF-78 | Estensioni | Slot tab nella scheda del gioco: un Pod aggiunge una tab con il suo contenuto | 5 | Fatto (`GameTab`, API 3) |
| RF-79 | Estensioni | Slot badge: un Pod mostra un dato breve sulla scheda e sulla tile del gioco (per esempio "12/40") | 5 | Parziale: badge nella scheda; sulla tile no |
| RF-80 | Estensioni | Avviso del Pod con un suono del tema, anche al rientro da una sessione | 5 | Fatto (`PodHost.toast` con suono; `GameEnded` inviato al rientro) |
| RF-81 | Estensioni | Accesso dei Pod alle cartelle degli emulatori (dati e salvataggi), con il permesso dichiarato nel manifest | 5 |  |
| RF-82 | Estensioni | Servizio delle sessioni multimediali di Android: elenco dei lettori attivi, titolo, copertina, comandi | 6 |  |
| RF-83 | Estensioni | La musica del launcher cede il passo quando un Pod o un'altra app riproduce audio | 6 |  |

## Pod integrati

Pod pubblici che vivono in questa repo e usano lo stesso contratto dei Pod esterni.

| ID | Pod | Requisito | S. | Stato |
| --- | --- | --- | --- | --- |
| RF-32 | Achievements | Tab Achievements nella scheda del gioco (slot RF-78) | 5 |  |
| RF-53 | Achievements | Hash compatibile con RetroAchievements per le piattaforme supportate | 5 |  |
| RF-54 | Achievements | Trofei, percentuale, Hardcore e Softcore; abbinamento manuale | 5 |  |
| RF-55 | Achievements | Avviso con suono al rientro dalla sessione (RF-80) | 5 |  |
| RF-56 | Achievements | Achievement Steam dall'appid | 7 |  |
| RF-33 | Saves | Tab Saves nella scheda del gioco (slot RF-78) | 5 |  |
| RF-57 | Saves | Posizione dei salvataggi per emulatore, nei dati del Pod | 5 |  |
| RF-58 | Saves | Snapshot versionati con SHA-256, versioni fissate, snapshot a fine sessione | 5 |  |
| RF-59 | Saves | Ripristino sicuro (backup prima di sovrascrivere); import ed export ZIP o grezzi | 5 |  |
| RF-60 | Saves | Vault con vista per console; avviso per gli emulatori non raggiungibili | 5 |  |
| RF-84 | Media | Controllo di Spotify, YouTube Music e altri lettori dalle sessioni multimediali (RF-82): titolo, copertina, play, pausa, avanti | 6 |  |
| RF-85 | Media | Mini player nella barra della Home (slot `StatusWidget`) e widget Musica (RF-64) | 6 |  |

## Fuori dal launcher

Funzioni lasciate ai Pod esterni; il launcher fornisce solo gli slot.

| ID | Area | Requisito | Stato |
| --- | --- | --- | --- |
| RF-34 | Mod | Tab Mods nella scheda del gioco (slot RF-78) | Da un Pod esterno |
| RF-70 | Mod | Mod da GameBanana e cartelle mod degli emulatori; attiva o disattiva; conflitti | Da un Pod esterno |

## Requisiti non funzionali

- Avvio a freddo della Home sotto 1 s su Snapdragon 865; navigazione a 60 fps.
- Scansione di 5000 ROM sotto 10 s; rescan incrementale sotto 2 s.
- Suono di navigazione con latenza sotto 50 ms.
- Nessun servizio in primo piano mentre si gioca; sessioni ricostruite dagli eventi di sistema.
- Android 10 (API 29) o superiore, arm64; effetti in vetro pieni da API 33.
- Nessuna telemetria; chiavi API nel Keystore; tutto offline tranne scraping e servizi online.
- "Accesso a tutti i file" come modalità principale, SAF come ripiego.
- Legge e scrive formati aperti: temi Cocoon, media ES-DE, `.dpt` di Daijishō, `.m3u`.
- Distribuzione: APK firmato su GitHub Releases e Obtainium; F-Droid.
