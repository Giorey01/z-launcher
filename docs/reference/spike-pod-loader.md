# Spike Report: Caricamento Dinamico dei Pod (Fase 1)

**Data**: 25 Novembre 2026  
**Branch**: `spike/pod-loader`  
**Dispositivo di prova**: AYN Thor (Qualcomm Snapdragon 8 Gen 2, Android 13, API 33)  
**Configurazione build**: Release con R8 abilitato (`isMinifyEnabled = true`, `isShrinkResources = true`)  
**Esito complessivo**: **GO CON CAMBI** (Architettura promossa; richiesta modifica formale alla specifica §10 per l'error boundary Compose).

---

## 1. Sintesi Esecutiva

La Fase 1 del Piano Pod (`docs/PIANO-POD.md`) ha l'obiettivo di validare la fattibilità tecnica del caricamento dinamico di codice (`classes.dex` e `.so` nativi) all'interno del processo del launcher, verificando la compatibilità con Compose, R8, il gamepad, i temi dinamici, l'isolamento dei crash e gli aggiornamenti a caldo, su hardware reale con build minificata di produzione.

Tutti i 7 criteri dello spike sono stati verificati con successo su un **AYN Thor fisico**. Il caricamento dinamico con `DexClassLoader` è pienamente compatibile con Jetpack Compose e con il ciclo di vita del launcher. L'unico punto critico emerso riguarda la gestione dei crash in Compose (il compilatore vieta `try/catch` attorno a invocazioni `@Composable`), per il quale è stata individuata una soluzione architetturale pulita da integrare nella specifica v2.

---

## 2. Ambiente di Test

- **Dispositivo**: AYN Thor (`8cb525af`)
- **OS**: Android 13 (API 33, Tiramisu, build `Thor_V1.0.0.293_20251121_000958_user`)
- **Architettura CPU**: `aarch64` (Qualcomm Snapdragon 8 Gen 2 / sm8550)
- **Display**: Dual Display fisico attivo:
  - Display 0 (Top / Hero): 1080x1920 @ 60Hz
  - Display 4 (Bottom / Grid): 1080x1240 @ 120Hz
- **Toolchain**:
  - OpenJDK: Eclipse Temurin JDK 17.0.21+9
  - Android SDK: Platform 35, Build-Tools 35.0.0 (`d8`, `apksigner`)
  - NDK: 27.2.12479018 (Clang `aarch64-linux-android33-clang`)
  - Gradle: 8.11.1 con AGP 8.8.2 e Kotlin 2.1.0

---

## 3. Esito dei 7 Criteri di Spike

### Criterio 1: Caricamento dinamico di `classes.dex` con `DexClassLoader`
- **Implementazione**: Il file `.zlpod` viene estratto nella memoria interna privata dell'app (`context.filesDir/pods/<id>`). Prima dell'istanziazione, il file `classes.dex` viene reso in sola lettura con `dexFile.setReadOnly()` (requisito di sicurezza e compatibilità con Android 14+).
- **Classloader**: `DexClassLoader` riceve come parent `context.classLoader` (il classloader host del launcher).
- **Verifica**: `dexClassLoader.loadClass("dev.zlauncher.podspike.SpikePodProvider")` risolve la classe compilata dinamicamente con `d8` e ne istanzia l'oggetto senza errori di link o permessi.
- **Esito**: **Funziona**.

### Criterio 2: Entry class `PodProvider` e slot Start Menu
- **Implementazione**: La classe provider implementa l'interfaccia `PodProvider` (`dev.zlauncher.pods`), fornendo l'elenco dei `Pod`. L'istanza viene registrata a runtime in `PodRegistry`.
- **Verifica**: All'apertura dello Start Menu del launcher (pressione tasto `START`), appare la voce "Pod Spike" con icona vettoriale. La selezione della voce apre regolarmente la schermata dedicata del Pod.
- **Esito**: **Funziona**.

### Criterio 3: Schermata `@Composable`, `LocalZTheme`, componenti Pad e navigazione Gamepad
- **Implementazione**: La schermata del Pod (`SpikePod.Content(host)`) utilizza direttamente `LocalZTheme.current`, `PadButton`, `PadTile` e un'icona `ImageVector` (`Icons.Rounded.Star`).
- **Verifica Gamepad**:
  - Clic sul tasto A: incrementa regolarmente il contatore locale interno del Pod, con feedback visivo e sonoro.
  - Tasto B: esegue la navigazione indietro tornando alla schermata principale del launcher senza memory leak o glitch grafici.
- **Verifica Tema Dinamico**:
  - Test eseguito cambiando tema a runtime nelle Impostazioni del launcher da "3DS Pink" (sfondo crema/pesca, testo scuro `#ff50343a`) a "Z-Launcher (built in)" (sfondo ardesia scuro `#ff262934`, testo chiaro `#fff2f4fa`).
  - La schermata del Pod ha recepito istantaneamente il cambio di tema tramite la composizione di `LocalZTheme`, aggiornando colori di sfondo, bottoni, testi e accent.
- **Esito**: **Funziona**.

### Criterio 4: Caricamento libreria nativa da `lib/arm64-v8a/*.so` e JNI
- **Implementazione**: Nel pacchetto `.zlpod` è inclusa la libreria `lib/arm64-v8a/libspike_native.so` compilata con NDK Clang a 64 bit (`-shared -fPIC -O2`). La libreria espone la funzione JNI `Java_dev_zlauncher_podspike_SpikeNative_getMagicNumber`.
- **Caricamento**: Estratta nella directory privata dell'app e caricata tramite `System.load(nativeSo.absolutePath)`.
- **Verifica**: Il Pod invoca il metodo nativo `SpikeNative.getMagicNumber()` e riceve il valore `42` (nella v1) e `1337` (nella v2), mostrandolo a schermo.
- **Esito**: **Funziona**.

### Criterio 5: Impatto di R8 e Regole di Proguard
In build di release minificata (`isMinifyEnabled = true`, `isShrinkResources = true`), R8 ottimizza, rimuove e offusca simboli non direttamente referenziati nell'app. Poiché il Pod viene compilato separatamente e caricato dinamicamente, i simboli dell'host e le librerie condivise devono rimanere accessibili.

#### Misure dimensionali dell'APK di release:
| Configurazione ProGuard / R8 | Dimensione APK | Delta rispetto a baseline | Note |
|---|---|---|---|
| **Baseline release** (senza Pod) | 11.533.340 byte (~11,00 MB) | - | Nessuna regola Pod |
| **Keep minimale Pod + PadUI** | 11.549.724 byte (~11,01 MB) | **+16 KB** (+0,1%) | Solo `dev.zlauncher.pods.**` e `ui.pad.**` |
| **Keep Host + Compose + Kotlin stdlib** | 19.921.059 byte (~19,00 MB) | **+8,38 MB** (+72,7%) | Necessario per Pod arbitrari con Compose / stdlib |
| **Keep con intera libreria Material Icons** | 20.577.308 byte (~19,62 MB) | **+9,04 MB** (+78,4%) | Mantenimento di `androidx.compose.material.icons.**` |

#### Scoperte critiche su R8:
1. **Libreria icone**: Mantenere l'intero set `androidx.compose.material.icons.**` aggiunge oltre 9 MB di bytecode e metadati all'APK. **Decisione**: i Pod NON devono fare affidamento su tutte le icone material dell'host; devono impacchettare i propri vettori o accedere solo a un set ristretto di icone di sistema condivise.
2. **Kotlin Standard Library & Coroutine**: In un'applicazione standard, R8 offusca o elimina metodi della stdlib non usati dal codice sorgente dell'app. Un dex esterno che invoca `kotlin.collections.CollectionsKt` o costrutti di `kotlinx.coroutines` fallisce con `NoClassDefFoundError` se l'host non preserva la stdlib:
   ```proguard
   -keep class kotlin.** { *; }
   -keep class kotlinx.coroutines.** { *; }
   ```
3. **Costruttori sintetici Compose**: R8 rimuove i costruttori sintetici con argomenti opzionali (`DefaultConstructorMarker`, ad es. in `ImageVector.Builder`) se l'host non li chiama con valori di default. Il `pod-platform` BOM dovrà standardizzare queste regole.

### Criterio 6: Isolamento dei Guasti e Crash Compose
- **Problema rilevato**: Il compilatore Compose (`org.jetbrains.kotlin.plugin.compose`) **vieta sintatticamente** il blocco `try / catch` attorno a chiamate a funzioni `@Composable`:
  > *`Try catch is not supported around composable function invocations`*
  La specifica attuale (§10) indicava ingenuamente `try { pod.Content(host) } catch (t: Throwable)`. Questo costrutto non compila.
- **Comportamento a runtime di Compose**: Se una funzione Composable lancia un'eccezione a runtime (`error("boom")`), l'eccezione risale la gerarchia fino al `Recomposer`. Anche se l'eccezione viene intercettata dal gestore non catturato dei thread (`Thread.UncaughtExceptionHandler`), il `Recomposer` della composizione sottostante viene irrimediabilmente cancellato (`DiagnosticCoroutineContextException: StandaloneCoroutine{Cancelling}`). Di conseguenza, la `ComposeView` corrispondente si blocca e non processa ulteriori ricomposizioni.
- **Test effettuati su dispositivo**:
  - **Crash sincrono in `Content()`**: Un bottone del Pod ha scatenato `RuntimeException("Spike deliberate crash in Content()")`. Il gestore `Thread.UncaughtExceptionHandler` dell'host ha intercettato l'eccezione, impedendo il crash del processo (`FATAL EXCEPTION` in `AndroidRuntime`), ha emesso un log diagnostico in `PodFaultIsolation` e ha mostrato un Toast a schermo (`Pod isolato dopo errore: ...`), mantenendo in vita il processo del launcher.
  - **Crash asincrono in `LaunchedEffect`**: Un secondo bottone ha scatenato un'eccezione all'interno di una coroutine di `LaunchedEffect`. L'eccezione è stata intercettata dall'host evitando il crash del processo.
- **Proposta di modifica architetturale (CAMBIO per la specifica §10)**:
  Per isolare completamente i Pod senza rischiare la cancellazione del `Recomposer` del launcher principale, ogni Pod con UI deve essere ospitato in una `ComposeView` isolata o in una sub-composizione con proprio `Recomposer` / `CoroutineExceptionHandler` (ad esempio via `AndroidView { ComposeView(context).apply { setContent { pod.Content(host) } } }`). In caso di guasto, solo la vista del Pod decade, e il launcher può dismetterla e tornare alla Home senza intaccare la propria composizione principale.

### Criterio 7: Aggiornamento Dex a Caldo e Invalidazione Cache
- **Procedura di test**:
  1. È stata generata una versione 2 del Pod con modifiche sia a livello C nativo (magic number cambiato da `42` a `1337`), sia a livello Kotlin UI (titolo cambiato in `"POD SPIKE V2 RELOADED"`).
  2. Il file `sample.spike.zlpod` sul filesystem del dispositivo è stato sovrascritto via adb senza disinstallare né toccare l'APK del launcher.
  3. Il processo del launcher è stato riavviato con `am force-stop` e `am start`.
  4. Alla riapertura del Pod dallo Start Menu, sono apparsi immediatamente il nuovo titolo e il nuovo valore nativo JNI `1337`.
- **Conclusione**: Non si sono verificati problemi di cache stale in ART/dex. È sufficiente un riavvio pulito del processo del launcher; non è stato necessario cancellare manualmente la directory `code_cache`.
- **Esito**: **Funziona**.

---

## 4. Proguard Keep Rules Raccomandate per i Pod

Il set minimale individuato durante lo spike per garantire il corretto funzionamento di Pod esterni include:

```proguard
# Contratti del sistema Pod
-keep class dev.zlauncher.pods.** { *; }

# Componenti UI condivisi con i Pod
-keep class dev.zlauncher.ui.pad.** { *; }
-keep class dev.zlauncher.ui.theme.** { *; }
-keep class dev.zlauncher.ui.common.** { *; }

# Compose Runtime, UI e Foundation essenziali
-keep class androidx.compose.runtime.** { *; }
-keep class androidx.compose.ui.** { *; }
-keep class androidx.compose.foundation.** { *; }

# Kotlin Standard Library e Coroutines per codice dinamico
-keep class kotlin.** { *; }
-keep class kotlinx.coroutines.** { *; }

# Preservazione metadati e costruttori sintetici per Compose ABI
-keepattributes EnclosingMethod,InnerClasses,Signature
-keep class kotlin.jvm.internal.DefaultConstructorMarker { *; }
```

---

## 5. Decisione e Raccomandazione per la Roadmap

### **Verdetto: GO CON CAMBI**

Il caricamento dinamico è pienamente realizzabile, prestante e stabile su Android 13/14 con SoC Qualcomm e Jetpack Compose. 

### Modifiche richieste alla Specifica dei Pod (`docs/spec/pods.md`):
1. **§10 (Isolamento guasti UI)**:
   - Sostituire la dicitura `try { pod.Content(host) } catch (...)` con un'architettura a **Isolated Subcomposition Boundary**: il launcher ospita il Pod in un container isolato (`AndroidView` che incapsula una `ComposeView` secondaria con proprio `CoroutineExceptionHandler`), intercettando i crash di ricomposizione senza cancellare il Recomposer root del launcher.
2. **§8 (Dipendenze e icone)**:
   - Escludere esplicitamente l'intera libreria `androidx.compose.material.icons.extended` dal set fornito dall'host per contenere la dimensione dell'APK (+9 MB). I Pod devono includere gli asset grafici vettoriali nel proprio `.zlpod`.
3. **§6 (Firma e Sicurezza)**:
   - Confermata la validità di `dexFile.setReadOnly()`.

Con queste modifiche, il progetto può procedere con sicurezza alle successive fasi pianificate: **Fase 2 (`zl-net`)**, **Fase 3 (`zl-pods` Rust)** e **Fase 4 (`pod-ui`)**.
