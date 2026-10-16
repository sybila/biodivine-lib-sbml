#!/usr/bin/env bash
#
# Whole-corpus validation gate for the SBML library.
#
#     scripts/full_corpus_parity.sh            # check the current tree against the baseline
#     scripts/full_corpus_parity.sh --record   # (re)record the baseline from the current corpus
#
# `cargo run --release --example test-suite-syntactic -- $(cat validated-rules.txt)` only reports
# mismatches for the rules this crate declares as checked, so a rule that the library checks but
# `validated-rules.txt` does not list is invisible to it. This gate runs the very same example with
# *no* rule filter, so the report covers every rule that appears anywhere in the corpus, and turns
# that report into a reviewable, order-independent snapshot by sorting it and counting duplicates.
#
# The example deliberately exits with a non-zero status when it finds mismatches, so this gate
# tolerates that exit code and compares the report files instead of the exit status. Because
# `cargo run` also exits 101 when the build or the run fails, the gate requires the example's
# `Report written.` marker in its output: that line is printed once the report files are written and
# before the example's own asserts fire. It is therefore the contract between the example and this
# script, so an edit to the example's output strings has to keep it.
#
# The snapshot under `scripts/full-corpus-baseline/` is derived from the (git-ignored) SBML
# syntactic test suite in `./syntactic`; its header records the corpus it was recorded from and a
# fingerprint that this script re-checks. See `VALIDATION.md`.
#
# The script needs the unpacked test suite in `./syntactic`; without it, it reports a skip and
# exits successfully, matching the convention of `scripts/verify.sh`. It overwrites the example's
# `test_suite_{error,warning,info}.txt` files and restores their previous contents on exit.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
[ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"

CORPUS="$ROOT/syntactic"
BASELINE="$ROOT/scripts/full-corpus-baseline"
REPORTS=(test_suite_error.txt test_suite_warning.txt test_suite_info.txt)

record=0
case "${1:-}" in
    --record) record=1 ;;
    -h|--help) sed -n '2,20p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    "") ;;
    *) echo "Unknown option: $1 (expected --record)"; exit 2 ;;
esac

if [ ! -d "$CORPUS" ]; then
    echo "SKIP: the SBML syntactic test suite is not unpacked in $CORPUS"
    echo "      (download a release from https://github.com/sbmlteam/sbml-test-suite/releases"
    echo "       and extract its syntactic subset there); nothing to check."
    exit 0
fi

# A fingerprint of the corpus, so that a snapshot recorded from one release is not compared against
# a different release. Order-independent (the file list is sorted before it is measured).
corpus_fingerprint() {
    local dirs files bytes
    dirs=$(find "$CORPUS" -mindepth 1 -maxdepth 1 -type d | wc -l)
    files=$(find "$CORPUS" -type f -name '*.xml' | wc -l)
    bytes=$(find "$CORPUS" -type f -name '*.xml' -print0 | sort -z | xargs -0 cat | wc -c)
    echo "rule_dirs=$dirs xml_files=$files xml_bytes=$bytes"
}

current_fingerprint=$(corpus_fingerprint)

if [ "$record" -eq 0 ]; then
    if [ ! -f "$BASELINE/fingerprint.txt" ]; then
        echo "FAIL: no recorded baseline in $BASELINE."
        echo "      Record one with: scripts/full_corpus_parity.sh --record"
        exit 1
    fi
    recorded_fingerprint=$(cat "$BASELINE/fingerprint.txt")
    if [ "$recorded_fingerprint" != "$current_fingerprint" ]; then
        echo "FAIL: the corpus fingerprint differs from the recorded baseline."
        echo "      recorded: $recorded_fingerprint"
        echo "      current:  $current_fingerprint"
        echo "      The baseline is stale; re-record it with: scripts/full_corpus_parity.sh --record"
        exit 1
    fi
fi

# Preserve the report files a previous (filtered) run may have written.
saved=()
for report in "${REPORTS[@]}"; do
    if [ -f "$report" ]; then
        cp "$report" "$report.full-corpus-parity.bak"
        saved+=("$report")
    fi
done
restore_reports() {
    local report
    for report in "${REPORTS[@]}"; do
        rm -f "$report"
    done
    for report in "${saved[@]:-}"; do
        [ -n "$report" ] && mv -f "$report.full-corpus-parity.bak" "$report"
    done
}
trap restore_reports EXIT

workdir=$(mktemp -d)
trap 'restore_reports; rm -rf "$workdir"' EXIT

echo "\$ cargo run --release --example test-suite-syntactic   # no rule filter"
cargo run --release --example test-suite-syntactic > "$workdir/run.log" 2>&1
status=$?
grep -A4 '^Found:' "$workdir/run.log" || true

# The example prints `Report written.` once it has written its report, just before its (by-design)
# asserts fire. Requiring it separates "the example ran and reported its result" (exit 0 or 101)
# from "the build or the run broke", which also exits 101 for `cargo run`.
if ! grep -q 'Report written\.' "$workdir/run.log"; then
    echo ""
    echo "FAIL: the test-suite example did not complete (no 'Report written.' in its output; exit status $status)."
    echo "      The report files were not compared. Last lines of the run:"
    tail -20 "$workdir/run.log" | sed 's/^/      /'
    exit 1
fi
if [ "$status" -ne 0 ] && [ "$status" -ne 101 ]; then
    echo "FAIL: the test-suite example exited with unexpected status $status"
    exit 1
fi
for report in "${REPORTS[@]}"; do
    if [ ! -f "$report" ]; then
        echo "FAIL: the example reported completion but did not write $report"
        exit 1
    fi
done

failed=0
for report in "${REPORTS[@]}"; do
    sort "$report" | uniq -c > "$workdir/$report.sorted"
    if [ "$record" -eq 1 ]; then
        continue
    fi
    if ! diff -u "$BASELINE/$report.sorted" "$workdir/$report.sorted" > "$workdir/$report.diff"; then
        echo ""
        echo "FAIL: $report differs from the recorded baseline:"
        cat "$workdir/$report.diff"
        failed=1
    fi
done

if [ "$record" -eq 1 ]; then
    mkdir -p "$BASELINE"
    for report in "${REPORTS[@]}"; do
        cp "$workdir/$report.sorted" "$BASELINE/$report.sorted"
    done
    echo "$current_fingerprint" > "$BASELINE/fingerprint.txt"
    echo ""
    echo "Recorded a new baseline in $BASELINE"
    echo "  corpus:  $current_fingerprint"
    echo "  recorded on: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    exit 0
fi

if [ "$failed" -ne 0 ]; then
    echo ""
    echo "The whole-corpus mismatch report changed. Corpus fingerprint: $current_fingerprint"
    exit 1
fi

echo ""
echo "The whole-corpus mismatch report matches the recorded baseline."
echo "  corpus: $current_fingerprint"
exit 0
