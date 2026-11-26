"""Writes the deterministic tree the RAR fixtures are made of into the folder given."""
import os, random, sys
root = os.path.join(sys.argv[1], "game")
rng = random.Random(20261118)
os.makedirs(os.path.join(root, "data", "empty"), exist_ok=True)
with open(os.path.join(root, "readme.txt"), "w", encoding="utf-8") as f:
    f.write("Gioco di prova per i test RAR: àèìòù 日本語\n" * 40)
with open(os.path.join(root, "data", "disc.bin"), "wb") as f:
    f.write(bytes(rng.getrandbits(8) for _ in range(150_000)))
    f.write(b"\x00" * 60_000)
for i in range(5):
    with open(os.path.join(root, "data", f"track{i:02d}.raw"), "wb") as f:
        f.write(bytes(rng.getrandbits(8) for _ in range(rng.randint(4_000, 30_000))))
