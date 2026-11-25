# Piano di lavoro per chi sviluppa (persone e agenti)

Questo file dice **come** si lavora sul launcher: cosa stiamo costruendo, dove va ogni cosa e
l'ordine da seguire prima di scrivere codice. Le regole di casa sono in `CLAUDE.md`, i requisiti
in `docs/ROADMAP.md`. Se qualcosa qui contraddice `CLAUDE.md`, vale `CLAUDE.md`.

## 1. Cosa stiamo costruendo

Un launcher Android open source, gamepad-first, con estetica da console. Il comportamento di
riferimento è quello di Cocoon (clean-room, vedi `CLAUDE.md`), con identità nostra.

- **Il launcher è il prodotto.** È completo senza nient'altro: libreria, lancio, temi, ore giocate,
  salvataggi, achievement, metadati.
- **Estendibile da fuori.** Una build di terze parti può aggiungere Pod (RF-73) che usano tema,
  suoni e libreria del launcher. Le dipendenze vanno sempre dall'esterno verso il launcher, mai il
  contrario: questa repo non sa chi la estende.
- **Un solo core in Rust.** Logica e database nei crate `crates/zl-*`; Kotlin solo per ciò che
  Android impone. La UI chiama il core attraverso `zl-ffi` (UniFFI).
- **Due varianti Gradle:** `oss` (questa repo, quella pubblicata) e `full` (composta altrove,
  riusa `zl-ffi` e aggiunge Pod). Niente in `oss` deve dipendere da `full`.

## 2. Dove va ogni cosa

| Crate | Responsabilità |
| --- | --- |
| `zl-config` | Ogni costante regolabile, commentata e divisa per tema |
| `zl-db` | SQLite (`rusqlite`), migrazioni versionate, repository tipizzati |
| `zl-library` | Scansione delle cartelle, multi-disco, rescan |
| `zl-platforms` | Piattaforme, emulatori, profili di lancio, scelta dell'emulatore |
| `zl-tags` | File tag `[tag] valore` |
| `zl-theme` | Formato temi, import Cocoon, layer, validatore |
| `zl-media` | Media già presenti accanto alla libreria (ES-DE, Cocoon), `gamelist.xml`, titoli a schermo |
| `zl-meta` | Orchestratore dei metadati e fonti online (una per file in `sources/`) |
| `zl-net` | *(da creare)* client HTTP sincrono condiviso: trait `HttpClient`, implementazione `ureq` con rustls, retry, backoff, `Retry-After`, limite per host, finto per i test. Lo useranno metadati, RetroAchievements, Steam, Theme Hub, gestore emulatori, mod |
| `zl-steam` | Steam: Web API ufficiale con la chiave dell'utente (giochi posseduti, nome del profilo) e `appdetails` pubblico dello store (dettagli del gioco, senza chiave) |
| `zl-sessions` | Tempo di gioco dagli eventi di utilizzo del sistema (in primo piano / in pausa), con pause e riprese |
| `zl-core` | Servizio applicativo: compone i crate, emette eventi strutturati |
| `zl-ffi` | Superficie UniFFI verso Kotlin |

Crate futuri previsti dalla roadmap: `layout`, `input` (sett. 3), `achievements`,
`saves` (sett. 5), `pcbridge` (sett. 7), `mods` (sett. 8). Si creano quando serve, non prima.

**Sincrono.** Il core è sincrono: niente `tokio` né `reqwest`. Il parallelismo, quando serve, sta
in `zl-core` con thread e canali. Il codice asincrono portato da altrove si converte.

## 3. Prima di scrivere una riga: l'ordine da seguire

Ogni funzione nuova passa per questi passi, **in quest'ordine**. Lo scopo è non reinventare.

1. **Esiste già qui?** Cerca in `crates/`, `android/`, `fixtures/` e in `git log --oneline`.
   Se esiste, estendila; non scriverne una seconda.
2. **Esiste nel codice ereditato?** Se c'è il file locale `docs/local/EREDITA.md` (non versionato,
   ce l'ha solo il maintainer), leggilo: dice quale codice già scritto e testato si porta, da dove
   e come. Se la funzione è lì, **si porta, non si riscrive**: stesse regole di scelta, stessi
   casi di test, stesse fixture. Si cambiano solo trasporto, nomi e dipendenze.
3. **Esiste in un progetto di riferimento?** Vedi la tabella al §5. Codice solo da MIT, Apache o
   BSD (con voce in `NOTICE`); da GPL solo idee e formati.
4. **Solo allora si scrive da zero**, partendo da una specifica in `docs/spec/` se tocca un
   formato o un comportamento di Cocoon.

Nel messaggio di commit e nella PR scrivi quale dei quattro casi hai seguito ("portato",
"esteso", "da riferimento X", "nuovo").

## 4. Come si porta codice ereditato

1. Copia il file e i suoi test; copia le fixture in `fixtures/<tema>/` senza cambiare i casi.
2. Sostituisci le dipendenze del progetto di origine con i crate `zl-*` e le costanti con
   `zl-config`. Il trasporto HTTP passa da `zl-net`.
3. Converti `async` in sincrono; i test con server HTTP finto diventano test con il client finto
   di `zl-net` che risponde dalle fixture.
4. **Ripulisci ogni riferimento all'origine:** nomi di crate, di file, di app, commenti come
   "twin of …", percorsi, chiavi di cache legate a cataloghi esterni. Nella repo non deve restare
   alcuna traccia del progetto di origine. Non si importa la sua history git.
5. Adegua alle convenzioni (`//!`, `///`, 50 righe, eventi al posto delle frasi) e fai passare
   `scripts/gate.sh`. I casi della fixture devono passare tutti: se uno non passa, il porting è
   sbagliato, non il caso.
6. Aggiungi ciò che manca sopra il codice portato, non al suo posto (esempio: un esito
   "ambiguo" sopra la regola di scelta esistente).

## 5. Progetti di riferimento pubblici

| Progetto | Licenza | Cosa prendere | Uso |
| --- | --- | --- | --- |
| Jarngreipr | MIT | Dual-screen su AYN Thor, display di lancio, Intent di ~110 emulatori, ordine di risoluzione | Codice |
| Xross | Apache-2.0 | Hash dentro zip, 7z e CHD, DAT No-Intro e Redump, Hasheous, shader AGSL, `emulators.json` | Codice |
| es-de-companion | MIT | Widget sul secondo schermo, override del lancio, media ES-DE | Codice |
| Daijishō | MIT (asset) | Database piattaforme e player, formato `.dpt` | Dati e formato |
| Playnite | MIT | Modello dati della libreria, plugin metadati, sessioni | Modello |
| rcheevos | MIT | Hash RetroAchievements, legato via FFI C | Libreria |
| libretro-database | CC-BY-SA-4.0 | Dat con CRC per il riconoscimento offline | Dati con attribuzione |
| Argosy, Arcoon, NeoStation, pocket-ui, Kvaesitso, Lemuroid, Steam ROM Manager | GPL | Sync salvataggi, migrazioni, host widget, libretro-db, artwork | Solo idee |

## 6. Stato e prossimi passi

Lo stato dei requisiti è in `docs/ROADMAP.md`. Ordine di lavoro attuale (settimana 2, metadati):

1. `zl-net` con trait, client `ureq`, retry e client finto.
2. SteamGridDB in `zl-meta/sources/steamgriddb.rs`: grids (copertine), heroes, logos, icons;
   scelta del gioco e dell'immagine **portate** (vedi §3 passo 2); verifica della chiave.
3. Import `gamelist.xml` di ES-DE (in corso in `zl-media`).
4. FFI e UI: chiave nelle impostazioni, "Scarica immagini", tab Info.
5. Poi le altre fonti di RF-18, una alla volta, ciascuna solo quando è reale e testata.

Niente stub: una fonte che non funziona davvero non entra nell'enum, nel registro né nel database.
