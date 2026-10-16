# Convenience entry points for building and checking this repository. Everything here is also
# runnable by hand; `make verify` runs every gate and prints each command with its exit code.
.PHONY: help build test lint docs examples coverage verify

help:
	@echo "make build       build the crate (and the workspace member crates)"
	@echo "make test        run the Rust test suite"
	@echo "make lint        cargo fmt --check and cargo clippy (warnings denied)"
	@echo "make docs        build rustdoc with warnings denied"
	@echo "make examples    run every example"
	@echo "make coverage    regenerate the specification coverage matrix"
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

# Regenerate the coverage matrix from the SBML specification sources. Pass the checkout with
# SPEC=/path/to/sbml-specifications.
coverage:
	python3 scripts/spec_coverage.py --spec "$${SPEC:-../../../sbml-specifications}"

verify:
	scripts/verify.sh
