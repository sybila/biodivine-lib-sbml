# Convenience entry points for building and checking this repository. Everything here is also
# runnable by hand; `make verify` runs every gate and prints each command with its exit code.
.PHONY: help build test lint docs examples rules coverage full-corpus verify

help:
	@echo "make build       build the crate (and the workspace member crates)"
	@echo "make rules       check that every specification rule id in src/validation/rules.rs"
	@echo "                 is a real rule of the specification"
	@echo "make test        run the Rust test suite"
	@echo "make lint        cargo fmt --check and cargo clippy (warnings denied)"
	@echo "make docs        build rustdoc with warnings denied"
	@echo "make examples    run every example"
	@echo "make coverage    regenerate the specification coverage matrix"
	@echo "make full-corpus check the whole-corpus mismatch, issue and round-trip reports"
	@echo "                 against their recorded baselines"
	@echo "make verify      run every gate and report each exit code"
	@echo ""
	@echo "The SBML syntactic test suite needs an unpacked ./syntactic folder; see VALIDATION.md."

build:
	cargo build --all-features

test:
	cargo test --all-features

lint:
	cargo fmt --check
	cargo clippy --all-targets --all-features -- -D warnings

docs:
	RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

examples:
	@for example in $$(ls examples/*.rs | sed 's|examples/||; s|\.rs||'); do \
		echo "== $$example"; cargo run --quiet --example $$example >/dev/null || exit 1; \
	done

# Check the crate's rule table against the specification rule list of the coverage matrix: every
# descriptor whose kind is `RuleKind::SpecRule` must name a rule the specification defines. The
# check skips successfully when the coverage matrix is absent.
rules:
	python3 scripts/check_rule_ids.py

# Regenerate the coverage matrix from the SBML specification sources. Pass the checkout with
# SPEC=/path/to/sbml-specifications.
coverage:
	python3 scripts/spec_coverage.py --spec "$${SPEC:-../../sbml-specifications}"

# Compare the whole-corpus syntactic-test-suite report against the recorded baseline in
# scripts/full-corpus-baseline/. Re-record with `make full-corpus RECORD=1`.
full-corpus:
	@if [ "$${RECORD:-0}" = "1" ]; then \
		scripts/full_corpus_parity.sh --record; \
	else \
		scripts/full_corpus_parity.sh; \
	fi

verify:
	scripts/verify.sh
