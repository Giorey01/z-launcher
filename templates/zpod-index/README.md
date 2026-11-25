# Z-Launcher Pod Index (`zpod-index`)

Indice pubblico ufficiale dei Pod per [Z-Launcher](https://github.com/Giorey01/z-launcher).

Il launcher interroga `index.json` in questo repository per mostrare l'elenco dei Pod disponibili nello Store integrato (Schermata Pod -> Store).

---

## Struttura dell'indice (`index.json`)

```json
{
  "schema": 1,
  "pods": [
    {
      "id": "com.example.sample",
      "name": "Sample Pod",
      "version": "1.0.0",
      "url": "https://github.com/example/zpod-sample/releases/download/v1.0.0/sample-1.0.0.zlpod",
      "sha256": "33e5c8636ba42397f2302225888e3cb655bfd1c964ef1640e7c75eb0793c30e8",
      "signer": "7527f3a211047bdf711430f6fbc0d12e7857751ea1f2d394d089f50485d013cd",
      "api": 1,
      "description": "Descrizione sintetica del Pod.",
      "homepage": "https://github.com/example/zpod-sample"
    }
  ]
}
```

---

## Come sottomettere un Pod allo Store

1. Assicurati che il tuo Pod sia compilato dal modello `zpod-template` e rispetti le linee guida:
   - Contratto Pod v2 (`dev.zlauncher.pods.Pod`).
   - Navigazione gamepad-first con `pod-ui`.
   - Nessun crash all'avvio o perdita di memoria.
   - Nessun accesso di rete non dichiarato o malevolo.
2. Crea una Release GitHub pubblica nel repository del tuo Pod allegando il file `.zlpod` firmato.
3. Genera la voce JSON per l'indice usando `package_pod.py`:
   ```bash
   python3 scripts/package_pod.py --index-entry <tuo-file>.zlpod --url "https://github.com/.../download/.../<tuo-file>.zlpod"
   ```
4. Apri una Pull Request aggiungendo il blocco all'array `pods` in `index.json`.
