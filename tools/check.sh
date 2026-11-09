#!/bin/sh
# Every check CI runs, in the same order. Run it before a commit.
set -eu
cd "$(dirname "$0")/.."

step() {
    printf '\n== %s\n' "$1"
}

step "Format"
cargo fmt --all -- --check

step "Lint"
cargo clippy --workspace --all-targets --locked -- -D warnings

step "Tests"
cargo test --workspace --locked

step "Compression that zlib reads, and zlib's that is read here"
python3 -I tools/check_flate.py

step "Documentation"
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked --quiet

step "No dashes"
python3 -I tools/check_text.py

step "Dependencies on the allow list"
python3 -I tools/check_deps.py

step "At most one line of test for two lines of code"
python3 -I tools/stats.py --check

printf '\nAll checks passed.\n'
