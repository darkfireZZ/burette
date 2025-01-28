
.PHONY: check
check: lint format-check doc-check unit_tests system_tests

.PHONY: system_tests
system_tests:
	bash system_tests/runner.sh

.PHONY: unit_tests
unit_tests:
	cargo test

.PHONY: lint
lint:
	cargo clippy -- -D warnings

.PHONY: format-check
format-check:
	cargo fmt --check

.PHONY: doc-check
doc-check:
	RUSTDOCFLAGS="-D warnings" cargo doc --document-private-items
