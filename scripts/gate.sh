#!/usr/bin/env bash
# The gate every change must pass before a commit: format, lints, strict attributes, tests.
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p target
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
bash scripts/strict-check.sh
cargo test --workspace --all-features 2>&1 | tee target/last-test.log
grep -E "^test result" target/last-test.log | awk '{p+=$4;f+=$6} END{print "TOTALE passed",p,"failed",f; exit (f>0)}'
