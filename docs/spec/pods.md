# Pod: il contratto per chi estende il launcher

Un **Pod** è una schermata che una build aggiunge al launcher. Si apre dal menu Start e usa tema,
suoni e libreria del launcher (RF-73). Questa repo definisce solo il contratto e non contiene
nessun Pod: chi li scrive sta fuori, e la dipendenza va sempre dal Pod al launcher, mai il
contrario.

## Le interfacce (`dev.zlauncher.pods`)

- `Pod`: `id` stabile (minuscole, cifre, `.`, `-`, `_`), `icon` e `title(context)` per la voce del
  menu Start, `Content(host)` per la schermata. La schermata è disegnata dentro il tema del
  launcher (`LocalZTheme`); B torna indietro, a meno che il Pod non lo gestisca da sé.
- `PodProvider`: dà i Pod di una build (`pods()`), in ordine. Deve avere un costruttore pubblico
  senza argomenti.
- `PodHost`: quello che il launcher offre al Pod:
  - `theme`, il tema attivo;
  - `play(action)`, i suoni dell'interfaccia;
  - `folders()`, `games(folder)`, `details(gameId)`, `playStats(gameId)`, la libreria in sola
    lettura;
  - `launch(game)`, il lancio con la stessa scelta di emulatore, lo stesso display e lo stesso
    conteggio del tempo;
  - `close()`, che chiude la schermata.

## Come si aggiunge un Pod

Si elenca la classe del provider in `META-INF/services/dev.zlauncher.pods.PodProvider` (una per
riga), dentro le sorgenti o le dipendenze della variante `full` composta altrove. Il launcher li
trova all'avvio con `ServiceLoader`, e R8 tiene i costruttori dei provider (`proguard-rules.pro`).
Il codice di `oss` non nomina mai un Pod.

## Regole di carico

- Un id vuoto o con altri caratteri: il Pod resta fuori (`BAD_ID`).
- Due Pod con lo stesso id: resta il primo (`DUPLICATE_ID`).
- Un provider che non si carica o che lancia un'eccezione in `pods()`: resta fuori, gli altri si
  caricano (`PROVIDER_FAILED`). Dopo 16 errori i provider non si leggono più.
- Senza provider (la build `oss`) non c'è nessun Pod e il menu Start non cambia.
