# Convenience entry points for building and checking this repository. Everything here is also
# runnable by hand; `make verify` runs every gate and prints each command with its exit code.
.PHONY: help build test lint docs bindings examples rules coverage full-corpus verify

help:
	@echo "make build       build the crate (and the workspace member crates)"
	@echo "make rules       check the rule table of src/validation/rules.rs: every specification"
	@echo "                 rule id is a real rule of the specification, and every rule id the"
	@echo "                 sources emit is registered"
	@echo "make test        run the Rust test suite"
	@echo "make lint        cargo fmt --check and cargo clippy (warnings denied)"
	@echo "make docs        check the generated coverage page and build rustdoc with warnings denied"
	@echo "make bindings    regenerate (RUSTDOC=1) or check the bindings manifest of the Python"
	@echo "                 wrappers in biodivine-lib-sbml-py-sys/BINDINGS.md"
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
	python3 scripts/gen_rule_coverage_doc.py --check
	RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

# The bindings manifest and the generated wrappers: two artifacts generated from the crate's public
# surface, and one command that keeps both in step with it. The documentation is rebuilt first,
# because the manifest describes the *public* surface and the rustdoc gate of this repository
# documents private items too: the two builds share `target/doc`, so the build has to be part of this
# target. `make bindings WRITE=1` regenerates both files; without it, the target only checks that the
# committed copies are current (which is what `scripts/verify.sh` runs).
bindings:
	cargo doc --no-deps
	python3 biodivine-lib-sbml-py-sys/scripts/public_surface.py $(if $(WRITE),,--check)
	python3 biodivine-lib-sbml-py-sys/scripts/gen_wrappers.py $(if $(WRITE),,--check)

# The three artifacts of this repository that are generated and must equal their committed copy: the
# coverage matrix (from the specification sources), the documentation page rendered from it, and the
# doc sentences of the typed accessors, which a test re-derives from the code. One command, so there
# is one place where the convention is visible.
docs-check:
	python3 scripts/spec_coverage.py --spec "$${SPEC:-../../sbml-specifications}" --check
	python3 scripts/gen_rule_coverage_doc.py --check
	cargo test --lib a_doc_sentence_names_the_xml_name_the_code_actually_uses

examples:
	@for example in $$(ls examples/*.rs | sed 's|examples/||; s|\.rs||'); do \
		echo "== $$example"; cargo run --quiet --example $$example >/dev/null || exit 1; \
	done

# Check the crate's rule table against the specification rule list of the coverage matrix: every
# descriptor whose kind is `RuleKind::SpecRule` must name a rule the specification defines. The
# check skips successfully when the coverage matrix is absent.
rules:
	python3 scripts/corpus_duplicates.py --dir scripts/full-corpus-baseline --expect 1
	python3 scripts/check_rule_ids.py
	python3 scripts/check_emitted_ids.py
	python3 scripts/spec_ids_audit.py --spec "$${SPEC:-../../sbml-specifications}" --check
	python3 scripts/corpus_duplicates.py --dir scripts/full-corpus-baseline
	python3 scripts/gen_rule_coverage_doc.py --check

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
