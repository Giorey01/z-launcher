# Linee guida di sviluppo Z-Launcher

Launcher Android open source, gamepad-first. Repo **pubblica**.

## Documenti
- `docs/PIANO.md`: **da leggere per primo.** Cosa costruiamo, dove va ogni cosa, l'ordine da
  seguire prima di scrivere codice e come si porta codice già scritto.
- `docs/local/EREDITA.md` (non versionato, se presente): quale codice già scritto e testato si
  porta e da dove. Mai copiarne nomi o percorsi in file versionati.
- `docs/ROADMAP.md`: requisiti funzionali e stato.
- `docs/spec/cocoon-theme.md`: formato dei temi (Cocoon e nostre estensioni).
- `docs/PIANO-POD.md`: piano dei Pod (moduli, formato, store, fasi); il contratto è in `spec/pods.md`.
- `docs/spec/pods.md`: il contratto dei Pod per chi estende il launcher.
- `docs/reference/cocoon-wiki.md`: comportamento di Cocoon da raggiungere.
- `docs/reference/cocoon-news.md`: funzioni annunciate nelle news di Cocoon, con il rimando alla roadmap.

## Regole
- **Un solo core.** La logica (libreria, piattaforme, lancio, temi, metadati, sessioni,
  salvataggi, database) sta nei crate Rust in `crates/`. Kotlin tiene solo ciò che Android
  impone: UI Compose, Intent, permessi, `UsageStatsManager`, audio, notifiche, host dei widget.
- **Database** SQLite in Rust (`rusqlite`) con migrazioni versionate. Niente Room.
- **Repo pubblica, senza dipendenze a valle.** Nessun codice né riferimento a progetti privati
  che usano il launcher. Il launcher espone punti di estensione generici (Pod esterni); chi li usa
  sta fuori da questa repo.
- **Clean-room su Cocoon.** Da Cocoon si prendono solo comportamento, documenti pubblici e i
  formati dei file dell'utente (temi ZIP). Mai codice decompilato, asset, suoni, testi, nome o
  logo. Le specifiche si scrivono in parole nostre in `docs/spec/`.
- **Licenze.** Codice solo da progetti MIT/Apache/BSD (con attribuzione in `NOTICE`); da GPL
  solo idee. Dati CC-BY-SA con attribuzione.
- **Prima di implementare** segui l'ordine di `docs/PIANO.md` §3: esiste già qui (codice,
  `git log`)? esiste nel codice ereditato? esiste in un progetto di riferimento? Solo dopo si
  scrive da zero.

## Convenzioni di codice
- Funzioni di al massimo 50 righe (`clippy.toml`); moduli divisi per responsabilità.
- `//!` in cima a ogni modulo, `///` su ogni elemento anche privato: attributi in cima a ogni crate,
  controllati da `scripts/strict-check.sh`. In Kotlin KDoc.
- Costanti regolabili solo in `zl-config`, per tema e commentate. I fatti dei formati restano
  accanto al parser.
- Il core manda eventi e codici strutturati, mai frasi: le traduzioni stanno nell'app
  (`strings.xml`, italiano e inglese).
- Casi condivisi in `fixtures/`, letti dai test Rust e da quelli Kotlin attraverso la FFI.
- Niente stub né test vuoti: ogni test deve poter fallire.

## Verifica (con `JAVA_HOME` su un JDK 17 e `ANDROID_HOME` impostati)
```bash
bash scripts/gate.sh
cd android && ./gradlew compileOssDebugKotlin testOssDebugUnitTest assembleOssDebug detekt
```

## Commit
Autore `Giorey01 <61801344+Giorey01@users.noreply.github.com>` (già nella config della repo).
