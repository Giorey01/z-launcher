#!/usr/bin/env python3
"""Regenerates fixtures/hash/: small synthetic ROM files and expected.json with their digests.

The digests come from Python's own hashlib and zlib, so the Rust tests check zl-hash against an
independent implementation. Needs the `7z` command for the .7z case. Run from the repo root.
"""
import hashlib
import json
import os
import shutil
import struct
import subprocess
import tempfile
import zipfile
import zlib

OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "fixtures", "hash")


def pattern(n, step):
    """Deterministic bytes."""
    return bytes((i * step + 3) % 256 for i in range(n))


def digests(data):
    """CRC32, MD5 and SHA1 as lower-case hex."""
    return {
        "crc32": f"{zlib.crc32(data) & 0xFFFFFFFF:08x}",
        "md5": hashlib.md5(data).hexdigest(),
        "sha1": hashlib.sha1(data).hexdigest(),
    }


def entry(name, data, header=0):
    """Expected item for content with an optional header to leave out."""
    item = {"entry": name, "size": len(data), **digests(data), "headerless": None}
    if header:
        item["headerless"] = {"header_bytes": header, **digests(data[header:])}
    return item


def chd(version, label, logical):
    """A CHD header only (the layout the parser reads), with the SHA1 of `label` as raw SHA1."""
    sha1 = hashlib.sha1(label).digest()
    layouts = {3: (120, 28, 80), 4: (108, 28, 88), 5: (124, 32, 64)}
    length, logical_at, sha_at = layouts[version]
    head = bytearray(length)
    head[0:8] = b"MComprHD"
    head[8:12] = struct.pack(">I", length)
    head[12:16] = struct.pack(">I", version)
    head[logical_at : logical_at + 8] = struct.pack(">Q", logical)
    head[sha_at : sha_at + 20] = sha1
    md5 = None
    if version == 3:
        md5 = hashlib.md5(label).digest()
        head[44:60] = md5
    return bytes(head), {
        "entry": "",
        "size": logical,
        "crc32": None,
        "md5": md5.hex() if md5 else None,
        "sha1": sha1.hex(),
        "headerless": None,
    }


def main():
    """Writes the files and the expected digests."""
    shutil.rmtree(OUT, ignore_errors=True)
    os.makedirs(OUT)
    cases = []

    def put(name, data, container, items):
        with open(os.path.join(OUT, name), "wb") as f:
            f.write(data)
        cases.append({"file": name, "container": container, "entries": items})

    plain = pattern(200, 7)
    nes = b"NES\x1a" + bytes(12) + pattern(64, 5)
    smc = bytes(512) + pattern(1024, 3)
    sfc = pattern(1024, 3)
    lnx = b"LYNX" + bytes(60) + pattern(100, 11)
    a78 = b"\x01ATARI7800" + bytes(117) + pattern(100, 13)
    put("plain.bin", plain, "raw", [entry("", plain)])
    put("mario.nes", nes, "raw", [entry("", nes, 16)])
    put("game.smc", smc, "raw", [entry("", smc, 512)])
    put("game.sfc", sfc, "raw", [entry("", sfc)])
    put("game.lnx", lnx, "raw", [entry("", lnx, 64)])
    put("game.a78", a78, "raw", [entry("", a78, 128)])

    zip_path = os.path.join(OUT, "set.zip")
    with zipfile.ZipFile(zip_path, "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr("roms/", b"")
        z.writestr("roms/a.nes", nes)
        z.writestr("b.bin", plain)
    cases.append({"file": "set.zip", "container": "zip",
                  "entries": [entry("roms/a.nes", nes, 16), entry("b.bin", plain)]})

    work = tempfile.mkdtemp()
    os.makedirs(os.path.join(work, "roms"))
    with open(os.path.join(work, "roms", "a.nes"), "wb") as f:
        f.write(nes)
    with open(os.path.join(work, "b.bin"), "wb") as f:
        f.write(plain)
    subprocess.run(["7z", "a", "-bd", os.path.join(OUT, "set.7z"), "roms", "b.bin"],
                   cwd=work, check=True, stdout=subprocess.DEVNULL)
    for name, args in (("locked.zip", ["-tzip", "-psecret"]), ("locked.7z", ["-psecret", "-mhe=on"])):
        subprocess.run(["7z", "a", "-bd", *args, os.path.join(OUT, name), "b.bin"],
                       cwd=work, check=True, stdout=subprocess.DEVNULL)
        cases.append({"file": name, "error": "password"})
    shutil.rmtree(work)
    cases.append({"file": "set.7z", "container": "7z",
                  "entries": [entry("roms/a.nes", nes, 16), entry("b.bin", plain)]})

    for version in (3, 4, 5):
        data, item = chd(version, f"disc-v{version}".encode(), 700 * 1024 * 1024)
        put(f"disc-v{version}.chd", data, "chd", [item])

    with open(os.path.join(OUT, "not-a-chd.chd"), "wb") as f:
        f.write(b"this is not a CHD at all, just text")
    cases.append({"file": "not-a-chd.chd", "error": "damaged"})
    with zipfile.ZipFile(os.path.join(OUT, "empty.zip"), "w"):
        pass
    cases.append({"file": "empty.zip", "error": "empty"})

    with open(os.path.join(OUT, "expected.json"), "w") as f:
        json.dump({"cases": cases}, f, indent=2)
        f.write("\n")


if __name__ == "__main__":
    main()
