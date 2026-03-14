RUSTUP_TOOLCHAIN ?= 1.93.0

.PHONY: fmt fmt-check clippy check test verify

fmt:
	cargo +$(RUSTUP_TOOLCHAIN) fmt

fmt-check:
	cargo +$(RUSTUP_TOOLCHAIN) fmt --check

clippy:
	cargo +$(RUSTUP_TOOLCHAIN) clippy --all-targets --all-features -- -D warnings

check:
	cargo +$(RUSTUP_TOOLCHAIN) check --all-targets --all-features

test:
	cargo +$(RUSTUP_TOOLCHAIN) test --all-targets --all-features

verify: fmt-check clippy check test

