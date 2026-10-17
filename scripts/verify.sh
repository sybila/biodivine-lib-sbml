#!/usr/bin/env bash
#
# Runs every verification gate of this repository and prints each command with its exit code.
#
#     scripts/verify.sh          # from the repository root, or anywhere
#     make verify                # same thing
#
# Scope notes:
#   * The tests, the lint gates and the documentation gate run for the crate itself and for the
#     helper crates in this repository (`sbml-macros`, `sbml-test-suite`).
#   * The SBML syntactic test suite is not part of `cargo test`; it needs an unpacked `./syntactic`
#     folder and is run separately (see `VALIDATION.md`). `make coverage` regenerates the
#     specification coverage matrix and needs the specification checkout.
#   * A gate that cannot run in the current environment (a toolchain the repository declares but
#     that is not installed, or a missing test-suite folder) is reported as skipped, not as a
#     failure, so that the result stays honest about what actually ran.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
[ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"

PYTHON="${PYTHON:-python3}"
SPEC="${SPEC:-$ROOT/../../sbml-specifications}"

failures=0
skipped=0
run() {
    printf '\n$ %s\n' "$*"
    "$@"
    local status=$?
    printf -- '-> exit %s\n' "$status"
    if [ "$status" -ne 0 ]; then
        failures=$((failures + 1))
        printf '!!! FAILED: %s\n' "$*"
    fi
    return 0
}

skip() {
    printf '\n--- SKIPPED: %s\n' "$*"
    skipped=$((skipped + 1))
}

printf '=== environment\n'
rustc --version
cargo --version
"$PYTHON" -V

# --- Rust: format, lint, test -----------------------------------------------------------------
run cargo fmt --all -- --check
run cargo clippy --all-targets --all-features -- -D warnings
run cargo clippy --all-targets --all-features --release -- -D warnings
run cargo test --all-features
run cargo test --all-features --release

# --- examples ---------------------------------------------------------------------------------
run cargo build --examples

# --- documentation ----------------------------------------------------------------------------
run env RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items

# --- rule table -------------------------------------------------------------------------------
# Every specification rule id of the crate's rule table must be a rule of the specification; the
# check skips successfully when `spec-coverage/rules.csv` is absent. The second check is the other
# direction: every rule id the sources can emit must be in the table, or the issue constructor
# would panic on that path at runtime.
run "$PYTHON" scripts/check_rule_ids.py
run "$PYTHON" scripts/check_emitted_ids.py

# --- specification coverage matrix ------------------------------------------------------------
if [ -d "$SPEC" ]; then
    run "$PYTHON" scripts/spec_coverage.py --spec "$SPEC" --check
else
    skip "specification checkout not found at $SPEC (set SPEC=...)"
fi

# --- SBML syntactic test suite ----------------------------------------------------------------
if [ -d "$ROOT/syntactic" ]; then
    run cargo run --release --example test-suite-syntactic -- $(cat validated-rules.txt)
    # The filtered run above only reports the rules listed in `validated-rules.txt`; this second
    # gate runs the same example without a filter and compares the whole-corpus mismatch, issue
    # and round-trip reports against recorded snapshots, so rules (and documents) the filtered run
    # cannot see are still covered.
    run scripts/full_corpus_parity.sh
else
    skip "the SBML syntactic test suite is not unpacked in $ROOT/syntactic"
fi

printf '\n=== summary\n'
if [ "$failures" -eq 0 ]; then
    echo "all gates passed ($skipped skipped)"
else
    echo "$failures gate(s) failed ($skipped skipped)"
fi
exit "$failures"
