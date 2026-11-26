#!/usr/bin/env python3
"""Generates `fixtures/pods/` for `docs/spec/pods.md` (section 12).

Run `python3 -I scripts/gen_pod_fixtures.py` to write the fixtures and
`python3 -I scripts/gen_pod_fixtures.py --check` to verify the committed ones are exactly what
this script produces (same Python and zlib family; if the zip bytes ever drift, regenerate).

The signing keys are TEST keys derived from public seeds below: they sign nothing real.
Requires the `cryptography` package.
"""
import base64
import hashlib
import json
import sys
import zipfile
from pathlib import Path

from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
from cryptography.hazmat.primitives.serialization import Encoding, PublicFormat

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "fixtures" / "pods"
API_CURRENT = 2
API_MIN = 1
DEX = b"dex\n035\x00 test classes\n"
ZIP_TIME = (2026, 1, 1, 0, 0, 0)


class Key:
    """A test signing key with its public half and fingerprint."""

    def __init__(self, seed: bytes):
        """Derives the key pair from a 32-byte public test seed."""
        self.private = Ed25519PrivateKey.from_private_bytes(seed)
        self.public = self.private.public_key().public_bytes(Encoding.Raw, PublicFormat.Raw)
        self.fingerprint = hashlib.sha256(self.public).hexdigest()

    def sign(self, message: bytes) -> bytes:
        """Ed25519 signature (deterministic)."""
        return self.private.sign(message)


KEY_A = Key(bytes(range(32)))
KEY_B = Key(bytes(range(32, 64)))


def b64(data: bytes) -> str:
    """Standard base64 as text."""
    return base64.b64encode(data).decode()


def dumps(obj) -> bytes:
    """Stable JSON with a trailing newline."""
    return (json.dumps(obj, indent=2, ensure_ascii=False) + "\n").encode()


def base_manifest(**over):
    """A valid manifest; keyword overrides replace fields, a value of `...` removes the field."""
    m = {
        "schema": 1, "id": "sample.hello", "name": "Hello", "version": "1.2.0", "api": 1,
        "author": {"name": "Example", "signer": KEY_A.fingerprint},
        "entry": "dev.example.hello.HelloProvider", "slots": ["start_menu"], "permissions": [],
    }
    for k, v in over.items():
        if v is ...:
            m.pop(k, None)
        else:
            m[k] = v
    return m


def author(**over):
    """An `author` object with overrides (`...` removes)."""
    a = {"name": "Example", "signer": KEY_A.fingerprint}
    for k, v in over.items():
        if v is ...:
            a.pop(k, None)
        else:
            a[k] = v
    return a


def manifest_cases():
    """(file name, content, expected) for every manifest case."""
    ok = lambda m: {"ok": True, "id": m["id"], "version": m["version"]}
    minimal = base_manifest(id="sample.min", slots=[], permissions=[])
    full = base_manifest(
        description="A sample Pod.", slots=["start_menu", "platform_grid_lead"],
        permissions=["library.read", "library.rescan"], abis=["arm64-v8a"],
        homepage="https://example.com/hello")
    cases = [("valid-minimal", minimal, ok(minimal)), ("valid-full", full, ok(full))]
    bad = lambda name, content, code: cases.append((name, content, {"issues": [code]}))
    bad("invalid-json", b"{ not json", "MANIFEST_INVALID_JSON")
    bad("not-an-object", b"[]\n", "MANIFEST_INVALID_JSON")
    bad("missing-id", base_manifest(id=...), "MISSING_FIELD")
    bad("missing-slots", base_manifest(slots=...), "MISSING_FIELD")
    bad("missing-author-name", base_manifest(author=author(name=...)), "MISSING_FIELD")
    bad("schema-wrong-type", base_manifest(schema="1"), "WRONG_TYPE")
    bad("schema-2", base_manifest(schema=2), "UNSUPPORTED_SCHEMA")
    bad("description-wrong-type", base_manifest(description=5), "WRONG_TYPE")
    bad("api-wrong-type", base_manifest(api="1"), "WRONG_TYPE")
    for name, value in [("upper", "Sample.Hello"), ("empty", ""), ("space", "a b"), ("long", "a" * 65)]:
        bad("bad-id-" + name, base_manifest(id=value), "BAD_ID")
    bad("bad-name-empty", base_manifest(name=""), "BAD_NAME")
    bad("bad-name-long", base_manifest(name="n" * 41), "BAD_NAME")
    for name, value in [("two-parts", "1.2"), ("prerelease", "1.2.3-beta"), ("letters", "a.b.c")]:
        bad("bad-version-" + name, base_manifest(version=value), "BAD_VERSION")
    bad("api-too-new", base_manifest(api=API_CURRENT + 1), "API_TOO_NEW")
    bad("api-too-old", base_manifest(api=API_MIN - 1), "API_TOO_OLD")
    bad("bad-signer-short", base_manifest(author=author(signer="abc")), "BAD_SIGNER")
    bad("bad-signer-upper", base_manifest(author=author(signer=KEY_A.fingerprint.upper())), "BAD_SIGNER")
    bad("bad-entry-spaces", base_manifest(entry="not a class"), "BAD_ENTRY")
    bad("bad-entry-empty", base_manifest(entry=""), "BAD_ENTRY")
    bad("unknown-slot", base_manifest(slots=["start_menu", "nope"]), "UNKNOWN_SLOT")
    bad("duplicate-slot", base_manifest(slots=["start_menu", "start_menu"]), "DUPLICATE_VALUE")
    bad("unknown-permission", base_manifest(permissions=["root"]), "UNKNOWN_PERMISSION")
    bad("duplicate-permission", base_manifest(permissions=["http", "http"]), "DUPLICATE_VALUE")
    bad("unknown-abi", base_manifest(abis=["x86_64"]), "UNKNOWN_ABI")
    bad("bad-url", base_manifest(homepage="http://example.com"), "BAD_URL")
    return cases


def zip_bytes(entries):
    """A zip from (name, bytes, deflate) triples, with fixed times so the output is stable."""
    import io
    buf = io.BytesIO()
    with zipfile.ZipFile(buf, "w") as z:
        for name, data, deflate in entries:
            info = zipfile.ZipInfo(name, ZIP_TIME)
            info.external_attr = 0o644 << 16
            info.compress_type = zipfile.ZIP_DEFLATED if deflate else zipfile.ZIP_STORED
            z.writestr(info, data, compresslevel=9)
    return buf.getvalue()


def signed_message(files):
    """The signed text of spec section 4 for `files` (name -> bytes), SIGNATURE excluded."""
    lines = ["zlpod-v1"]
    for name in sorted(files, key=lambda n: n.encode()):
        lines.append(hashlib.sha256(files[name]).hexdigest() + "  " + name)
    return ("\n".join(lines) + "\n").encode()


def signature_file(files, key=KEY_A, claimed=None):
    """The SIGNATURE file: `key` signs `files`; `claimed` is the public key to declare."""
    claimed = claimed or key
    sig = key.sign(signed_message(files))
    return dumps({"algorithm": "ed25519", "publicKey": b64(claimed.public), "signature": b64(sig)})


def package(manifest=None, extra=None, key=KEY_A, drop=(), sign=True, tamper=None, after=None):
    """A `.zlpod`: manifest + dex + `extra` files, signed by `key`.

    `drop` removes files, `tamper` rewrites file contents after signing, `after` adds files after
    signing, `sign=False` leaves SIGNATURE out.
    """
    files = {"manifest.json": dumps(manifest or base_manifest()), "classes.dex": DEX}
    files.update(extra or {})
    signature = signature_file(files, key)
    for name in drop:
        files.pop(name, None)
    files.update(tamper or {})
    files.update(after or {})
    entries = [(n, d, False) for n, d in files.items()]
    if sign:
        entries.append(("SIGNATURE", signature, False))
    return zip_bytes(entries)


def package_cases():
    """(file name, bytes, expected) for every package case."""
    info = lambda m: {"ok": True, "id": m["id"], "version": m["version"], "signer": KEY_A.fingerprint}
    lib = base_manifest(abis=["arm64-v8a"])
    plain = base_manifest()
    full_extra = {"lib/arm64-v8a/libsample.so": b"\x7fELF test", "assets/data.txt": b"data\n",
                  "i18n/en.json": dumps({"title": "Hello"}), "i18n/it.json": dumps({"title": "Ciao"})}
    cases = [("valid", package(), info(plain)),
             ("valid-with-lib", package(lib, full_extra), info(lib)),
             ("valid-v1.3.0", package(base_manifest(version="1.3.0")), info(base_manifest(version="1.3.0"))),
             ("valid-signed-by-b", package(base_manifest(version="1.3.0", author=author(signer=KEY_B.fingerprint)),
                                           key=KEY_B),
                {"ok": True, "id": "sample.hello", "version": "1.3.0", "signer": KEY_B.fingerprint})]
    bad = lambda name, data, code: cases.append((name, data, {"issues": [code]}))
    bad("not-a-zip", b"this is not a zip file\n", "NOT_A_ZIP")
    bad("bad-path-traversal", package(extra={"../evil.txt": b"x"}), "BAD_PATH")
    bad("bad-path-absolute", package(extra={"/evil.txt": b"x"}), "BAD_PATH")
    bad("bad-path-backslash", package(extra={"assets\\evil.txt": b"x"}), "BAD_PATH")
    bad("unexpected-entry", package(extra={"extra/readme.txt": b"x"}), "UNEXPECTED_ENTRY")
    bad("duplicate-path-case", package(extra={"assets/A.txt": b"1", "assets/a.txt": b"2"}), "DUPLICATE_PATH")
    many = {"assets/f%04d" % i: b"" for i in range(2001)}
    bad("too-many-entries", package(extra=many), "TOO_MANY_ENTRIES")
    bomb = zip_bytes([("manifest.json", dumps(plain), False), ("classes.dex", DEX, False),
                      ("assets/zeros.bin", bytes(3 * 1024 * 1024), True)])
    bad("entry-ratio", bomb, "ENTRY_RATIO")
    bad("missing-manifest", package(drop=["manifest.json"]), "MISSING_FILE")
    bad("missing-dex", package(drop=["classes.dex"]), "MISSING_FILE")
    bad("missing-signature", package(sign=False), "SIGNATURE_MISSING")
    bad("signature-malformed", zip_bytes([("manifest.json", dumps(plain), False), ("classes.dex", DEX, False),
                                          ("SIGNATURE", b"not json", False)]), "SIGNATURE_MALFORMED")
    bad("signature-bad-algorithm", bad_algorithm_package(), "SIGNATURE_MALFORMED")
    bad("tampered-dex", package(tamper={"classes.dex": DEX + b"evil"}), "SIGNATURE_INVALID")
    bad("extra-file-after-signing", package(after={"assets/late.txt": b"x"}), "SIGNATURE_INVALID")
    bad("wrong-key-signature", wrong_key_package(), "SIGNATURE_INVALID")
    mismatch = base_manifest(author=author(signer=KEY_B.fingerprint))
    bad("signer-mismatch", package(mismatch), "SIGNER_MISMATCH")
    bad("api-too-new", package(base_manifest(api=API_CURRENT + 1)), "API_TOO_NEW")
    bad("lib-not-declared", package(lib, {"lib/x86_64/libx.so": b"\x7fELF"}), "LIB_NOT_DECLARED")
    return cases


def bad_algorithm_package():
    """A package whose SIGNATURE declares an algorithm other than ed25519."""
    files = {"manifest.json": dumps(base_manifest()), "classes.dex": DEX}
    sig = json.loads(signature_file(files))
    sig["algorithm"] = "rsa"
    entries = [(n, d, False) for n, d in files.items()] + [("SIGNATURE", dumps(sig), False)]
    return zip_bytes(entries)


def wrong_key_package():
    """A package signed by key B while SIGNATURE declares key A's public key."""
    files = {"manifest.json": dumps(base_manifest()), "classes.dex": DEX}
    entries = [(n, d, False) for n, d in files.items()]
    entries.append(("SIGNATURE", signature_file(files, KEY_B, claimed=KEY_A), False))
    return zip_bytes(entries)


def entry(pod_id, version, data, **over):
    """An index entry pointing at package bytes `data`."""
    e = {"id": pod_id, "name": pod_id.title(), "version": version, "api": 1,
         "description": "Sample entry.", "url": "https://example.invalid/pods/%s-%s.zlpod" % (pod_id, version),
         "sha256": hashlib.sha256(data).hexdigest(), "size": len(data), "signer": KEY_A.fingerprint}
    for k, v in over.items():
        if v is ...:
            e.pop(k, None)
        else:
            e[k] = v
    return e


def index(pods, **over):
    """An index document."""
    doc = {"schema": 1, "name": "Test index", "updated": "2026-11-25", "pods": pods}
    doc.update(over)
    return doc


def index_cases(packages):
    """(file name, content, expected) for every index case, plus the install cases."""
    valid = packages["valid"]
    lib = packages["valid-with-lib"]
    cases = []
    good = [entry("sample.hello", "1.2.0", valid), entry("sample.lib", "1.2.0", lib)]
    cases.append(("valid", index(good), {"kept": [{"id": "sample.hello", "flag": None},
                                                   {"id": "sample.lib", "flag": None}], "issues": []}))
    cases.append(("invalid-json", b"{ nope", {"issues": [{"code": "INDEX_INVALID_JSON"}]}))
    cases.append(("unsupported-schema", index(good, schema=2), {"issues": [{"code": "UNSUPPORTED_SCHEMA"}]}))
    mixed = [entry("good.one", "1.0.0", valid),
             entry("http.url", "1.0.0", valid, url="http://example.invalid/x.zlpod"),
             entry("bad.hash", "1.0.0", valid, sha256="xyz"),
             entry("Bad.Id", "1.0.0", valid),
             entry("bad.version", "1.0", valid),
             entry("missing.url", "1.0.0", valid, url=...),
             entry("good.one", "2.0.0", valid),
             entry("future.pod", "1.0.0", valid, api=API_CURRENT + 1),
             entry("old.pod", "1.0.0", valid, api=API_MIN - 1)]
    cases.append(("mixed", index(mixed), {
        "kept": [{"id": "good.one", "flag": None}, {"id": "future.pod", "flag": "API_TOO_NEW"},
                 {"id": "old.pod", "flag": "API_TOO_OLD"}],
        "issues": [{"entry": 1, "code": "BAD_URL"}, {"entry": 2, "code": "BAD_HASH"},
                   {"entry": 3, "code": "BAD_ID"}, {"entry": 4, "code": "BAD_VERSION"},
                   {"entry": 5, "code": "MISSING_FIELD"}, {"entry": 6, "code": "DUPLICATE_ID"}]}))
    wrong_hash = [entry("sample.hello", "1.2.0", valid, sha256="0" * 64)]
    cases.append(("install-hash-mismatch", index(wrong_hash), {"issues": []}))
    other = [entry("sample.hello", "1.2.0", valid, signer=KEY_B.fingerprint)]
    cases.append(("install-signer-mismatch", index(other), {"issues": []}))
    return cases


INSTALLS = [
    {"index": "valid.json", "id": "sample.hello", "package": "valid.zlpod", "expected": {"ok": True}},
    {"index": "install-hash-mismatch.json", "id": "sample.hello", "package": "valid.zlpod",
     "expected": {"issues": ["HASH_MISMATCH"]}},
    {"index": "install-signer-mismatch.json", "id": "sample.hello", "package": "valid.zlpod",
     "expected": {"issues": ["SIGNER_MISMATCH"]}},
]


def policy():
    """The install-outcome table of spec section 6."""
    a, b = KEY_A.fingerprint, KEY_B.fingerprint
    rows = [(None, "1.0.0", a, "NEW"), ("1.0.0", "1.0.1", a, "UPDATE"), ("1.9.0", "1.10.0", a, "UPDATE"),
            ("1.0.0", "2.0.0", a, "UPDATE"), ("1.2.0", "1.2.0", a, "SAME_VERSION"),
            ("1.2.0", "1.1.9", a, "DOWNGRADE"), ("1.10.0", "1.9.0", a, "DOWNGRADE"),
            ("1.0.0", "1.0.1", b, "SIGNER_CHANGED"), ("1.2.0", "1.2.0", b, "SIGNER_CHANGED"),
            ("1.2.0", "1.1.0", b, "SIGNER_CHANGED")]
    return [{"installed": None if v is None else {"version": v, "signer": a},
             "candidate": {"version": c, "signer": s}, "expected": e} for v, c, s, e in rows]


def build():
    """Every fixture file as {relative path: bytes}."""
    files = {"keys.json": dumps({"note": "TEST keys derived from public seeds; they sign nothing real.",
                                 "a": {"publicKey": b64(KEY_A.public), "fingerprint": KEY_A.fingerprint},
                                 "b": {"publicKey": b64(KEY_B.public), "fingerprint": KEY_B.fingerprint}})}
    expected = {"constants": {"api_current": API_CURRENT, "api_min": API_MIN},
                "manifests": {}, "packages": {}, "indexes": {}, "installs": INSTALLS}
    for name, content, exp in manifest_cases():
        files["manifests/%s.json" % name] = content if isinstance(content, bytes) else dumps(content)
        expected["manifests"][name + ".json"] = exp
    packages = {}
    for name, data, exp in package_cases():
        files["packages/%s.zlpod" % name] = data
        packages[name] = data
        expected["packages"][name + ".zlpod"] = exp
    for name, content, exp in index_cases(packages):
        files["indexes/%s.json" % name] = content if isinstance(content, bytes) else dumps(content)
        expected["indexes"][name + ".json"] = exp
    files["policy.json"] = dumps(policy())
    files["expected.json"] = dumps(expected)
    return files


def write(files, base: Path):
    """Writes `files` under `base`, replacing what was there."""
    for rel, data in files.items():
        path = base / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)


def check(files) -> int:
    """Compares the committed fixtures with `files`; returns a process exit code."""
    problems = []
    for rel, data in files.items():
        path = OUT / rel
        if not path.exists() or path.read_bytes() != data:
            problems.append("differs: " + rel)
    committed = {str(p.relative_to(OUT)) for p in OUT.rglob("*") if p.is_file()}
    problems += ["unexpected file: " + r for r in sorted(committed - set(files))]
    for p in problems:
        print(p)
    return 1 if problems else 0


def main():
    """Entry point."""
    files = build()
    if "--check" in sys.argv:
        sys.exit(check(files))
    if OUT.exists():
        for p in sorted(OUT.rglob("*"), reverse=True):
            p.unlink() if p.is_file() else p.rmdir()
    write(files, OUT)
    print("wrote %d files to %s" % (len(files), OUT))


if __name__ == "__main__":
    main()
