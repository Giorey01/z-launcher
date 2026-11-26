# Pod: il contratto (versione 2, `api` = 1)

Un **Pod** è un componente che si aggiunge al front-end del launcher: ha schermate e logica
proprie, ma si disegna solo con i componenti del launcher e vive dentro il suo processo. Può
essere **integrato** (compilato nel launcher) o **esterno** (un file `.zlpod` installato
dall'utente). Il contratto è lo stesso: il Pod SDK (`pod-api`, `pod-ui`, `pod-platform`, plugin
Gradle) è l'unica cosa che un Pod conosce del launcher.

Il piano di lavoro è in `docs/PIANO-POD.md`. Questa repo non contiene Pod esterni né ne nomina:
la dipendenza va sempre dal Pod al launcher.

Nelle tabelle, i **codici** (`BAD_ID`, `SIGNATURE_INVALID`…) sono i valori che il core manda
all'app: l'app li traduce (`strings.xml`), il core non scrive frasi. Ogni regola di questo
documento ha almeno un caso in `fixtures/pods/` (§12).

## 1. Ruoli

| Termine | Significato |
| --- | --- |
| Host | Il launcher: carica i Pod, disegna gli slot, offre i servizi |
| Pod | Una schermata (o più) più la logica che la serve, con un `id` stabile |
| Slot | Punto dell'interfaccia in cui un Pod può agganciarsi; la cornice la disegna l'host |
| Route | Nome di una schermata del Pod più argomenti: dove porta uno slot |
| Servizio | Capacità che l'host offre al Pod (`PodHost`) |
| Evento | Notifica dall'host al Pod (gioco avviato, tema cambiato…) |
| Firmatario | Chi ha firmato il `.zlpod`: una chiave ed25519 e la sua impronta |

## 2. Il pacchetto `.zlpod`

Un file zip. Contenuto ammesso:

| Percorso | Obbligo | Note |
| --- | --- | --- |
| `manifest.json` | sì | §3 |
| `classes.dex` | sì | Il codice del Pod (Kotlin/Compose compilato con il plugin Gradle) |
| `SIGNATURE` | sì | §4 |
| `lib/<abi>/*.so` | no | Librerie native; l'`<abi>` deve stare in `abis` del manifest |
| `assets/**` | no | File del Pod |
| `i18n/<lingua>.json` | no | Testi del Pod: oggetto `chiave → testo`; `en` è il ripiego |

**Regole sui percorsi** (si controllano prima di leggere un solo byte del contenuto):

- Percorso relativo, con `/`, senza `..`, senza segmento vuoto, senza `\`, senza `/` iniziale:
  altrimenti `BAD_PATH`.
- Solo file normali (niente collegamenti simbolici); le cartelle vuote si ignorano.
- Solo le voci della tabella: un'altra voce di primo livello è `UNEXPECTED_ENTRY`.
- Due voci che differiscono solo per le maiuscole sono `DUPLICATE_PATH`.
- Un file sotto `lib/<abi>/` con `<abi>` non dichiarato nel manifest è `LIB_NOT_DECLARED`.
- Manca `manifest.json` o `classes.dex`: `MISSING_FILE`. Manca `SIGNATURE`: `SIGNATURE_MISSING`.
- Un file che non è uno zip: `NOT_A_ZIP`.

**Limiti** (costanti in `zl-config::pods`, stessa logica di quelli dei temi):

| Limite | Valore | Codice |
| --- | --- | --- |
| Voci in un pacchetto | 2000 | `TOO_MANY_ENTRIES` |
| Dimensione del file `.zlpod` | 128 MiB | `PACKAGE_TOO_LARGE` |
| Dimensione espansa totale | 512 MiB | `PACKAGE_TOO_LARGE` |
| Dimensione espansa di una voce | 256 MiB | `ENTRY_TOO_LARGE` |
| Rapporto espanso / compresso di una voce | 200 | `ENTRY_RATIO` |
| …ignorato sotto questa dimensione espansa | 1 MiB | |
| Dimensione di `manifest.json` | 64 KiB | `ENTRY_TOO_LARGE` |

I limiti si applicano alle dimensioni dichiarate dallo zip **e** a quelle realmente lette.

**Ordine dei controlli.** Il lettore si ferma alla prima fase che trova problemi e li riporta
tutti (le fasi servono a non fidarsi del contenuto prima di averlo verificato):

1. lo zip si apre (`NOT_A_ZIP`);
2. percorsi e limiti (`BAD_PATH`, `UNEXPECTED_ENTRY`, `DUPLICATE_PATH`, `TOO_MANY_ENTRIES`,
   `PACKAGE_TOO_LARGE`, `ENTRY_TOO_LARGE`, `ENTRY_RATIO`);
3. file obbligatori (`MISSING_FILE`, `SIGNATURE_MISSING`);
4. firma (`SIGNATURE_MALFORMED`, `SIGNATURE_INVALID`);
5. manifest (§3);
6. coerenza tra manifest e pacchetto (`SIGNER_MISMATCH`, `LIB_NOT_DECLARED`).

## 3. `manifest.json`

```json
{
  "schema": 1,
  "id": "sample.hello",
  "name": "Hello",
  "version": "1.2.0",
  "api": 1,
  "author": { "name": "Esempio", "signer": "<64 cifre esadecimali>" },
  "entry": "dev.example.hello.HelloProvider",
  "description": "Un Pod di prova.",
  "slots": ["start_menu", "platform_grid_lead"],
  "permissions": ["library.read", "library.rescan"],
  "abis": ["arm64-v8a"],
  "homepage": "https://example.com/hello"
}
```

Campi sconosciuti si ignorano (compatibilità in avanti). I valori sono questi:

| Campo | Regola | Codice |
| --- | --- | --- |
| (file) | JSON valido, oggetto | `MANIFEST_INVALID_JSON` |
| `schema` | obbligatorio, intero, `1` | `MISSING_FIELD`, `WRONG_TYPE`, `UNSUPPORTED_SCHEMA` |
| `id` | obbligatorio; minuscole, cifre, `.`, `-`, `_`; 1–64 caratteri | `MISSING_FIELD`, `BAD_ID` |
| `name` | obbligatorio; 1–40 caratteri, senza controlli | `MISSING_FIELD`, `BAD_NAME` |
| `version` | obbligatorio; `MAJOR.MINOR.PATCH`, tre numeri interi senza segno (niente prerelease) | `BAD_VERSION` |
| `api` | obbligatorio, intero; deve stare tra `API_MIN` e `API_CURRENT` (oggi `1` e `1`) | `API_TOO_NEW`, `API_TOO_OLD` |
| `author.name` | obbligatorio; 1–60 caratteri | `MISSING_FIELD` |
| `author.signer` | obbligatorio; impronta intera, 64 cifre esadecimali minuscole | `BAD_SIGNER` |
| `entry` | obbligatorio; nome di classe Java pienamente qualificato che implementa `PodProvider` | `BAD_ENTRY` |
| `description` | facoltativo; fino a 200 caratteri | `WRONG_TYPE` |
| `slots` | obbligatorio (anche vuoto); tra i nomi di §7; senza ripetizioni | `UNKNOWN_SLOT`, `DUPLICATE_VALUE` |
| `permissions` | obbligatorio (anche vuoto); tra i nomi di §8; senza ripetizioni | `UNKNOWN_PERMISSION`, `DUPLICATE_VALUE` |
| `abis` | obbligatorio se il pacchetto ha `lib/`; oggi solo `arm64-v8a` | `UNKNOWN_ABI` |
| `homepage` | facoltativo; URL `https` | `BAD_URL` |

Un `id` che coincide con quello di un Pod integrato non si installa (`DUPLICATE_ID`).
Il tipo sbagliato di un campo (testo dove va un numero…) è `WRONG_TYPE`.

## 4. Firma e fiducia

Il file `SIGNATURE` è JSON:

```json
{ "algorithm": "ed25519", "publicKey": "<base64, 32 byte>", "signature": "<base64, 64 byte>" }
```

**Messaggio firmato**: il testo UTF-8 formato dalla riga `zlpod-v1`, poi una riga per ogni file
del pacchetto **tranne** `SIGNATURE`, in ordine crescente per i byte del percorso, nella forma
`<sha256 del contenuto in minuscole esadecimali>␠␠<percorso>`; ogni riga termina con `\n`.
Aggiungere, togliere o cambiare un file cambia il messaggio, quindi invalida la firma
(`SIGNATURE_INVALID`).

- `SIGNATURE` mal formato, algoritmo diverso o lunghezze sbagliate: `SIGNATURE_MALFORMED`.
- **Impronta** della chiave: SHA-256 della chiave pubblica di 32 byte, in esadecimale minuscolo
  (64 cifre). All'utente si mostrano le prime 32 cifre in maiuscolo, a gruppi di 4.
- `author.signer` del manifest deve essere l'impronta della chiave che ha firmato; altrimenti
  `SIGNER_MISMATCH`. Il manifest è dentro il messaggio firmato, quindi `signer` è autenticato.
- **Fiducia alla prima installazione**: l'utente vede nome, versione, autore, impronta,
  permessi e dimensione, e conferma. L'impronta si salva (`pod_trust`) per quell'`id`.
- **Aggiornamenti**: ammessi solo se il firmatario è lo stesso (`SIGNER_CHANGED` altrimenti).
  Disinstallare il Pod cancella anche l'impronta: è l'unico modo di cambiare firmatario, ed è
  una scelta esplicita dell'utente.
- Non esistono Pod "non firmati": un pacchetto senza firma valida non si installa.

## 5. Indice dello store

Un file JSON pubblico. L'indirizzo predefinito è una costante di `zl-config::pods`; l'utente può
aggiungere altri indici (o un solo Pod da link): così un Pod resta installabile anche se non sta in
nessun indice pubblico.

```json
{
  "schema": 1,
  "name": "Nome dell'indice",
  "updated": "2026-11-25",
  "pods": [
    {
      "id": "sample.hello", "name": "Hello", "version": "1.2.0", "api": 1,
      "description": "Un Pod di prova.",
      "url": "https://example.com/hello-1.2.0.zlpod",
      "sha256": "<64 cifre>", "size": 123456, "signer": "<64 cifre>"
    }
  ]
}
```

- `schema` diverso da `1`: `UNSUPPORTED_SCHEMA`; JSON non valido: `INDEX_INVALID_JSON`.
- Il lettore è tollerante per voce: una voce con un problema si scarta con il suo codice e le
  altre restano. Codici per voce: `BAD_ID`, `BAD_VERSION`, `BAD_URL` (solo `https`), `BAD_HASH`
  (`sha256` e `signer`: 64 cifre esadecimali), `MISSING_FIELD`, `DUPLICATE_ID` (resta la prima).
- Una voce con `api` più nuova di `API_CURRENT` resta nell'elenco segnata `API_TOO_NEW`
  (non installabile); una con `api` troppo vecchia, `API_TOO_OLD`.
- All'installazione da indice, lo `sha256` del file scaricato deve coincidere (`HASH_MISMATCH`)
  e il firmatario del pacchetto deve essere quello dell'indice (`SIGNER_MISMATCH`).
  Da link diretto non c'è un hash atteso: vale la firma e la conferma dell'utente.

## 6. Installazione, aggiornamento, stato

**Due passi.** `inspect(file)` non cambia nulla: restituisce un'anteprima (manifest, impronta,
permessi, dimensione, esito rispetto a ciò che è installato). `install(file)` lo ripete,
poi installa. Esiti dell'anteprima:

| Esito | Quando |
| --- | --- |
| `NEW` | `id` non installato |
| `UPDATE` | stesso `id`, stesso firmatario, versione maggiore |
| `SAME_VERSION` | stesso `id` e stessa versione: nulla da fare |
| `DOWNGRADE` | stessa `id`, versione minore: rifiutato |
| `SIGNER_CHANGED` | stesso `id`, altro firmatario: rifiutato |

Le versioni si confrontano per numero, componente per componente (`1.10.0` > `1.9.0`).

**Atomicità.** Il pacchetto si verifica per intero in una cartella temporanea e si sposta in
`pods/<id>/<versione>/` solo se tutto è valido; la versione precedente resta finché la nuova non è
a posto. Il nuovo codice si carica al **prossimo avvio** del launcher, mai a caldo. Il file
`classes.dex` si rende di sola lettura (Android 14+).

**Stato di un Pod installato** (tabella `pods`):

| Stato | Significato |
| --- | --- |
| `active` | caricato all'avvio |
| `disabled_by_user` | spento dall'utente |
| `disabled_by_crashes` | spento dal launcher dopo troppi guasti (§10) |
| `incompatible` | `api` non più supportata dal launcher (dopo un suo aggiornamento) |

**Opzione "Pod esterni".** Impostazione del launcher, **spenta di default**. Con l'opzione spenta i
Pod esterni installati non si caricano e i loro slot non appaiono; i Pod integrati funzionano
sempre. Il tasto **+** spiega e propone di accendere l'opzione; un link `zlauncher://pod?url=…`
aperto a opzione spenta chiede prima di accenderla. Nulla si scarica da solo: l'installazione parte
sempre da un gesto dell'utente.

## 7. Slot

Ogni slot è un dato che il Pod dichiara e che l'host disegna col suo stile. Aprendo uno slot
l'host chiama `Pod.Content(host, route)`. Il manifest elenca i tipi di slot che il Pod userà
(`slots`); uno slot non dichiarato non si disegna.

| Nome nel manifest | Dove appare | Dati dichiarati |
| --- | --- | --- |
| `start_menu` | Voce nel menu Start | etichetta, icona, route |
| `platform_grid_lead` | **Primo elemento** della griglia di una console (il tasto **+**) | piattaforme (insieme di id, o tutte), etichetta, icona, route in funzione della piattaforma |
| `game_actions` | Bottoni nella scheda del gioco | etichetta, icona, condizione sul gioco, route in funzione del gioco |
| `settings_section` | Riga nelle impostazioni | etichetta, route |
| `status_widget` | Widget nella barra della Home (per esempio un mini player) | contenuto Compose compatto, larghezza massima |
| `home_shortcut` | Scorciatoia nella Home | etichetta, icona, route |

**Regole di disegno**

- Più Pod sullo stesso slot: ordine di installazione. In una griglia, al massimo
  `MAX_LEAD_TILES` (3) elementi iniziali; i successivi non si mostrano.
- Al massimo `MAX_SLOTS_PER_POD` (16) dichiarazioni per Pod; le altre si ignorano.
- Il focus del pad entra negli elementi come in quelli del launcher, con gli stessi suoni.
- Un errore mentre si dichiara o si disegna uno slot conta come guasto del Pod (§10) e lo slot
  sparisce; il resto della schermata non cambia.

## 8. Permessi e servizi (`PodHost`)

Sempre disponibili: `theme`, `play(action)`, `strings`, `storage`, `close()`, `navigate(route)`,
`events`. Gli altri richiedono il permesso nel manifest.

| Permesso | Servizio |
| --- | --- |
| `library.read` | `folders()`, `games(folder)`, `details(id)`, `playStats(id)`, eventi di gioco |
| `library.rescan` | `rescan(platformId)`: nuova scansione (oggi di tutta la libreria); le schermate aperte si aggiornano e arriva `LibraryChanged` |
| `launch` | `launch(game)` con la stessa scelta di emulatore, display e conteggio del tempo |
| `secrets` | `getSecret`/`setSecret`: valori cifrati con una chiave del Keystore, con nomi separati per Pod (`pod.<id>.<chiave>`) |
| `http` | richieste HTTP con retry, backoff e limiti per host (`zl-net`) |
| `work` | `work.start(id, titolo) { … }`: lavori lunghi nel processo del launcher, tenuti vivi da un servizio in primo piano con una notifica di avanzamento; un lavoro che lancia un'eccezione conta come guasto; i lavori non sopravvivono alla fine del processo (il Pod tiene la sua coda) |
| `notifications` | notifiche del Pod, con canale proprio |
| `folders` | `platformRoot(id)`: la cartella della console nella libreria (percorso assoluto: quella di una cartella ROM che ne contiene i giochi, altrimenti quella col nome della piattaforma o di un alias); `pick()` (da fare): cartella scelta dall'utente e ricordata per il Pod |
| `audio` | richiesta di focus audio; abbassare o sostituire la musica del launcher |

- `storage`: una cartella di file privata e una di cache, più preferenze chiave-valore. Niente
  altro è raggiungibile da un altro Pod.
- `strings`: i testi di `i18n/<lingua>.json` secondo la lingua dell'app, con ripiego su `en` e poi
  sulla chiave stessa.
- **I permessi sono un consenso, non una barriera**: il codice del Pod gira nel processo del
  launcher con gli stessi diritti. L'host non consegna un servizio non dichiarato (l'accesso
  lancia `PermissionNotDeclared`), ma un Pod ostile non è fermato da questo. L'interfaccia di
  installazione lo dice in chiaro (§11).

**Eventi** (`PodHost.events`): `GameStarted(game)`, `GameEnded(game, seconds)`,
`LibraryChanged(platformIds)`, `ThemeChanged(theme)`. Quelli di gioco e libreria richiedono
`library.read`. Un'eccezione nel gestore di un evento conta come guasto.

## 9. Il contratto Kotlin

Pacchetto `dev.zlauncher.pods` (modulo `pod-api`). Forma di riferimento; i dettagli si fissano
nella fase 5 senza cambiare le regole di questo documento.

```kotlin
interface Pod {
    val id: String                                   // come nel manifest
    val icon: ImageVector
    fun title(context: Context): String
    fun slots(): List<PodSlot>                       // §7
    @Composable fun Content(host: PodHost, route: PodRoute)
}
interface PodProvider { fun pods(): List<Pod> }      // costruttore pubblico senza argomenti
data class PodRoute(val screen: String = "main", val args: Map<String, String> = emptyMap())
```

`PodHost` aggiunge a quello attuale (`theme`, `play`, `folders`, `games`, `details`,
`playStats`, `launch`, `close`) i servizi della §8. Il contratto attuale resta valido: un Pod
senza slot funziona come oggi, e `Content(host)` diventa `Content(host, route)` con la route
`main` per la voce del menu Start.

**Caricamento di un Pod esterno**: copia in `pods/<id>/<versione>/`, `classes.dex` di sola
lettura, `DexClassLoader` con il classloader del launcher come padre, libreria nativa da
`lib/<abi>/`, istanza della classe `entry`. Il launcher ha la precedenza sulle librerie: il Pod
**non include** quelle fornite da `pod-platform` (Compose, coroutines, Coil, WorkManager,
serialization, `pod-api`, `pod-ui`); include le altre (per esempio Room). R8 conserva per intero, nomi e membri, `dev.zlauncher.pods`,
`dev.zlauncher.ui` e le librerie fornite (regole in `app/proguard-rules.pro`): il dex di un Pod usa
parti che il launcher non usa, e senza queste regole fallisce solo nelle build release.
I Pod integrati si registrano con `ServiceLoader`, come oggi.

**Librerie native con UniFFI.** Il `DexClassLoader` riceve `lib/<abi>/` come cartella delle librerie,
ma JNA carica per nome e non guarda il classloader. Un Pod con binding UniFFI chiede il percorso al
proprio classloader (`BaseDexClassLoader.findLibrary(nome)`) e lo passa ai binding con la proprietà
di sistema `uniffi.component.<nome>.libraryOverride` prima della prima chiamata.

**Un Pod che non si carica** (eccezione, `LinkageError` per una classe o un metodo che il launcher
non ha) conta come guasto (§10) e la causa va nel log con il tag `PodLoader`.

## 10. Guasti e modalità sicura

Un **guasto** è un'eccezione che esce da un punto d'ingresso del Pod (`pods()`, `slots()`,
`Content`, un gestore di evento, un lavoro) e che l'host intercetta, oppure la morte del processo
mentre una schermata del Pod era aperta.

- Il guasto si conta per Pod (`pod_health`). Con `MAX_CRASHES` (3) guasti in `CRASH_WINDOW_HOURS`
  (24) il Pod passa a `disabled_by_crashes`; l'utente lo riaccende dalla schermata dei Pod.
- Prima di caricare un Pod esterno l'host scrive `loading=<id>`; a caricamento finito lo cancella.
  Se all'avvio successivo `loading` esiste, il processo è morto durante il caricamento: quell'avvio
  parte **senza Pod esterni** e il Pod indicato passa a `disabled_by_crashes`.
- Un Pod in errore non deve fermare gli altri: i provider si leggono uno per uno, come oggi
  (`PROVIDER_FAILED`). `BAD_ID` e `DUPLICATE_ID` restano come nella versione 1.

## 11. Cosa vede l'utente (requisiti per l'interfaccia)

- La schermata dei Pod elenca integrati e installati, con stato, versione e autore; accende,
  spegne, aggiorna e disinstalla.
- Il tasto **+** apre lo store: installazione **da file** (scelta di un documento di sistema),
  **da link** (con la tastiera a schermo del launcher) e gli indici con il loro elenco.
- Nel doppio schermo l'elenco sta sul display della griglia e il dettaglio della riga sotto il
  cursore (stato, impronta, permessi) sul display grande; a schermo singolo stanno affiancati.
  L1 e R1 cambiano scheda.
- Prima di installare: nome, versione, autore, impronta in gruppi, permessi con una riga ciascuno,
  dimensione, e l'avviso **"Un Pod gira con gli stessi permessi del launcher: installa solo Pod di
  cui ti fidi."** I testi stanno in `strings.xml` (italiano e inglese).

## 12. Fixture

`fixtures/pods/` contiene i casi letti dai test Rust (crate `zl-pods`, fase 3) e da quelli Kotlin
attraverso la FFI. Si **generano** con `python3 -I scripts/gen_pod_fixtures.py` e si controllano
con `--check` (rigenera e confronta byte per byte; se cambia la versione di Python o di zlib
basta rigenerare). Lo script richiede il pacchetto Python `cryptography`.
La chiave di firma è una **chiave di prova** (seme pubblico nello script): non firma nulla di reale.

| File | Contenuto |
| --- | --- |
| `keys.json` | Le due chiavi di prova (pubblica e impronta) |
| `manifests/*.json` | Un manifest per caso |
| `packages/*.zlpod` | Un pacchetto per caso, con firme vere |
| `indexes/*.json` | Un indice per caso |
| `policy.json` | Tabella installato / candidato → esito (§6) |
| `expected.json` | Per ogni caso, `ok` oppure l'elenco di codici attesi |

Ogni codice di questo documento ha almeno un caso che lo produce, **tranne** `PACKAGE_TOO_LARGE`,
`ENTRY_TOO_LARGE`, i codici di stato (§6) e `DUPLICATE_ID` tra un Pod esterno e uno integrato: una
fixture realistica per i limiti di dimensione peserebbe centinaia di MiB, quindi si provano con
test unitari che abbassano i limiti, e gli altri con test sul registro. Un caso che non produce il
codice atteso è un errore del lettore, non della fixture.

`expected.json` ha una voce per file: `ok` con `id`, `version` (e `signer` per i pacchetti), oppure
`issues` con i codici attesi. Per gli indici: `kept` (id e `flag`) e `issues` (posizione della voce
e codice). `installs` abbina un indice, un `id` e un pacchetto (`HASH_MISMATCH`, `SIGNER_MISMATCH`).
`policy.json` è la tabella della §6.
