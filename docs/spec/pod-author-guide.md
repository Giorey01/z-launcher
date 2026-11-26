# Guida per gli Autori di Pod (Pod Author Guide)

Guida ufficiale allo sviluppo, firma e distribuzione di **Pod** per **Z-Launcher**.

---

## 1. Cos'è un Pod

Un **Pod** è un componente modulare del front-end di Z-Launcher distribuito in un singolo file compresso `.zlpod`.
I Pod estendono il launcher aggiungendo sezioni al menu Start, schede di impostazioni, schermate informative, leaderboard o elementi in testa alle griglie delle piattaforme.

I Pod esterni vengono caricati dinamicamente da Z-Launcher a runtime tramite `DexClassLoader`, girando all'interno del processo del launcher. Condividono lo stesso ciclo di vita, tema grafico e gestione degli input.

---

## 2. Il Contratto Pod v2 (`pod-api`)

Ogni Pod implementa l'interfaccia `dev.zlauncher.pods.Pod`:

```kotlin
package dev.zlauncher.pods

import android.content.Context
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.vector.ImageVector

interface Pod {
    /** Identificativo univoco del Pod (es. 'com.autore.mio-pod'). Corrisponde a manifest.json. */
    val id: String

    /** Icona vettoriale mostrata nei menu e negli slot. */
    val icon: ImageVector

    /** Titolo localizzato del Pod. */
    fun title(context: Context): String

    /** Slot in cui il Pod richiede di agganciarsi nell'interfaccia del launcher. */
    fun slots(): List<PodSlot>

    /** Contenuto Composable principale per una specifica route interna. */
    @Composable
    fun Content(host: PodHost, route: PodRoute)
}
```

### Il Provider (`PodProvider`)
La classe indicata come `entry` in `manifest.json` implementa `PodProvider`:
```kotlin
interface PodProvider {
    /** Restituisce i Pod forniti da questo modulo. */
    fun pods(): List<Pod>
}
```

### Slot Disponibili (`PodSlot`)
I Pod possono comparire in diversi punti dell'interfaccia senza alterare la struttura del launcher:
- `PodSlot.StartMenu(title, icon, route)`: aggiunge una voce nel menu Start principale.
- `PodSlot.PlatformGridLead(platformId, route)`: compare come prima scheda nella griglia dei giochi di una console.
- `PodSlot.HomeLead(route)`: compare prima delle cartelle o schede della schermata Home.
- `PodSlot.SettingsSection(title, route)`: aggiunge una riga nella schermata delle Impostazioni.

### Servizi dell'Host (`PodHost`)
Tramite l'oggetto `host` passato a `Content`, il Pod ha accesso a:
- `host.theme`: colori, forme e tipografia del tema attivo (`ZTheme`).
- `host.play(SoundAction)`: riproduzione dei suoni di sistema sintetizzati (movimento cursore, selezione, apertura/chiusura).
- `host.storage`: archiviazione isolata di impostazioni (`getString`, `setInt`, `setBoolean`) e cartelle private su disco (`filesDir`, `cacheDir`).
- `host.close()`: chiusura immediata della schermata del Pod con ritorno alla vista precedente.
- `host.navigate(route)`: cambio rotta all'interno del Pod.
- `host.folders()`, `host.games(platform)`, `host.details(id)`, `host.playStats(id)`: consultazione della libreria (richiede permesso `library.read`).
- `host.launch(game)`: lancio configurato di un gioco (richiede permesso `launch`).

---

## 3. Linee Guida per l'Interfaccia: Gamepad-First

Z-Launcher è progettato per console portatili da gioco. Qualsiasi interfaccia utente di un Pod **deve** poter essere navigata fluidamente con D-Pad, levette analogiche e tasti fisici.

### Regole obbligatorie
1. **Navigazione da D-Pad**: ogni elemento cliccabile o interattivo deve essere selezionabile con le frecce direzionali.
2. **Focus visibile e reattivo**: l'elemento selezionato deve evidenziare chiaramente la propria selezione tramite bordi accentati, ombreggiature o gradienti ricavati da `LocalZTheme.current`.
3. **Pulsante Annulla/B**: la pressione del tasto B / Back deve sempre permettere all'utente di uscire dallo schermo invocando `host.close()`.
4. **Nessun pulsante quadrato touch standard**: non usare componenti Material 3 standard (`Button`, `Card`) che assumono l'uso delle dita. Usa i componenti di `pod-ui`:
   - `PadButton`: pulsante con suono nativo, focus ring automatico e supporto pressione tasto A.
   - `PadTile`: riquadro/tessera quadrata per elementi di griglia o icone rapide.
   - `ZPanel`: card di sfondo sopraelevata con gradiente coerente al tema.
5. **Colori del tema**: non inserire colori hex cablati. Leggi sempre da `host.theme` o `LocalZTheme.current` (`backgroundBrush()`, `accentStart`, `textPrimary`, `textSecondary`).

---

## 4. Modello di Sicurezza e Permessi

Un Pod gira con gli stessi permessi Android di Z-Launcher. Prima dell'installazione, l'utente vede l'impronta crittografica dell'autore, l'elenco dei permessi richiesti e l'avviso di sicurezza:

> *"Un Pod gira con gli stessi permessi del launcher: installa solo Pod di cui ti fidi."*

### Permessi disponibili
| Permesso | Descrizione |
| --- | --- |
| `library.read` | Lettura cartelle, piattaforme, elenco giochi, dettagli e statistiche di gioco. |
| `library.rescan` | Richiesta di riesecuzione della scansione filesystem per una console. |
| `launch` | Avvio di emulatori o app configurate per i giochi. |

Se un Pod tenta di invocare un'operazione dell'host senza aver dichiarato il relativo permesso in `manifest.json`, viene sollevata una `PermissionNotDeclaredException`.

### Firma Crittografica (Ed25519)
- Ogni pacchetto `.zlpod` è firmato digitalmente con Ed25519.
- `manifest.json` contiene in `author.signer` l'impronta (hash SHA-256 esadecimale della chiave pubblica).
- Gli aggiornamenti futuri del Pod **devono** essere firmati con la medesima chiave. Se la chiave cambia, Z-Launcher rifiuta l'aggiornamento con errore `SIGNER_CHANGED`.

### Isolamento da Crash e Safe Mode
Se un Pod va in crash più di 3 volte in 24 ore, Z-Launcher lo disattiva automaticamente marcandolo come `disabled_by_crashes` per impedire blocchi all'avvio del dispositivo. L'utente può riabilitarlo manualmente dalle Impostazioni dei Pod.

### Librerie native (`lib/arm64-v8a/`)
Il launcher passa la cartella `lib/<abi>/` del pacchetto al `DexClassLoader` del Pod. Una libreria
caricata con `System.loadLibrary` la trova da sola; i binding UniFFI invece caricano con JNA, che non
guarda il classloader. Prima della prima chiamata FFI, chiedi il percorso al classloader del Pod e
passalo ai binding:

```kotlin
val path = (MyPod::class.java.classLoader as? dalvik.system.BaseDexClassLoader)?.findLibrary("mio_core")
if (path != null) System.setProperty("uniffi.component.mio_core.libraryOverride", path)
```

JNA non va messa nel Pod: la fornisce il launcher (dichiarala `compileOnly`, alla versione del suo catalogo).

---

## 5. Creazione di un Pod con `zpod-template`

1. Clona la repository `zpod-template`:
   ```bash
   git clone https://github.com/Giorey01/zpod-template.git zpod-mio
   cd zpod-mio
   ```

2. Genera la tua coppia di chiavi privata/pubblica:
   ```bash
   python3 ../z-launcher/scripts/package_pod.py --keygen --key-out my_pod.key
   ```
   Annota l'impronta generata (`signer`) e inseriscila in `src/main/manifest.json`.

3. Implementa il tuo Pod estendendo `Pod` e registrandolo nel provider.

4. Verifica con Detekt:
   ```bash
   ./gradlew detekt
   ```

5. Compila il DEX e prepara i file:
   ```bash
   ./gradlew stagePod
   ```

6. Impacchetta e firma il file `.zlpod`:
   ```bash
   python3 ../z-launcher/scripts/package_pod.py \
       --dir build/outputs/pod-staging \
       --key my_pod.key \
       --out build/com.mio.pod-1.0.0.zlpod
   ```

7. Verifica l'integrità del pacchetto generato:
   ```bash
   python3 ../z-launcher/scripts/package_pod.py --verify build/com.mio.pod-1.0.0.zlpod
   ```

---

## 6. Distribuzione

### Da file (sviluppo e distribuzione privata)
Copia il `.zlpod` sul dispositivo (per esempio `adb push mio.zlpod /sdcard/Download/`), poi in
Z-Launcher: **Pod → Store → Installa da file**. Il pacchetto passa gli stessi controlli di uno
scaricato e chiede la stessa conferma. L'opzione "Pod esterni" deve essere accesa.

### Tramite Deep Link
Z-Launcher intercetta i link nel formato:
```
zlauncher://pod?url=https://tuosito.com/percorso/com.mio.pod-1.0.0.zlpod
```
All'apertura del link su dispositivo Android, Z-Launcher scaricherà il pacchetto, ne verificherà la firma e presenterà all'utente la schermata di conferma d'installazione con impronta e permessi.

### Tramite lo Store Pubblico (`zpod-index`)
Per rendere il Pod disponibile nel catalogo Store all'interno di Z-Launcher:
1. Carica il `.zlpod` come Release asset su GitHub.
2. Genera il blocco JSON dell'indice:
   ```bash
   python3 ../z-launcher/scripts/package_pod.py \
       --index-entry build/com.mio.pod-1.0.0.zlpod \
       --url "https://github.com/.../releases/download/v1.0.0/com.mio.pod-1.0.0.zlpod"
   ```
3. Invia una Pull Request aggiungendo il blocco all'array `pods` in `index.json` nella repository `zpod-index`.
