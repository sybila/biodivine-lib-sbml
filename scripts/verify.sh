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
# The workspace has two kinds of members. The library, `sbml-macros` and `sbml-test-suite` are the
# crates of the repository; `biodivine-lib-sbml-py-sys` is the thin PyO3 wrapper crate, whose tests
# need an embedded Python interpreter (they link libpython, so `libpython3.x-dev` has to be
# installed). The library's own gates below do not include the wrappers: `cargo test` in the
# repository root tests the root package alone, which keeps the numbers this file reports the
# library's own.
run cargo fmt --all -- --check
run cargo clippy --all-targets --all-features -- -D warnings
run cargo clippy --all-targets --all-features --release -- -D warnings
run cargo test --all-features
run cargo test --all-features --release

# --- Python wrappers --------------------------------------------------------------------------
# The wrapper crate's tests embed an interpreter, so they link `libpython`. A machine without the
# development package of the interpreter (the `libpython3.x.so` the linker needs) cannot run them;
# that is an environment prerequisite, not a defect of the repository, so it is reported as skipped
# with the reason instead of as a failure.
python_lib=$("$PYTHON" - <<'EOF'
import os, sysconfig
directory = sysconfig.get_config_var("LIBDIR")
library = sysconfig.get_config_var("LDLIBRARY")
print(os.path.join(directory, library) if directory and library else "")
EOF
)
# `BINDINGS.md` states what every public item of the crate maps to; it is generated from rustdoc's
# output, so it cannot lose an item silently, and this gate keeps it in step with the code. The
# documentation is rebuilt from the public surface first: the rustdoc gate above documents private
# items too, and both builds share `target/doc`.
run cargo doc --no-deps
run "$PYTHON" biodivine-lib-sbml-py-sys/scripts/public_surface.py --check
if [ -n "$python_lib" ] && [ -e "$python_lib" ]; then
	run cargo test -p biodivine-lib-sbml-py-sys
	run cargo clippy -p biodivine-lib-sbml-py-sys --all-targets -- -D warnings
	run env RUSTDOCFLAGS="-D warnings" cargo doc -p biodivine-lib-sbml-py-sys --no-deps
else
	skip "the Python shared library ($python_lib) is not installed: install python3-dev (or the \
equivalent) to run the wrapper crate's tests"
fi

# --- examples ---------------------------------------------------------------------------------
run cargo build --examples

# --- documentation ----------------------------------------------------------------------------
# The coverage page of the crate's documentation is generated from the coverage matrix; this check
# fails when it is stale, so the page cannot describe a state the matrix no longer holds.
run "$PYTHON" scripts/gen_rule_coverage_doc.py --check
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
    # The other direction of the same claim: the matrix must not have lost a rule id the released
    # documents define, because a lost id silently leaves the scope of every ranking in this repo.
    run "$PYTHON" scripts/spec_ids_audit.py --spec "$SPEC" --check
else
    skip "specification checkout not found at $SPEC (set SPEC=...)"
fi

# --- SBML syntactic test suite ----------------------------------------------------------------
if [ -d "$ROOT/syntactic" ]; then
    # Only the *enabled* rule ids are passed (a bare id; `#`-commented ones are documented but not
    # compared). Extracting them explicitly matters: `$(cat validated-rules.txt)` relied on the
    # shell treating a `#id` word as a comment, so any other line in the file silently became a
    # nonsense argument and the gate degraded instead of failing.
    enabled_rules=$(sed -nE '/^[A-Za-z0-9_-]+$/p' validated-rules.txt | tr '\n' ' ')
    run cargo run --release --example test-suite-syntactic -- $enabled_rules
    # The filtered run above only reports the rules listed in `validated-rules.txt`; this second
    # gate runs the same example without a filter and compares the whole-corpus mismatch, issue
    # and round-trip reports against recorded snapshots, so rules (and documents) the filtered run
    # cannot see are still covered.
    run scripts/full_corpus_parity.sh
    # The census compares a document's reported rules as a *set*, so a rule the library reports twice
    # for one document moves no number at all. This reads the multiplicity out of the issue dump and
    # pins it: the recorded baseline has exactly one such pair (`layout-21603`), and a new one fails
    # here rather than hiding behind an unchanged census.
    run "$PYTHON" scripts/corpus_duplicates.py --dir scripts/full-corpus-baseline --expect 1
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
