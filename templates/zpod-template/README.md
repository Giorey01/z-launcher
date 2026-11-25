# Z-Launcher Pod Template (`zpod-template`)

Modello di repository per creare un **Pod** per [Z-Launcher](https://github.com/Giorey01/z-launcher).

Un Pod è un'estensione autonoma distribuita in formato `.zlpod` (archivio zip firmato contenente `manifest.json`, `classes.dex`, risorse e librerie opzionali).

---

## Struttura del progetto

```
zpod-template/
├── .github/
│   └── workflows/
│       └── release.yml          # Compila, firma e pubblica la release .zlpod su GitHub
├── src/
│   └── main/
│       ├── kotlin/
│       │   └── dev/example/pod/
│       │       └── SamplePod.kt # Codice sorgente del Pod (contratto v2)
│       ├── resources/
│       │   └── META-INF/services/
│       │       └── dev.zlauncher.pods.PodProvider
│       ├── res/
│       │   ├── values/strings.xml
│       │   └── values-it/strings.xml
│       └── manifest.json        # Manifest v1 del Pod
├── build.gradle.kts
├── settings.gradle.kts
├── detekt.yml                   # Regole di stile (vietato androidx.compose.material*)
└── README.md
```

---

## 1. Configurazione iniziale

1. Clona o usa questo template come base per la tua repo:
   ```bash
   git clone <tuo-repo>
   ```

2. Genera una chiave di firma Ed25519 per il tuo Pod usando lo strumento `package_pod.py`:
   ```bash
   python3 scripts/package_pod.py --keygen --key-out pod-signing.key
   ```
   L'output mostrerà l'impronta (SHA-256 della chiave pubblica). Copia questa impronta nel campo `author.signer` di `src/main/manifest.json`.

3. Modifica `manifest.json`:
   - `id`: identificativo univoco (es. `com.mio.pod`).
   - `name`: nome visualizzato nel launcher.
   - `version`: versione semantica (es. `1.0.0`).
   - `author.name`: nome o alias dell'autore.
   - `author.signer`: l'impronta generata al punto 2.
   - `entry`: classe che implementa `PodProvider` (es. `dev.example.pod.SamplePodProvider`).
   - `slots`: slot dell'interfaccia in cui il Pod desidera mostrarsi (`start_menu`, `platform_grid_lead`, `settings_section`, etc.).
   - `permissions`: permessi richiesti (`library.read`, `library.rescan`, `launch`). Dichiara solo quelli necessari.

---

## 2. Linee guida per l'interfaccia (Gamepad-first)

I Pod girano su console portatili Android controllate da gamepad:
- **Nessun componente Material standard**: usa i componenti di `pod-ui` (`PadButton`, `PadTile`, `ZPanel`, `LocalZTheme`).
- **Focus gamepad obbligatorio**: ogni elemento interattivo deve essere raggiungibile con D-Pad / stick analogico ed evidenziato visivamente con il bordo o glow del tema.
- **Tasto B / Annulla**: deve sempre permettere di chiudere la schermata chiamando `host.close()`.
- **Tema del launcher**: rispetta i colori e i font forniti da `host.theme` o `LocalZTheme.current`. Non usare colori esadecimali cablati nel codice.

---

## 3. Compilazione e test in locale

In sviluppo locale, posiziona questa cartella accanto a `z-launcher`:
```
workspace/
├── z-launcher/
└── zpod-mio/
```
In questo modo `settings.gradle.kts` includerà direttamente i moduli `pod-api` e `pod-ui`.

Compila il file DEX:
```bash
./gradlew assembleRelease
```

---

## 4. Impacchettamento e firma del `.zlpod`

Usa lo script `package_pod.py`:
```bash
python3 ../z-launcher/scripts/package_pod.py \
    --dir build/outputs/pod-staging \
    --key pod-signing.key \
    --out build/com.mio.pod-1.0.0.zlpod
```

Verifica il pacchetto firmato:
```bash
python3 ../z-launcher/scripts/package_pod.py --verify build/com.mio.pod-1.0.0.zlpod
```

---

## 5. Rilascio e indice dello Store

1. Configura il segreto `POD_SIGNING_KEY` nelle impostazioni della repository GitHub (Secrets and variables -> Actions).
2. Crea un tag di versione:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```
3. Il workflow GitHub Actions compilerà, firmerà e pubblicherà l'archivio `.zlpod` nella pagina Release.
4. Per includere il Pod nello store pubblico di Z-Launcher, genera la voce per `index.json`:
   ```bash
   python3 ../z-launcher/scripts/package_pod.py --index-entry com.mio.pod-1.0.0.zlpod --url "https://github.com/.../download/v1.0.0/com.mio.pod-1.0.0.zlpod"
   ```
   e apri una Pull Request nella repository `zpod-index`.
