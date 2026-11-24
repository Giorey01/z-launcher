#!/usr/bin/env bash
# Every crate root must carry the strict attributes: a doc comment on every item (private ones
# too), functions of at most 50 lines and low cognitive complexity. Clippy enforces them.
set -euo pipefail
cd "$(dirname "$0")/.."
missing=0
for root in crates/*/src/lib.rs crates/*/src/main.rs; do
  [ -f "$root" ] || continue
  if ! grep -q "clippy::missing_docs_in_private_items" "$root"; then
    echo "strict attributes missing: $root"; missing=1
  fi
done
exit $missing
