# Roadmap

Requisiti funzionali del launcher. **S.** è la settimana prevista (8 settimane: release 0.1 alla
fine della 4, 1.0 alla fine dell'8). **Stato**: vuoto = da fare.

| ID | Area | Requisito | S. | Stato |
| --- | --- | --- | --- | --- |
| RF-01 | Libreria | Scansione delle cartelle per piattaforma, filtrata per estensione, su memoria interna, microSD e USB | 1 | Fatto su memoria interna (provato sul Thor); microSD e USB da provare |
| RF-02 | Libreria | Multi-disco (`.m3u`, una cartella per gioco, `(Disc N)`) raccolto in una sola voce | 1 |  |
| RF-03 | Libreria | Rescan incrementale per data di modifica; i giochi spariti vengono segnalati, non cancellati | 1 |  |
| RF-04 | Libreria | Lettura dei file tag `[tag] valore` (`.steamappid`, `.localgameid`, `.dpt`, `.moonlight`, `.xcloud`) | 1 |  |
| RF-05 | Libreria | Hash delle ROM in background, anche dentro zip, 7z e CHD | 2 |  |
| RF-06 | Libreria | Elenco delle app Android installate, con categoria giochi | 1 | Fatto |
| RF-07 | Lancio | Database JSON di piattaforme, emulatori e profili di lancio, aggiornabile senza nuovo APK | 1 | Fatto |
| RF-08 | Lancio | Ordine di scelta dell'emulatore: gioco, poi sistema, poi estensione, poi chiedi | 1 | Fatto |
| RF-09 | Lancio | Variabili nei profili, come `{rom}`, `{core}` e `{tags.NOME}` | 1 | Fatto |
| RF-10 | Lancio | Rilevamento degli emulatori installati e proposta automatica nell'onboarding | 1 | Fatto |
| RF-11 | Lancio | Lancio di giochi PC con profili per GameNative, GameHub, Winlator e DroidDeck | 7 | Parziale: profili di tutti e quattro, giochi Steam e Windows in una sola cartella "PC"; DroidDeck non provato su dispositivo |
| RF-12 | Lancio | Gestore emulatori: segnala gli aggiornamenti e scarica le release ufficiali | 7 |  |
| RF-13 | Organizzazione | Smart Folders: Preferiti, Recenti, Più giocati, Mai giocati, Nuovi, App, Emulatori, Per piattaforma | 2 | Parziale: Preferiti, Recenti, Mai giocati, Nascosti, Per piattaforma |
| RF-14 | Organizzazione | Simple Mode, con tutti i giochi nella Home | 2 |  |
| RF-15 | Organizzazione | Cartelle manuali e lo stesso gioco in più cartelle | 2 |  |
| RF-16 | Organizzazione | Ricerca e ordinamenti; selezione multipla con sposta e rimuovi | 2 | Parziale: ricerca fuzzy typo-tolerant e schermata di ricerca gamepad-first |
| RF-17 | Organizzazione | Nascondi gioco, preferito, valutazione | 2 | Fatto |
| RF-18 | Metadati | Orchestratore con fonti in ordine scelto: libretro-db, ScreenScraper, SteamGridDB, IGDB, LaunchBox, HLTB | 2 | Parziale: SteamGridDB (immagini, chiave dell'utente), Steam Store (dettagli e screenshot, gratis), ES-DE; niente orchestratore |
| RF-19 | Metadati | Tre modalità: solo mancanti, campi scelti, forza tutto | 2 |  |
| RF-20 | Metadati | Campi protetti: una modifica manuale non viene sovrascritta | 2 |  |
| RF-21 | Metadati | Import dei media già scaricati da ES-DE | 2 | Fatto (media e `gamelist.xml`) |
| RF-22 | Metadati | Modifica di nome, descrizione e artwork, con scelta tra le alternative | 2 |  |
| RF-23 | Home | Griglia con zoom continuo e un layout salvato per ogni livello | 3 |  |
| RF-24 | Home | Page Mode con righe e colonne per pagina e frecce di scorrimento | 3 |  |
| RF-25 | Home | Stili a riga singola: Standard, Carousel, Cover Flow | 3 |  |
| RF-26 | Home | Riordino manuale (prendi e posa) | 3 |  |
| RF-27 | Home | Dock in basso e cassetto delle app | 1 | Fatto (dock con pin da cassetto con Y) |
| RF-28 | Home | Ruolo di Home del sistema | 1 | Fatto (richiesta del ruolo nel setup; l'esito dipende dall'utente) |
| RF-29 | Home | Onboarding: permessi, cartelle, modalità, emulatori | 1 | Fatto |
| RF-30 | Scheda gioco | Tab Info: descrizione, screenshot, trailer, badge HLTB, scelta di emulatore e display | 2 | Parziale: descrizione, dati, ore, screenshot, trailer in streaming, emulatore per gioco; HLTB e display no |
| RF-31 | Scheda gioco | Tab Activity: ore, sessioni, media, grafico settimanale | 3 |  |
| RF-32 | Scheda gioco | Tab Achievements | 5 |  |
| RF-33 | Scheda gioco | Tab Saves | 5 |  |
| RF-34 | Scheda gioco | Tab Mods | 8 |  |
| RF-35 | Controller | Navigazione completa da D-Pad e stick, senza punti morti | 1 | Parziale: D-pad, A, B, Start provati; stick sinistro non provato |
| RF-36 | Controller | Mappa di default: A conferma, B indietro, Y menu, X scambia schermi, LB social, RB stato, Start impostazioni | 3 |  |
| RF-37 | Controller | Glifi Nintendo, Xbox e PlayStation, automatici o manuali, e Corner Hints | 3 |  |
| RF-38 | Controller | Remapping con profili per controller | 3 |  |
| RF-39 | Controller | Tastiera, touch e pinch come alternative | 3 |  |
| RF-40 | Sessioni | Tracciamento delle sessioni dall'uso delle app, al secondo, con pausa e ripresa | 3 | Fatto (stima dal lancio al ritorno senza accesso ai dati di utilizzo); ore Steam importate a parte |
| RF-41 | Sessioni | Modifica manuale delle sessioni ed esclusione di un'app | 3 |  |
| RF-42 | Sessioni | Pod Log: totali, calendario, classifica | 3 | Fatto (Pod integrato `builtin.log` con totali, classifica giochi e avvio) |
| RF-43 | Sessioni | Now Playing sullo schermo secondario | 6 |  |
| RF-44 | Temi | Formato temi proprio e import dei temi Cocoon senza conversione manuale | 4 |  |
| RF-45 | Temi | Cinque layer componibili da pacchetti diversi; i file dell'utente hanno la precedenza | 4 |  |
| RF-46 | Temi | Sfondi statici e animati (webp, png, gif, mp4), separati per schermo principale ed esterno | 4 |  |
| RF-47 | Temi | Superficie solida o in vetro, con parametri; ripiego sotto API 33 | 4 |  |
| RF-48 | Temi | Quattro stili di cursore con gradiente proprio; maschere e cornici delle icone per piattaforma | 4 |  |
| RF-49 | Temi | Validatore con errori comprensibili | 4 |  |
| RF-50 | Temi | Theme Hub: catalogo da un indice su GitHub, anteprima, installazione, import ZIP | 4 | Fatto (Pod integrato `builtin.themehub` con catalogo e anteprime) |
| RF-51 | Audio | Effetti sonori a bassa latenza per le 22 azioni | 4 | Parziale: suoni propri sintetizzati per le 22 azioni, sostituibili dal tema; latenza non misurata |
| RF-52 | Audio | Musica in playlist, per ora del giorno o spenta; jingle per gioco; sfumatura quando parte un gioco | 4 |  |
| RF-53 | Achievement | Hash compatibile con RetroAchievements per le piattaforme supportate | 5 |  |
| RF-54 | Achievement | Trofei, percentuale, Hardcore e Softcore; abbinamento manuale | 5 |  |
| RF-55 | Achievement | Avviso con suono al rientro dalla sessione | 5 |  |
| RF-56 | Achievement | Achievement Steam dall'appid | 7 |  |
| RF-57 | Salvataggi | Posizione dei salvataggi per emulatore nel database dei profili | 5 |  |
| RF-58 | Salvataggi | Snapshot versionati con SHA-256, versioni fissate, snapshot a fine sessione | 5 |  |
| RF-59 | Salvataggi | Ripristino sicuro (backup prima di sovrascrivere); import ed export ZIP o grezzi | 5 |  |
| RF-60 | Salvataggi | Pod Vault con vista per console; avviso per gli emulatori non raggiungibili | 5 |  |
| RF-61 | Dual-screen | Schermo sopra con wallpaper, hero e logo del gioco selezionato | 6 |  |
| RF-62 | Dual-screen | Docking automatico, sempre o mai; scambio con X; display preferito per gioco, cartella o app | 6 |  |
| RF-63 | Dual-screen | Display esterno con fino a 5 righe | 6 |  |
| RF-64 | Widget | Orologio, Recenti, Ultimo giocato, Musica, Immagine, Log, Gioco casuale nella griglia | 6 |  |
| RF-65 | Widget | Host di AppWidget Android di terze parti | 6 |  |
| RF-66 | Widget | Pod Picnic: screenshot per gioco | 6 |  |
| RF-67 | Steam e PC | Libreria Steam, ore e artwork con la Web API ufficiale | 7 | Parziale: import dei giochi posseduti con la chiave dell'utente; ore non ancora salvate |
| RF-68 | Pannelli | Pannello stato (RB): Wi-Fi, Bluetooth, Non disturbare, luminosità, volume, notifiche | 3 |  |
| RF-69 | Pannelli | Pannello social (LB): amici Steam e Discord in sola lettura, conversazioni Android con risposta rapida | 8 |  |
| RF-70 | Mod | Mod da GameBanana e cartelle mod degli emulatori; attiva o disattiva; conflitti | 8 |  |
| RF-71 | Sistema | Backup e ripristino di tutta la configurazione | 3 |  |
| RF-72 | Sistema | Sync in LAN verso il PC di salvataggi, screenshot e backup | 8 |  |
| RF-73 | Estensioni | Pod esterni: una build terza può aggiungere Pod che usano tema, suoni e libreria del launcher | 4 | Fatto: contratto v2 (`Pod`, `PodHost`, `PodSlot`), `PodLoader` dex dinamico, store, firma Ed25519, `pod-platform`, `zpod-template` |
| RF-74 | Sistema | Interfaccia in italiano e inglese | 1 | Fatto |
| RF-75 | Libreria | App Android che sono giochi di una console (recomp, port) nella cartella di quella console, avviate come app | 2 | Fatto: `.localgameid` in ogni console, "Sposta in una console" dal cassetto |
| RF-76 | Controller | Cursore che scivola senza ritardo e senza superare l'elemento; la lista scorre solo vicino ai bordi, con margine; l'inerzia si ferma al primo tasto | 3 |  |
| RF-77 | Home | Stile e velocità delle animazioni regolabili; menu e dialoghi che si trasformano l'uno nell'altro | 4 |  |
| RF-78 | Home | Scorciatoie ai Pod e alle impostazioni nella griglia, che si tolgono e si rimettono dal menu Start | 3 |  |
| RF-79 | Home | Modifica della griglia tenendo premuto A: sposta, ridimensiona (anche i giochi), le vicine fanno spazio, X toglie con annulla | 3 |  |
| RF-80 | Organizzazione | Griglia "Aggiungi giochi" con i giochi non ancora nella Home, scelti in blocco | 3 |  |
| RF-81 | Sessioni | Gioco sospeso con Home: resta indicato nella Home, A riprende, X chiude (anche le app accanto); emulatore crashato gestito | 6 |  |
| RF-82 | Lancio | Animazione di lancio: l'artwork cresce fino a riempire gli schermi; al ritorno si torna sul gioco | 4 |  |
| RF-83 | Sistema | Prestazioni sui dispositivi lenti: baseline profile, artwork animati spegnibili, animazioni ferme mentre si gioca | 4 |  |
| RF-84 | Scheda gioco | Logo spostabile e ridimensionabile sopra l'hero, per gioco | 4 |  |
| RF-85 | Libreria | Scorciatoie fissate di altre app usate come giochi, con piattaforma assegnabile | 7 |  |
| RF-86 | Home | Icona delle app personalizzabile (icona del launcher colorata o da icon pack), anche solo nel dock | 4 |  |
| RF-87 | Sistema | Permessi delle cartelle persi rilevati con guida per ridarli; avvio ritardato dopo il riavvio | 3 |  |
| RF-88 | Sistema | Aggiornamento dal launcher stesso, con le novità mostrate prima di installare | 8 |  |
| RF-89 | Estensioni | Pod manager e store nello stile delle altre schermate: card grandi, schede con LB/RB, dettaglio con anteprime, azioni sui tasti | 4 |  |
| RF-90 | Estensioni | Collegamento a un server RomM (sfoglia, scarica, cancella), come Pod | 8 |  |

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
