"""Extracts every fixture set with UnRAR and writes fixtures/archive/rar.json (the cases both targets read)."""
import hashlib, json, os, subprocess, sys, tempfile
unrar, here = sys.argv[1], sys.argv[2]
PASSWORD = "Secret123"
CASES = [
    ("rar5-single", "set.rar", None, False),
    ("rar5-volumes", "set.part1.rar", None, False),
    ("rar5-solid-volumes", "set.part1.rar", None, True),
    ("rar5-password", "set.part1.rar", PASSWORD, False),
    ("rar5-headers", "set.rar", PASSWORD, False),
    ("rar4-volumes", "set.rar", None, False),
    ("rar4-solid", "set.rar", None, True),
]
cases = []
for folder, first, password, solid in CASES:
    out = tempfile.mkdtemp()
    cmd = [unrar, "x", "-idq", "-o+", f"-p{password}" if password else "-p-", os.path.join(here, folder, first), out + "/"]
    subprocess.run(cmd, check=True)
    files = {}
    for dirpath, _, names in os.walk(out):
        for n in names:
            p = os.path.join(dirpath, n)
            files[os.path.relpath(p, out)] = hashlib.sha256(open(p, "rb").read()).hexdigest()
    volumes = sorted(n for n in os.listdir(os.path.join(here, folder)))
    cases.append({"name": folder, "first": f"archive/rar/{folder}/{first}", "password": password,
                  "solid": solid, "volumes": len(volumes), "files": dict(sorted(files.items()))})
names = {
    "first": ["Game.rar", "game.part1.rar", "Game.Part01.RAR", "game.part001.rar", "game.v1.2.rar", "my.party.rar"],
    "later": ["game.part2.rar", "game.part02.rar", "Game.PART10.rar", "game.part0.rar", "game.r00", "game.r01", "game.zip", "game.rar.txt"],
    "splitCount": [
        {"names": ["g.part1.rar", "g.part2.rar", "g.part3.rar"], "parts": 3},
        {"names": ["g.part1.rar", "g.part1.rar", "G.PART2.RAR"], "parts": 2},
        {"names": ["g.rar", "g.r00", "g.r01"], "parts": 3},
        {"names": ["Game.rar"], "parts": 0},
        {"names": ["Game", "Game.zip"], "parts": 0},
    ],
}
doc = {"_doc": "RAR archives (fixtures/archive/rar/) and their expected contents (sha256 per extracted file, from the official UnRAR); "
       "names: which file names start a RAR set. Rebuilt by fixtures/archive/rar/make.sh.",
       "password": PASSWORD, "cases": cases, "names": names}
with open(os.path.join(here, "..", "rar.json"), "w", encoding="utf-8") as f:
    json.dump(doc, f, indent=2, ensure_ascii=False)
    f.write("\n")
