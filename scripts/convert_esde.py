#!/usr/bin/env python3
"""Genera data/platforms.json e data/emulators.json dalle configurazioni Android di ES-DE.

Fonte (licenza MIT, vedi NOTICE): gitlab.com/es-de/emulationstation-de, file
resources/systems/android/es_find_rules.xml e es_systems.xml.

Uso: convert_esde.py <es_find_rules.xml> <es_systems.xml> <cartella-data>

Il convertitore e' la sola fonte dei due JSON generati: non modificarli a mano, correggi qui.
Cio' che ES-DE non ha (Moonlight, xCloud) e' aggiunto in fondo con "verified": false.
"""
import json
import re
import sys
import xml.etree.ElementTree as ET
from collections import OrderedDict

SCHEMA = 1
VERSION = 1

# ES-DE system name -> id Cocoon (cartella). Nomi non presenti qui e non nella lista Cocoon
# restano col proprio nome e `cocoon: false`.
ALIASES = {
    "famicom": "nes", "sfc": "snes", "snesna": "snes", "sufami": "snes", "n64dd": "n64",
    "megadrive": "genesis", "megadrivejp": "genesis", "mark3": "mastersystem",
    "sega32xjp": "sega32x", "sega32xna": "sega32x", "megacd": "segacd", "megacdjp": "segacd",
    "saturnjp": "saturn", "pcengine": "tg16", "pcenginecd": "tg-cd", "neogeocdjp": "neogeocd",
    "naomi2": "naomi", "naomigd": "naomi", "arcade": "mame", "mame-advmame": "mame",
    "fba": "fbneo", "cps": "cps1", "amiga600": "amiga", "amiga1200": "amiga",
    "msx1": "msx", "msx2": "msx", "msxturbor": "msx", "pc": "dos", "sgb": "gb",
}
# Sistemi ES-DE che non sono giochi (app Android, shell) o hanno solo comandi segnaposto.
SKIP_SYSTEMS = {"android", "androidapps", "androidgames", "desktop", "emulators", "electron",
                "kodi", "lutris", "ags", "daphne", "mugen", "openbor", "flash", "windows3x",
                "windows9x", "xboxone", "ps4", "epic", "vpinball", "fpinball"}
# Pacchetti "mimetizzati" (build camuffate): darebbero falsi positivi nel rilevamento (es. PUBG).
DISGUISED_PREFIXES = ("com.antutu.", "com.tencent.ig", "com.miHoYo.", "com.ludashi.")
# Chiave del file tag per %INJECT%=%ROM% in base alla piattaforma (nomi Cocoon).
INJECT_ROM_TAG = {"steam": "steamappid", "windows": "localgameid", "scummvm": "scummvm_id"}
# %INJECT%=%BASENAME%.<ext>: chiave tag per estensione.
INJECT_EXT_TAG = {"ps3": "ps3_serial", "ps4": "ps4_id", "psvita": "vita_game_id"}
# File tag per piattaforma: estensione -> chiave (docs/reference/cocoon-wiki.md).
TAG_FILES = {
    "steam": {"steamappid": "steamappid"},
    "windows": {"localgameid": "localgameid"},
    "scummvm": {"dpt": "scummvm_id"},
    "psvita": {"dpt": "vita_game_id"},
    "moonlight": {"moonlight": "moonlight_id"},
    "xcloud": {"xcloud": "titleId"},
}
TAG_ONLY = {"steam", "windows", "moonlight", "xcloud"}
# Nomi estensioni sostitutivi (multi-parte) dove ES-DE e' troppo largo.
EXT_OVERRIDE = {"pico8": ["p8", "p8.png"]}
FLAGS = {"%ACTIVITY_CLEAR_TASK%": "clear_task", "%ACTIVITY_CLEAR_TOP%": "clear_top"}
# Piattaforme multi-disco (m3u).
MULTI_DISC = {"psx", "ps2", "segacd", "saturn", "dreamcast", "tg-cd", "neogeocd", "3do", "amigacd32",
              "atarijaguarcd", "pcfx", "cdimono1", "psp"}
# Ordine di preferenza per il default proposto: standalone per i sistemi pesanti.
# (emulatore, core opzionale). Le piattaforme non elencate: RetroArch col primo core, poi il resto.
PREFERENCES = {
    "ps2": [["nethersx2"], ["armsx2"], ["emucorex"], ["play!"]],
    "psp": [["ppsspp"], ["retroarch", "ppsspp"]],
    "psx": [["duckstation"], ["retroarch", "pcsx_rearmed"], ["epsxe"]],
    "psvita": [["vita3k"]],
    "n3ds": [["azahar"], ["lime3ds"], ["citra"], ["retroarch", "azahar"]],
    "nds": [["melonds"], ["drastic"], ["retroarch", "melondsds"]],
    "gc": [["dolphin"], ["dolphin-mmjr2"], ["dolphin-mmjr"], ["retroarch", "dolphin"]],
    "wii": [["dolphin"], ["dolphin-mmjr2"], ["dolphin-mmjr"], ["retroarch", "dolphin"]],
    "wiiu": [["cemu"]],
    "switch": [["eden"], ["eden-nightly"], ["kenji-nx"], ["skyline"]],
    "dreamcast": [["flycast"], ["redream"], ["retroarch", "flycast"]],
    "n64": [["m64plus-fz"], ["mupen64plus-ae"], ["retroarch", "mupen64plus_next_gles3"]],
    "saturn": [["yabasanshiro-2"], ["retroarch", "yabasanshiro"]],
    "steam": [["gamenative"], ["gamehub-lite"], ["winnative"]],
    "windows": [["gamenative"], ["winlator-cmod"], ["gamehub-lite"], ["winnative"]],
    "scummvm": [["scummvm"], ["retroarch", "scummvm"]],
    "ps3": [["aps3e"], ["armsx3"], ["emucorec"]],
}
# Nomi leggibili dove l'etichetta ES-DE non basta.
NAMES = {"retroarch": "RetroArch", "aps3e": "aPS3e", "armsx3": "ARMSX3",
         "emucorec": "EmuCoreC", "mame4droid-current": "MAME4droid Current", "dolphin-mmjr": "Dolphin MMJR", "dolphin-mmjr2": "Dolphin MMJR2"}

TOKEN = re.compile(r'(%[^%\s]+%)(?:=("[^"]*"|\S+))?')
SUBST = [
    ("%ROMRAW%", "{rom}"), ("%ROMSAF%", "{rom_saf}"), ("%ROMPROVIDER%", "{rom_provider}"),
    ("%ROM%", "{rom}"), ("%GAMEDIRRAW%", "{rom_dir}"), ("%ROMPATHRAW%", "{platform_dir}"),
    ("%BASENAME%", "{rom_name}"), ("%ANDROIDPACKAGE%", "{package}"),
    ("%EXTERNALDATA%", "/storage/emulated/0"), ("%INTERNALDATA%", "/data/data"),
]
ACCESS_BY_VAR = {"{rom}": "path", "{rom_uri}": "file_uri", "{rom_saf}": "saf",
                 "{rom_provider}": "content_uri"}


def cocoon_ids(wiki):
    """Legge gli id piattaforma dal documento Cocoon (righe di backtick dopo 'Id delle')."""
    text = open(wiki, encoding="utf-8").read()
    block = text.split("### Id delle piattaforme")[1].split("La wiki non")[0]
    return set(re.findall(r"`([a-z0-9-]+)`", block))


def parse_find_rules(path):
    """Emulatore -> lista di (package, activity completa) dal file find rules."""
    out = OrderedDict()
    for em in ET.parse(path).getroot().iter("emulator"):
        entries = []
        for e in em.iter("entry"):
            pkg, act = e.text.strip().split("/", 1)
            if any(pkg.startswith(p) for p in DISGUISED_PREFIXES):
                continue
            if act.startswith("."):
                act = pkg + act
            entries.append({"package": pkg, "activity": act, "verified": True})
        comment = ""
        out[em.get("name").lower()] = {"entries": entries}
    return out


def emu_names_from_comments(path):
    """Nome leggibile dall'ultimo commento prima di ogni emulatore (fallback)."""
    text = open(path, encoding="utf-8").read()
    names = {}
    for m in re.finditer(r'<emulator name="([^"]+)">\s*<!--\s*(.*?)\s*-->', text, re.S):
        c = m.group(2).split(", fork")[0]
        c = re.split(r" (?:emulator|client|engine|collection|game engine) ", c)[-1]
        names[m.group(1).lower()] = c.strip()
    return names


def subst(value, platform, tagkey_for_rom):
    """Sostituisce i segnaposto ES-DE con le nostre variabili {…}."""
    def inject(m):
        tail = m.group(1)
        if tail == "%ROM%":
            return "{tags.%s}" % tagkey_for_rom
        ext = tail.split(".", 1)[1]
        if ext not in INJECT_EXT_TAG:
            raise ValueError(ext)
        return "{tags.%s}" % INJECT_EXT_TAG[ext]
    value = re.sub(r"%INJECT%=(%ROM%|%BASENAME%\.[a-z0-9]+)", inject, value)
    for old, new in SUBST:
        value = value.replace(old, new)
    return value


def parse_command(text, platform):
    """Scompone un comando ES-DE in (emulatore, intent dict) o None se non supportato."""
    toks = TOKEN.findall(text)
    emu = None
    intent = OrderedDict()
    extras, flags, cats = [], [], []
    tagkey = INJECT_ROM_TAG.get(platform)
    for name, val in toks:
        val = val.strip('"') if val else val
        if name.startswith("%EMULATOR_"):
            emu = name[len("%EMULATOR_"):-1].lower()
        elif name in FLAGS:
            flags.append(FLAGS[name])
        elif name == "%ACTION%":
            intent["action"] = val
        elif name == "%CATEGORY%":
            cats.append(val)
        elif name == "%MIMETYPE%":
            intent["mime"] = val
        elif name == "%DATA%":
            intent["data"] = val
        elif name.startswith("%EXTRA"):
            extras.append((name, val))
        else:
            return None
    if emu is None:
        return None
    if "%INJECT%=%ROM%" in text and tagkey is None:
        return None
    for key in ("action", "data"):
        if key in intent:
            intent[key] = subst(intent[key], platform, tagkey)
    if cats:
        intent["category"] = cats
    out_extras = []
    for name, val in extras:
        m = re.match(r"%EXTRA(BOOL|INTEGER|ARRAY)?_(.+)%", name)
        kind, key = m.group(1), m.group(2)
        v = subst(val, platform, tagkey)
        if kind == "BOOL":
            out_extras.append({"key": key, "type": "bool", "value": v == "true"})
        elif kind == "INTEGER":
            out_extras.append({"key": key, "type": "int", "value": v})
        elif kind == "ARRAY":
            out_extras.append({"key": key, "type": "string_array", "value": v.split(",")})
        else:
            out_extras.append({"key": key, "type": "string", "value": v})
    intent["extras"] = out_extras
    intent["flags"] = flags
    return emu, intent


def rom_access(intent):
    """Forma di accesso al ROM dedotta dalle variabili usate dall'intent."""
    blob = json.dumps(intent)
    found = [a for var, a in ACCESS_BY_VAR.items() if var in blob]
    if "{tags." in blob and not found:
        return "none"
    return found[0] if found else "none"


def slug(label):
    """Id stabile da un'etichetta."""
    s = re.sub(r"\(standalone\)", "", label, flags=re.I)
    return re.sub(r"[^a-z0-9]+", "-", s.lower()).strip("-") or "default"


def retroarch_core(intent):
    """Se l'intent e' la forma RetroArch standard, ritorna il nome file del core, altrimenti None."""
    ex = {e["key"]: e["value"] for e in intent["extras"]}
    lib = ex.get("LIBRETRO", "")
    m = re.match(r"/data/data/\{package\}/cores/(.+\.so)$", lib)
    ok = (m and ex.get("ROM") == "{rom}" and
          ex.get("CONFIGFILE", "").endswith("/files/retroarch.cfg") and len(ex) == 3)
    return m.group(1) if ok else None


def main():
    find_rules, systems_xml, outdir = sys.argv[1:4]
    cocoon = cocoon_ids("docs/reference/cocoon-wiki.md")
    emus = parse_find_rules(find_rules)
    comment_names = emu_names_from_comments(find_rules)
    platforms = OrderedDict()
    profiles = {}  # emu -> OrderedDict(signature -> profile)
    cores = {}     # platform -> list of cores
    names_seen = {}
    unsupported = []
    for sysel in ET.parse(systems_xml).getroot().iter("system"):
        name = sysel.find("name").text
        if name in SKIP_SYSTEMS:
            continue
        pid = ALIASES.get(name, name)
        exts = sysel.find("extension").text.split()
        exts = sorted({e.lstrip(".").lower() for e in exts} - {""})
        plat = platforms.setdefault(pid, {
            "id": pid, "name": sysel.find("fullname").text, "extensions": [],
            "aliases": [], "multi_disc": pid in MULTI_DISC, "cocoon": pid in cocoon,
            "tag_files": []})
        plat["extensions"] = sorted(set(plat["extensions"]) | set(exts))
        if name != pid and name not in plat["aliases"]:
            plat["aliases"].append(name)
        for cmd in sysel.findall("command"):
            label = cmd.get("label") or ""
            try:
                res = parse_command(cmd.text, pid)
            except ValueError:
                res = None
            if res is None:
                if "PLACEHOLDER" not in cmd.text and "ANDROIDAPP" not in cmd.text:
                    unsupported.append((name, label))
                continue
            emu, intent = res
            if emu not in emus:
                unsupported.append((name, label + " [emulatore ignoto %s]" % emu))
                continue
            core = retroarch_core(intent) if emu == "retroarch" else None
            if emu == "retroarch":
                if core is None:
                    unsupported.append((name, label + " [retroarch non standard]"))
                    continue
                lst = cores.setdefault(pid, [])
                if all(c["file"] != core for c in lst):
                    cid = re.sub(r"_libretro(_android)?\.so$", "", core)
                    lst.append({"id": cid, "name": label, "file": core})
                continue
            if label.endswith("(Standalone)") and emu not in names_seen:
                names_seen[emu] = label[: -len("(Standalone)")].strip()
            sig = json.dumps(intent, sort_keys=True)
            table = profiles.setdefault(emu, OrderedDict())
            if sig in table:
                if pid not in table[sig]["platforms"]:
                    table[sig]["platforms"].append(pid)
                continue
            pl = {"id": slug(label or emu), "label": label or emu, "platforms": [pid],
                  "rom_access": rom_access(intent), "intent": intent,
                  "verified": True, "source": "es-de"}
            table[sig] = pl
    # emulators.json
    emulators = []
    for emu, info in emus.items():
        if emu == "retroarch":
            plist = [{"id": "core", "label": "RetroArch", "platforms": sorted(cores),
                      "rom_access": "path", "uses_core": True,
                      "intent": {"extras": [
                          {"key": "ROM", "type": "string", "value": "{rom}"},
                          {"key": "LIBRETRO", "type": "string", "value": "{core_path}"},
                          {"key": "CONFIGFILE", "type": "string", "value":
                           "/storage/emulated/0/Android/data/{package}/files/retroarch.cfg"}],
                          "flags": []},
                      "verified": True, "source": "es-de"}]
        else:
            plist = list(profiles.get(emu, {}).values())
        if not plist:
            continue
        seen = {}
        for p in plist:  # id univoci dentro l'emulatore
            n = seen.get(p["id"], 0)
            seen[p["id"]] = n + 1
            if n:
                p["id"] = "%s-%d" % (p["id"], n + 1)
        entries = list(info["entries"])
        if emu == "retroarch":
            entries.append({"package": "com.retroarch.plus",
                            "activity": "com.retroarch.browser.retroactivity.RetroActivityFuture",
                            "verified": False})
        e = {"id": emu,
             "name": NAMES.get(emu) or names_seen.get(emu) or comment_names.get(emu, emu),
             "kind": "retroarch" if emu == "retroarch" else "standalone",
             "packages": entries,
             "platforms": sorted({pl for p in plist for pl in p["platforms"]}),
             "profiles": plist}
        if emu == "retroarch":
            e["cores"] = {k: cores[k] for k in sorted(cores)}
        emulators.append(e)
    add_manual(emulators, platforms)
    apply_observed(emulators)
    add_grants(emulators)
    finalize_platforms(platforms)
    prefs = {}
    ids = {e["id"] for e in emulators}
    for plat, order in PREFERENCES.items():
        order = [o for o in order if o[0] in ids]
        if order:
            prefs[plat] = [{"emulator": o[0], **({"core": o[1]} if len(o) > 1 else {})} for o in order]
    emulators.sort(key=lambda e: e["id"])
    head = {"schema": SCHEMA, "version": VERSION,
            "sources": ["ES-DE (MIT) es_systems.xml / es_find_rules.xml for Android"]}
    write(outdir + "/platforms.json", {**head, "platforms": sorted(platforms.values(), key=lambda p: p["id"])})
    write(outdir + "/emulators.json", {**head, "default_preference": [{"emulator": "retroarch"}],
                                         "preferences": prefs, "emulators": emulators})
    for u in unsupported:
        print("non convertito:", u, file=sys.stderr)


OBSERVED_ON = "2026-11-24"


def find(emulators, emu_id):
    """Emulatore per id."""
    return next(e for e in emulators if e["id"] == emu_id)


def mark_observed(emulators, emu_id, profile_idx=0, note=""):
    """Segna un profilo come verificato su dispositivo (docs/reference/launch-observed.md)."""
    p = find(emulators, emu_id)["profiles"][profile_idx]
    p["verified"] = True
    p["verified_on"] = OBSERVED_ON
    p["source"] = "es-de+device"
    if note:
        p["note"] = note
    return p


def apply_observed(emulators):
    """Intent osservati su un AYN Thor (Android 13): prevalgono sui dati ES-DE se divergono."""
    ra = mark_observed(emulators, "retroarch", note="device: action VIEW; extras DATADIR, SDCARD, "
                       "EXTERNAL, APK, IME exist but their values are not known")
    ra["intent"]["action"] = "android.intent.action.VIEW"
    for eid in ("eden", "eden-nightly"):
        p = mark_observed(emulators, eid, note="device: ROM as SAF URI plus ClipData")
        p["rom_access"] = "saf"
        p["intent"]["data"] = "{rom_saf}"
        p["intent"]["flags"] = ["new_task", "no_animation"]
        p["intent"]["clip_data"] = {"uri": "{rom_saf}", "mime": "application/octet-stream"}
    p = mark_observed(emulators, "azahar", note="device: action VIEW")
    p["intent"]["action"] = "android.intent.action.VIEW"
    mark_observed(emulators, "ppsspp")
    mark_observed(emulators, "melonds")
    mark_observed(emulators, "nethersx2", note="device: AetherSX2 package xyz.aethersx2.android")
    d = find(emulators, "dolphin")
    d["packages"] = [{"package": "org.dolphinemu.dolphinemu", "verified": True, "activity":
                      "org.dolphinemu.dolphinemu.ui.main.MainActivity"}]
    p = mark_observed(emulators, "dolphin", note="device: MainActivity, MAIN, no LEANBACK category "
                      "(ES-DE has TvMainActivity)")
    p["intent"].pop("category", None)
    p = mark_observed(emulators, "cemu", note="device: VIEW; the game is the code/*.rpx file")
    p["intent"]["action"] = "android.intent.action.VIEW"


def add_grants(emulators):
    """I profili con URI SAF o provider concedono la lettura all'emulatore prima del lancio."""
    for e in emulators:
        for p in e["profiles"]:
            it = p["intent"]
            if p["rom_access"] in ("saf", "content_uri"):
                if "grant_read_uri_permission" not in it["flags"]:
                    it["flags"].append("grant_read_uri_permission")
            if p["rom_access"] == "saf":
                it["uri_grants"] = [
                    {"uri": "{rom_saf}", "read": True, "write": False, "optional": False},
                    {"uri": "{rom_saf_tree}", "read": True, "write": False, "optional": True}]
            elif p["rom_access"] == "content_uri":
                it["uri_grants"] = [
                    {"uri": "{rom_provider}", "read": True, "write": False, "optional": False}]


def add_manual(emulators, platforms):
    """Aggiunte senza fonte verificata: tutte con verified=false."""
    for pid, pname in (("moonlight", "Moonlight"), ("xcloud", "Xbox Cloud Gaming")):
        platforms.setdefault(pid, {"id": pid, "name": pname, "extensions": [], "aliases": [],
                                   "multi_disc": False, "cocoon": True, "tag_files": []})
    emulators.append({
        "id": "moonlight", "name": "Moonlight", "kind": "standalone",
        "packages": [{"package": "com.limelight", "verified": False},
                     {"package": "com.limelight.noir", "verified": False}],
        "platforms": ["moonlight"],
        "profiles": [{"id": "open-app", "label": "Moonlight (apre solo l'app)",
                      "platforms": ["moonlight"], "rom_access": "none",
                      "intent": {"action": "android.intent.action.MAIN", "extras": [], "flags": []},
                      "verified": False, "source": "manual"}]})


def finalize_platforms(platforms):
    """Applica estensioni speciali e file tag, ordina i campi."""
    for pid, p in platforms.items():
        if pid in EXT_OVERRIDE:
            p["extensions"] = EXT_OVERRIDE[pid]
        tags = TAG_FILES.get(pid, {})
        p["tag_files"] = [{"ext": e, "key": k} for e, k in sorted(tags.items())]
        if pid in TAG_ONLY:
            p["extensions"] = sorted(tags)
        else:
            p["extensions"] = sorted(set(p["extensions"]) | set(tags))
        p["aliases"].sort()


def write(path, obj):
    """Scrive JSON con indentazione stabile."""
    with open(path, "w", encoding="utf-8") as f:
        json.dump(obj, f, indent=1, ensure_ascii=False)
        f.write("\n")


if __name__ == "__main__":
    main()
