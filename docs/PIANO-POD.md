# Piano dei Pod

Piano completo per rendere il launcher modulare con i Pod (RF-73): cosa si costruisce, in che
ordine e con quali criteri di uscita. Il contratto di dettaglio vivrà in `docs/spec/pods.md` (v2,
fase 0). Questo file dice **come ci si arriva**. Se contraddice `CLAUDE.md`, vale `CLAUDE.md`.

Questa repo è pubblica e non nomina nessun Pod esterno né chi lo scrive: tutto qui è generico.

## 1. Principi

1. **Un Pod è un componente del front-end, non un'app a parte.** Disegna solo con i componenti
   del launcher (`pod-ui`), quindi segue tema, suoni, focus del pad e navigazione. Cambiare tema
   cambia anche i Pod.
2. **Un Pod porta la sua logica e i suoi dati.** Il core pubblico resta ciò che serve al launcher
   per essere completo da solo (libreria, lancio, temi, sessioni, metadati, import Steam).
   Mod, musica, salvataggi e simili sono Pod. Un Pod può portare codice Rust nel suo `.so`.
3. **Il Pod entra nell'interfaccia dagli slot**, punti di estensione tipizzati. Non tocca il
   codice interno e non disegna la cornice: la disegna il launcher.
4. **Codice scaricato, firmato, senza WebView.** Un Pod remoto è un file `.zlpod` che si installa
   da link o da uno store. Gira nel processo del launcher: la fiducia viene dalla firma e dal
   consenso dell'utente, non da una sandbox (§6).
5. **Il launcher non conosce i Pod.** La dipendenza va sempre dal Pod al launcher. I Pod integrati
   usano lo stesso contratto dei Pod esterni, così il contratto è provato ogni giorno.

## 2. Livelli

L'insieme di `pod-api`, `pod-ui`, `pod-platform`, plugin Gradle e `zpod-template` è il **Pod SDK**:
chi scrive un Pod dichiara una dipendenza e ha contratto, stile, build e firma. Vale per tutti,
autori esterni e maintainer.

```
┌──────────────────────── zpod-* (repo esterne) ───────────────────────┐
│ Pod: Kotlin/Compose con pod-ui + (facoltativo) .so Rust + asset       │
└──────────────┬──────────────────────────────────────────────────────┘
               │ compila contro
┌──────────────▼──────────────────────────────────────────────────────┐
│ pod-api       interfacce: Pod, PodHost, slot, eventi, servizi        │
│ pod-ui        componenti e template di schermata col tema            │
│ pod-platform  versioni delle librerie fornite dal launcher (BOM)     │
└──────────────┬──────────────────────────────────────────────────────┘
               │ implementati da
┌──────────────▼──────────────────────────────────────────────────────┐
│ :app          PodLoader (DexClassLoader), slot, schermata dei Pod    │
│ zl-pods       manifest, firma, registro, store, crash (Rust)         │
│ zl-net        HTTP condiviso (prerequisito)                          │
└─────────────────────────────────────────────────────────────────────┘
```

## 3. Cosa si crea

| Cosa | Dove | Responsabilità | Caso del `PIANO.md` §3 |
| --- | --- | --- | --- |
| `pod-api` | `android/pod-api` | Interfacce pubbliche e stabili, versionate con `apiVersion`; solo interfacce e tipi, nessuna logica | esteso (oggi in `app/.../pods/Pod.kt`) |
| `pod-ui` | `android/pod-ui` | Tema (`ZTheme`, pannelli, tipografia), componenti del pad (cursore, tile, bottoni, memoria del focus), elenco, griglia, tab, campi con tastiera a schermo, dialog, progresso, toast; template di schermata | esteso (si estrae da `ui/pad`, `ui/theme`, `ui/common`: nessun comportamento nuovo) |
| `pod-platform` | `android/pod-platform` | Piattaforma Gradle con le versioni di Compose, coroutines, Coil, WorkManager, serialization che il launcher fornisce | nuovo |
| Plugin Gradle `zlauncher.pod` | `android/build-logic` | Compila il Pod contro `pod-platform`, produce il dex, impacchetta e firma il `.zlpod` | nuovo |
| `pod-sample` | `android/pod-sample` | Pod di prova usato dai test strumentati. **Non entra nell'APK** | nuovo |
| `PodLoader` | `android/app/.../pods/` | Verifica, copia in sola lettura, carica con `DexClassLoader`, isola gli errori | esteso (`PodRegistry`) |
| Slot | `android/app/.../ui/` | Dove i Pod si agganciano, vedi §5 | nuovo |
| Schermata "Pod" | `android/app/.../ui/pods/` | Elenco installati, tasto **+**, store, aggiornamenti, disattivazione | nuovo |
| `zl-net` | `crates/zl-net` | HTTP condiviso; **già presente**, `zl-pods` lo usa | portato |
| `zl-pods` | `crates/zl-pods` | Vedi sotto | nuovo |
| Migrazione `zl-db` | `crates/zl-db` | Tabelle `pods`, `pod_trust`, `pod_health` | esteso |
| Costanti | `crates/zl-config` | Dimensione massima del `.zlpod`, soglia di crash, `apiVersion` corrente, indice predefinito | esteso |
| FFI | `crates/zl-ffi/src/pods_api.rs` | Installa, aggiorna, elenca, disattiva, firma | esteso |
| Fixture | `fixtures/pods/` | Manifest validi e non validi, `.zlpod` firmati e manomessi, indici | nuovo |

### `zl-pods`, un modulo per responsabilità

- `manifest.rs`: `manifest.json` e sue regole (id, versione, `apiVersion`, firmatario, slot,
  permessi dichiarati).
- `package.rs`: lettura dello zip, controllo degli hash e dei percorsi, limiti di dimensione.
- `signature.rs`: firma ed25519 sul contenuto; impronta del firmatario.
- `trust.rs`: chiavi accettate per id (pinning); un aggiornamento con altra chiave è rifiutato.
- `index.rs`: indice dello store (JSON) e confronto delle versioni.
- `registry.rs`: Pod installati, attivi, versioni; usa `zl-db`.
- `health.rs`: conteggio dei crash e disattivazione automatica.
- `install.rs`: scarica (via `zl-net`), verifica, installa; emette **eventi e codici**, mai frasi.

## 4. Formato `.zlpod` (sintesi; il dettaglio va in `docs/spec/pods.md`)

Uno zip con: `manifest.json`, `classes.dex`, facoltativi `lib/arm64-v8a/*.so`, `assets/`,
`i18n/`, e `SIGNATURE` (firma ed25519 dell'elenco dei file con i loro hash).
Il manifest dichiara `id`, `name`, `version`, `apiVersion` richiesta, `author`, `signer`,
l'elenco degli slot e i **permessi dichiarati** (mostrati all'utente, vedi §6).

## 5. Slot, eventi e servizi (sintesi)

**Slot** (il launcher disegna la cornice, il Pod dà dati e contenuto):

| Slot | Dove appare |
| --- | --- |
| `StartMenuEntry` | Voce nel menu Start (già esiste) |
| `PlatformGridLead(platform)` | Primo elemento della griglia di una console (il tasto **+**) |
| `GameActions(game)` | Bottoni nella scheda del gioco |
| `SettingsSection` | Sezione nelle impostazioni |
| `StatusWidget` | Widget nella barra della Home (per esempio un mini player) |
| `HomeShortcut` | Scorciatoia nella Home |

**Eventi** (dal launcher al Pod): gioco avviato, gioco terminato, libreria cambiata, tema
cambiato, Pod aggiornato.

**Servizi** (`PodHost`):
archivio per Pod (SQLite, preferenze, cartella file); segreti per Pod nel Keystore;
HTTP condiviso (`zl-net`); lavori in background e notifiche; cartelle SAF concesse; richiesta di
nuova scansione di una piattaforma; audio (abbassare o sostituire la musica del launcher);
libreria in sola lettura e lancio (già presenti).

## 6. Sicurezza

1. **Nessuna sandbox.** Il codice del Pod ha gli stessi permessi del launcher, chiavi comprese.
   I permessi del manifest sono un consenso mostrato all'utente, e l'interfaccia lo dice.
2. **Firma obbligatoria e pinning.** Alla prima installazione l'utente conferma autore e impronta;
   gli aggiornamenti passano solo con la stessa chiave. Hash dello zip confrontato con l'indice.
3. **Dex in sola lettura** (richiesto da Android 14+), in una cartella privata.
4. **Isolamento dei guasti.** Ogni slot e ogni `Content` sta in un `try/catch`; dopo N crash
   (costante in `zl-config`) il Pod si disattiva da solo; avvio in modalità sicura senza Pod.
5. **Compatibilità.** Un Pod con `apiVersion` più nuova di quella del launcher non si carica.
6. **Stile imposto.** Una regola detekt in `zpod-template` vieta `androidx.compose.material*` e
   colori o font scritti a mano. Non è una barriera di sicurezza, è una guardia per l'autore.
7. **F-Droid** può contestare il caricamento di codice scaricato. Va valutato prima di quella
   distribuzione; il caricamento dei Pod può stare dietro un'opzione.

## 7. Store e tasto +

- **Indice**: un `index.json` pubblico (id, nome, versione, URL del `.zlpod`, sha256, firmatario,
  `apiVersion`). L'indirizzo predefinito è una costante di `zl-config`.
- **Più indici, come i repository di add-on:** l'utente può aggiungere l'indirizzo di un altro
  indice o di un singolo Pod. Così un Pod resta installabile da link anche se non è in nessuno
  store pubblico, e questa repo non lo nomina.
- **Tasto +** nella schermata dei Pod: elenco dello store; campo per il link (con tastiera a
  schermo); apertura del link `zlauncher://pod?url=…`. Prima di installare: autore, impronta,
  permessi, dimensione.
- Il launcher non scarica nulla da sé: l'installazione parte solo da un gesto dell'utente.

## 8. Organizzazione delle repo

| Repo | Visibilità | Contenuto |
| --- | --- | --- |
| `z-launcher` | pubblica | Launcher, `pod-api`, `pod-ui`, `pod-platform`, plugin Gradle, `zl-pods` |
| `zpod-index` | pubblica | `index.json` e le Release con i `.zlpod` (il launcher li scarica senza token) |
| `zpod-template` | pubblica | Repo modello: Gradle, firma, regola detekt, workflow di rilascio |
| `zpod-<nome>` | privata | Un Pod per repo (o un solo `zpods` con un modulo per Pod, equivalente) |

Sul PC le repo stanno una accanto all'altra; in sviluppo ogni Pod usa
`includeBuild("../../z-launcher/android/…")`, in CI gli artefatti `pod-api`, `pod-ui` e
`pod-platform` pubblicati con versione. Il workflow di rilascio di un Pod compila, firma e carica
il `.zlpod` come Release in `zpod-index` e aggiorna l'indice. Un Pod che non va in un indice
pubblico si pubblica altrove con lo stesso workflow.

## 9. Fasi e criteri di uscita

Ogni fase si chiude solo con `bash scripts/gate.sh` e la verifica Android del `CLAUDE.md` verdi.

| Fase | Cosa | Uscita |
| --- | --- | --- |
| **0. Specifica** | **Fatta**: `docs/spec/pods.md` v2 (formato, slot, eventi, servizi, firma); `fixtures/pods/` generate da `scripts/gen_pod_fixtures.py` | Spec e fixture riviste; ogni regola della spec ha un caso in fixture |
| **1. Spike del caricamento** | Un dex con un `@Composable` caricato da file, in build `release` con R8, con un `.so` di prova, su emulatore o dispositivo. Branch usa e getta | **Go/no-go.** Se non regge (classloader, R8, Compose) si decide prima di costruirci sopra |
| **2. `zl-net`** | **Già fatto** (portato, `964e4c0`): trait, client `ureq`, retry, client finto | Nulla da fare; `zl-pods` lo usa |
| **3. `zl-pods` + DB + FFI** | Manifest, pacchetto, firma, trust, registro, health, indice | Fixture: `.zlpod` manomesso rifiutato, firma cambiata rifiutata, `apiVersion` troppo nuova rifiutata |
| **4. `pod-ui`** | Estrazione di tema e componenti del pad in modulo; `:app` ci dipende | Nessuna differenza di comportamento; test JVM esistenti passano; navigazione col pad invariata |
| **5. `pod-api`, loader e slot** | Contratto v2; `PodLoader`; slot uno alla volta; `pod-sample` | Test strumentati: il Pod di prova appare nel menu Start e come primo elemento di una griglia, e un suo crash non butta giù il launcher |
| **6. Schermata Pod e store** | Tasto **+**, indici, installazione, aggiornamento, disattivazione, deep link | Installazione e aggiornamento da indice di prova; rifiuto con chiave diversa |
| **7. Pod integrati** | Pod dello store dei temi (RF-50), poi Log (RF-42) e gli altri previsti, sul `BuiltinPodProvider` | Funzionano con lo stesso contratto dei Pod esterni, senza scorciatoie |
| **8. `zpod-template`, guida e CI** | Repo modello, guida per gli autori (stile, pad, esempi), workflow di rilascio, `zpod-index` | Un Pod nuovo si crea dal modello, si firma e si installa dal link |

I Pod esterni (le repo `zpod-*`) partono dopo la fase 6.

## 10. Test

- Rust: fixture condivise in `fixtures/pods/`, lette dai test di `zl-pods` e, attraverso la FFI,
  da quelli Kotlin. Ogni test deve poter fallire.
- Kotlin JVM: registro, slot, regole di caricamento.
- Strumentati (emulatore o dispositivo): caricamento reale di dex e `.so`, R8, crash isolato.
  Il `DexClassLoader` non funziona sulla JVM, quindi la fase 1 e la 5 non si possono coprire con
  test JVM. Dove non c'è un emulatore si prova sul dispositivo del maintainer e si scrive nel
  commit che è stato fatto così.

## 11. Rischi

| Rischio | Risposta |
| --- | --- |
| Il caricamento dex con Compose e R8 non regge | Fase 1 prima di tutto il resto |
| Aggiornare Compose nel launcher rompe i Pod | `pod-platform` fissa le versioni; `apiVersion` blocca i Pod incompatibili |
| Libreria in conflitto tra launcher e Pod | Il launcher ha la precedenza: elenco delle librerie fornite nella spec; il Pod include solo le altre |
| Un Pod malevolo | Firma, conferma dell'autore, avviso sui permessi; nessuna promessa di sandbox |
| `pod-ui` che copia il tema e diverge | Il launcher usa `pod-ui` per primo: stesso codice |
| F-Droid | Opzione per il caricamento; da decidere prima di quella distribuzione |

## 12. Da decidere

- Indice predefinito: quale indirizzo e chi lo cura.
- Se il caricamento dei Pod sta dietro un'opzione negli ordinari utenti (impostazione "Pod esterni").
- Nome e apiVersion iniziale (`1`).
- Un Pod per repo o un solo `zpods` con un modulo per Pod.
