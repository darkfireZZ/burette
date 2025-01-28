
.PHONY: check
check: lint format-check doc-check test

.PHONY: test
test:
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
