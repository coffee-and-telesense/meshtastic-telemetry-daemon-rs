# Meshtastic Telemetry Daemon

Make sure you have Rust installed:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

You might want [UPX](https://github.com/upx/upx) installed to compress the binary depending on your target. Your distribution's repos should provide it, e.g. `sudo apt install upx`.

I've optimized for size a best as I can. Remaining wishes are: `gnutls` or `mbedtls` support through FFI instead of shipping `rustls`. This would save space in the binary.

## Building

### Build debug for `X86` or your native architecture

```sh
cargo build --features debug
```

### Build release for `X86` or your native architecture

```sh
cargo build --features release --release
```

### Build release for `MIPS`

You must have [cross](https://github.com/cross-rs/cross?tab=readme-ov-file#dependencies) installed with all its dependencies. Unless you are building natively on `MIPS`.

```sh
CARGO_TARGET_MIPS_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-Zlocation-detail=none -Zfmt-debug=none" cross +nightly build --features release --release --target mips-unknown-linux-musl -Zbuild-std-features=optimize_for_size,panic_immediate_abort

# Then compress the executable further with upx
upx --brute target/mips-unknown-linux-musl/release/meshtastic-telemetry-daemon-rs
```

Sometimes, running a cross compile after a normal compile will fail spitting out some errors about `GLIBC` versions. This is most likely due to build caching of some sort (I am not entirely sure). If you just run `cargo clean` and then re-run the cross compilation command above it should work.

### Build debug for `MIPS`

This is not recommended due to `MIPS` devices typically having little free disk space and the debug binary being large.

## Running

### Debug

To run the debug version:

```sh
cargo run --features debug
```

This will use the `example_config.toml` file provided in the root of this repository.

Then specify the serial device connected via USB when prompted.

If you want to see Meshtastic packets as `json` output add `--features debug,print-packets` to the above command.

### Release

Run the release version with:

```sh
cargo run --features release --release
```

But this is also not recommended due to logs going straight to `journalctl` or wherever your system logs things by default.

## Documentation

To view documentation on this, run the following after you install Rust and clone this repo:

```sh
cargo doc --features release --release --no-deps --document-private-items --open
```

Your browser should then open to this repo's documentation as generated from the Rust docstrings.

## Notes

[`meshtastic`](https://docs.rs/meshtastic/0.1.6/meshtastic/) is their library for Rust. They also have a protobuf library. This crate is already installed using a fork from `gatlinnewhouse` that updated protobufs and lowered logging levels of `error!` to not pollute system logs.

[`tokio`](https://docs.rs/tokio/1.32.0/tokio/index.html) is the async runtime, learn it, it's great and already installed here. Ideally we'd be using just threads and threadpools, but the Meshtastic Rust crate uses tokio already.

[`serde`](https://crates.io/crates/serde) is the serialization/deserialization library, you can use it to parse the packets and grab what data we want and then do whatever with it.

Cargo.toml has more crates being used.

