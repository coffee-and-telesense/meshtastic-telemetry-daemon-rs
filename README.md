# Meshtastic Telemetry Daemon

Make sure you have Rust installed:
`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

## Building

### Build debug for x86

`cargo build --features debug`

### Build release for x86

`cargo build --features release --release`

### Build release for mips

You must have [cross](https://github.com/cross-rs/cross?tab=readme-ov-file#dependencies) installed with all its dependencies. Unless you are building natively on `mips`.

`CARGO_TARGET_MIPS_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-Zlocation-detail=none -Zfmt-debug=none" cross +nightly build --features release --release --target mips-unknown-linux-musl -Zbuild-std-features=optimize_for_size,panic_immediate_abort`

### Build debug for mips

This is not recommended due to `mips` devices typically having little free disk space and the debug binary being large.

## Running

To run: `cargo run --features debug`

This will use the `example_config.toml` file provided in the root of this repository.

Then specify the serial device connected via USB.

You could run the release version like `cargo run --features release --release` but this is also not recommended due to logs going straight to `journalctl` or wherever your system logs things by default.

## Notes

[`meshtastic`](https://docs.rs/meshtastic/0.1.6/meshtastic/) is their library for Rust. They also have a protobuf library. This crate is already installed using a fork from `gatlinnewhouse` that updated protobufs and lowered logging levels of `error!` to not pollute system logs.

[`tokio`](https://docs.rs/tokio/1.32.0/tokio/index.html) is the async runtime, learn it, it's great and already installed here. Ideally we'd be using just threads and threadpools, but the Meshtastic Rust crate uses tokio already.

[`serde`](https://crates.io/crates/serde) is the serialization/deserialization library, you can use it to parse the packets and grab what data we want and then do whatever with it.

Cargo.toml has more crates being used.

