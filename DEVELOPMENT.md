# Development

## Setup

Install Rust with [rustup](https://rustup.rs). The crate supports Rust 1.88 and newer.

## Run Tests

The unit tests run against a local mock HTTP server and need no credentials:

```shell
cargo test
```

Live end-to-end coverage against the deployed Mailisk API lives in the main
repository's `mailisk-e2e-testing` project (`npm run test:rust:dev`).

## Other Checks

```shell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo doc --no-deps
cargo check --no-default-features --features native-tls
cargo +1.88 test
```

## Build And Publish

Before publishing, confirm the version in `Cargo.toml`.

```shell
cargo test
cargo publish --dry-run
cargo package --list
```

Publish to crates.io:

```shell
cargo login
cargo publish
```

After publishing, verify installation in a fresh project:

```shell
cargo new /tmp/mailisk-install-test
cd /tmp/mailisk-install-test
cargo add mailisk
cargo check
```
