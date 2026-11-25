#!/usr/bin/env python3
"""Packaging, signing, and verification tool for Z-Launcher Pods (.zlpod).

Implements the specification in `docs/spec/pods.md` (sections 3, 4, 5, 8).
Requires the `cryptography` package.

Usage:
    # Generate a new Ed25519 key pair:
    python3 scripts/package_pod.py --keygen --key-out my_pod.key

    # Package and sign a Pod directory:
    python3 scripts/package_pod.py --dir /path/to/pod_build --key my_pod.key --out my_pod.zlpod

    # Verify a .zlpod package:
    python3 scripts/package_pod.py --verify my_pod.zlpod

    # Output store index entry:
    python3 scripts/package_pod.py --index-entry my_pod.zlpod --url "https://example.com/pods/my_pod.zlpod"
"""
import argparse
import base64
import hashlib
import json
import os
import sys
import zipfile
from pathlib import Path

try:
    from cryptography.hazmat.primitives.asymmetric.ed25519 import (
        Ed25519PrivateKey,
        Ed25519PublicKey,
    )
    from cryptography.hazmat.primitives.serialization import (
        Encoding,
        NoEncryption,
        PrivateFormat,
        PublicFormat,
    )
except ImportError:
    sys.exit("Error: The 'cryptography' Python package is required. Install with: pip install cryptography")

ZIP_TIME = (2026, 1, 1, 0, 0, 0)
MAX_ENTRIES = 2000
MAX_PACKAGE_SIZE = 100 * 1024 * 1024  # 100 MiB


def b64(data: bytes) -> str:
    """Standard base64 as text."""
    return base64.b64encode(data).decode("ascii")


def unb64(text: str) -> bytes:
    """Decodes standard base64 text to bytes."""
    return base64.b64decode(text.encode("ascii"))


def load_private_key(key_path: Path) -> Ed25519PrivateKey:
    """Loads an Ed25519 private key from raw 32 bytes, hex, base64, or PKCS#8 PEM."""
    content = key_path.read_bytes()
    if len(content) == 32:
        return Ed25519PrivateKey.from_private_bytes(content)
    text = content.decode("utf-8", errors="ignore").strip()
    if text.startswith("-----BEGIN PRIVATE KEY-----"):
        from cryptography.hazmat.primitives.serialization import load_pem_private_key
        key = load_pem_private_key(content, password=None)
        if not isinstance(key, Ed25519PrivateKey):
            raise ValueError("Private key is not an Ed25519 key")
        return key
    try:
        raw = bytes.fromhex(text)
        if len(raw) == 32:
            return Ed25519PrivateKey.from_private_bytes(raw)
    except ValueError:
        pass
    try:
        raw = base64.b64decode(text)
        if len(raw) == 32:
            return Ed25519PrivateKey.from_private_bytes(raw)
    except Exception:
        pass
    raise ValueError(f"Could not load valid 32-byte Ed25519 private key from {key_path}")


def signed_message(files: dict[str, bytes]) -> bytes:
    """Builds the canonical signed message format for zlpod-v1."""
    lines = ["zlpod-v1"]
    for name in sorted(files.keys(), key=lambda n: n.encode("utf-8")):
        h = hashlib.sha256(files[name]).hexdigest()
        lines.append(f"{h}  {name}")
    return ("\n".join(lines) + "\n").encode("utf-8")


def keygen(out_path: Path) -> None:
    """Generates a new Ed25519 private key and writes it to out_path."""
    key = Ed25519PrivateKey.generate()
    raw = key.private_bytes(Encoding.Raw, PrivateFormat.Raw, NoEncryption())
    out_path.write_bytes(raw)
    pub = key.public_key().public_bytes(Encoding.Raw, PublicFormat.Raw)
    fp = hashlib.sha256(pub).hexdigest()
    print(f"Generated new Ed25519 private key at: {out_path}")
    print(f"Public key (base64): {b64(pub)}")
    print(f"Signer fingerprint (sha256): {fp}")
    print("Put this fingerprint in manifest.json -> author.signer")


def package_dir(src_dir: Path, key: Ed25519PrivateKey, out_file: Path) -> None:
    """Packages src_dir into a signed .zlpod file."""
    if not src_dir.is_dir():
        sys.exit(f"Error: Directory not found: {src_dir}")

    manifest_file = src_dir / "manifest.json"
    if not manifest_file.is_file():
        sys.exit(f"Error: manifest.json not found in {src_dir}")

    dex_file = src_dir / "classes.dex"
    if not dex_file.is_file():
        sys.exit(f"Error: classes.dex not found in {src_dir}")

    manifest_bytes = manifest_file.read_bytes()
    try:
        manifest = json.loads(manifest_bytes.decode("utf-8"))
    except Exception as e:
        sys.exit(f"Error parsing manifest.json: {e}")

    # Check signer matches the signing key
    pub = key.public_key().public_bytes(Encoding.Raw, PublicFormat.Raw)
    fp = hashlib.sha256(pub).hexdigest()
    manifest_signer = manifest.get("author", {}).get("signer", "")
    if manifest_signer != fp:
        print(f"Warning: manifest.json author.signer '{manifest_signer}' does not match key fingerprint '{fp}'!")
        manifest["author"]["signer"] = fp
        manifest_bytes = (json.dumps(manifest, indent=2, ensure_ascii=False) + "\n").encode("utf-8")

    files: dict[str, bytes] = {}
    for p in src_dir.rglob("*"):
        if p.is_file():
            rel = p.relative_to(src_dir).as_posix()
            if rel == "SIGNATURE":
                continue
            if rel == "manifest.json":
                files[rel] = manifest_bytes
            else:
                files[rel] = p.read_bytes()

    if len(files) > MAX_ENTRIES:
        sys.exit(f"Error: Package has {len(files)} entries, exceeding limit of {MAX_ENTRIES}")

    # Create SIGNATURE
    message = signed_message(files)
    signature = key.sign(message)
    sig_obj = {
        "algorithm": "ed25519",
        "publicKey": b64(pub),
        "signature": b64(signature),
    }
    sig_bytes = (json.dumps(sig_obj, indent=2, ensure_ascii=False) + "\n").encode("utf-8")

    # Create deterministic zip
    out_file.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(out_file, "w") as z:
        for name in sorted(files.keys(), key=lambda n: n.encode("utf-8")):
            info = zipfile.ZipInfo(filename=name, date_time=ZIP_TIME)
            info.external_attr = 0o644 << 16
            deflate = not (name.endswith(".png") or name.endswith(".jpg") or name.endswith(".so"))
            info.compress_type = zipfile.ZIP_DEFLATED if deflate else zipfile.ZIP_STORED
            z.writestr(info, files[name], compresslevel=9)

        # Append SIGNATURE as the last entry
        sig_info = zipfile.ZipInfo(filename="SIGNATURE", date_time=ZIP_TIME)
        sig_info.external_attr = 0o644 << 16
        sig_info.compress_type = zipfile.ZIP_STORED
        z.writestr(sig_info, sig_bytes)

    pkg_bytes = out_file.read_bytes()
    pkg_hash = hashlib.sha256(pkg_bytes).hexdigest()
    print(f"Successfully packaged and signed: {out_file}")
    print(f"Pod ID: {manifest.get('id')} v{manifest.get('version')}")
    print(f"SHA-256: {pkg_hash}")
    print(f"Size: {len(pkg_bytes)} bytes")
    print(f"Signer: {fp}")


def verify_package(package_path: Path) -> dict:
    """Verifies a .zlpod package and returns manifest data if valid."""
    if not package_path.is_file():
        sys.exit(f"Error: File not found: {package_path}")

    try:
        with zipfile.ZipFile(package_path, "r") as z:
            names = z.namelist()
            if "SIGNATURE" not in names:
                sys.exit("Verification failed: SIGNATURE entry missing")
            if "manifest.json" not in names:
                sys.exit("Verification failed: manifest.json missing")
            if "classes.dex" not in names:
                sys.exit("Verification failed: classes.dex missing")

            files: dict[str, bytes] = {}
            sig_bytes = None
            for info in z.infolist():
                if info.filename == "SIGNATURE":
                    sig_bytes = z.read(info)
                else:
                    files[info.filename] = z.read(info)

            sig_obj = json.loads(sig_bytes.decode("utf-8"))
            if sig_obj.get("algorithm") != "ed25519":
                sys.exit(f"Verification failed: unsupported algorithm '{sig_obj.get('algorithm')}'")

            pub_raw = unb64(sig_obj["publicKey"])
            sig_raw = unb64(sig_obj["signature"])
            pub_key = Ed25519PublicKey.from_public_bytes(pub_raw)

            # Verify signature
            msg = signed_message(files)
            pub_key.verify(sig_raw, msg)

            manifest = json.loads(files["manifest.json"].decode("utf-8"))
            fp = hashlib.sha256(pub_raw).hexdigest()
            manifest_signer = manifest.get("author", {}).get("signer")
            if manifest_signer != fp:
                sys.exit(f"Verification failed: signer fingerprint mismatch ({manifest_signer} != {fp})")

            print(f"Verification OK: {package_path}")
            print(f"ID: {manifest.get('id')}")
            print(f"Version: {manifest.get('version')}")
            print(f"Signer: {fp}")
            return manifest
    except Exception as e:
        sys.exit(f"Verification failed with error: {e}")


def print_index_entry(package_path: Path, url: str) -> None:
    """Prints a store index JSON item for this package."""
    manifest = verify_package(package_path)
    pkg_bytes = package_path.read_bytes()
    entry = {
        "id": manifest.get("id"),
        "name": manifest.get("name"),
        "version": manifest.get("version"),
        "url": url,
        "sha256": hashlib.sha256(pkg_bytes).hexdigest(),
        "signer": manifest.get("author", {}).get("signer"),
        "api": manifest.get("api", 1),
    }
    if "description" in manifest:
        entry["description"] = manifest["description"]
    if "homepage" in manifest:
        entry["homepage"] = manifest["homepage"]
    print("\n--- Store index entry ---")
    print(json.dumps(entry, indent=2, ensure_ascii=False))


def main() -> None:
    parser = argparse.ArgumentParser(description="Z-Launcher Pod packager and verifier")
    parser.add_argument("--keygen", action="store_true", help="Generate a new Ed25519 signing key")
    parser.add_argument("--key-out", type=Path, default=Path("pod.key"), help="Output path for generated key")
    parser.add_argument("--key", type=Path, help="Path to Ed25519 private key")
    parser.add_argument("--dir", type=Path, help="Directory containing Pod files to package")
    parser.add_argument("--out", type=Path, help="Output .zlpod file path")
    parser.add_argument("--verify", type=Path, help="Path to .zlpod to verify")
    parser.add_argument("--index-entry", type=Path, help="Generate index.json entry for .zlpod")
    parser.add_argument("--url", type=str, default="", help="Download URL for index entry")

    args = parser.parse_args()

    if args.keygen:
        keygen(args.key_out)
        return

    if args.verify:
        verify_package(args.verify)
        return

    if args.index_entry:
        print_index_entry(args.index_entry, args.url or f"https://example.com/pods/{args.index_entry.name}")
        return

    if args.dir:
        if not args.key:
            sys.exit("Error: --key is required when packaging a Pod")
        key = load_private_key(args.key)
        out = args.out
        if not out:
            manifest_file = args.dir / "manifest.json"
            if manifest_file.is_file():
                m = json.loads(manifest_file.read_text(encoding="utf-8"))
                out = Path(f"{m.get('id', 'pod')}-{m.get('version', '1.0.0')}.zlpod")
            else:
                out = Path("output.zlpod")
        package_dir(args.dir, key, out)
        return

    parser.print_help()


if __name__ == "__main__":
    main()
