#!/usr/bin/env bash
# Rebuilds the RAR 5 archives of fixtures/archive/rar.json with RAR 7.23 (www.rarlab.com) and writes the
# expected contents with UnRAR (the official C++ reader, independent from the Rust one under test).
# Usage: bash fixtures/archive/rar/make.sh /path/to/rar-folder   (the folder with `rar` and `unrar`)
# The RAR 4 sets (rar4-*) are not rebuilt: RAR 7 no longer writes RAR 4; they come from HatchZip
# (MIT OR Apache-2.0, crates/cli/tests/fixtures/rar), see README.md.
set -euo pipefail
RAR="$1/rar"; UNRAR="$1/unrar"
here="$(cd "$(dirname "$0")" && pwd)"
work="$(mktemp -d)"; trap 'rm -rf -- "$work"' EXIT
python3 -I "$here/content.py" "$work/src"
cd "$work/src"
make() { local name="$1"; shift; rm -rf -- "${here:?}/$name"; mkdir -p "$here/$name"; "$RAR" a -idq -ma5 -m3 -tsm- -tsc- -tsa- "$@" "$here/$name/set.rar" game; }
make rar5-single
make rar5-volumes -v64k
make rar5-solid-volumes -s -v64k
make rar5-password -v64k -pSecret123
make rar5-headers -hpSecret123
python3 -I "$here/expected.py" "$UNRAR" "$here"
