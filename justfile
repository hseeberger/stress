set shell := ["bash", "-uc"]
rust_version := `grep channel rust-toolchain.toml | sed -r 's/channel = "(.*)"/\1/'`

check:
	cargo check

fmt toolchain="+nightly":
	cargo {{toolchain}} fmt

fmt-check toolchain="+nightly":
	cargo {{toolchain}} fmt --check

lint:
	cargo clippy --no-deps --tests -- -D warnings

test:
	cargo test

fix:
	cargo fix --allow-dirty --allow-staged

doc toolchain="+nightly":
	RUSTDOCFLAGS="-D warnings --cfg docsrs" cargo {{toolchain}} doc --no-deps

all: check fmt lint test doc

run:
	RUST_LOG=stress=debug,otel=off,info \
		cargo run -p stress
