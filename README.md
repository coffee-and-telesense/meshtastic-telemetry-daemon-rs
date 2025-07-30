# Meshtastic Telemetry Daemon

Make sure you have Rust installed:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

You might want [UPX](https://github.com/upx/upx) installed to compress the binary depending on your target. Your distribution's repos should provide it, e.g. `sudo apt install upx`.

I've optimized for size a best as I can. Remaining wishes are: `gnutls` or `mbedtls` support through FFI instead of shipping `rustls`. This would save space in the binary.

## Building

### GitHub Releases

Currently GitHub actions are building binaries targeting three devices/use cases:
* Alpine Linux containers running `musl-x86_64` or `musl-arm64` (with native TLS, system logging, and postgres)
* Beaglebone Blacks running an Ubuntu distribution for the `gnueabihf-armv7` platform (with Rust TLS, debug logging to stdout, and postgres)

These builds should not require Rust to be installed, and should only require a valid config at `/etc/meshtastic_telem.toml` with an accessible Postgres database running.

### Build debug for `X86` or your native architecture

```sh
cargo build --features debug
```

### Build release for `X86` or your native architecture

```sh
cargo build --features release --release
```

### Build release for `ARMv8 64` (Raspberry Pi4 B)

```sh
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-Zlocation-detail=none -Zfmt-debug=none" cross +nightly build --features rustls,postgres,syslog --no-default-features --release --target aarch64-unknown-linux-gnu -Zbuild-std-features=optimize_for_size,panic_immediate_abort
```

### Build release for `X86_64` Alpine Linux Container

```sh
CARGO_TARGET_x86_64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-Zlocation-detail=none -Zfmt-debug=none" cross +nightly build --features alpine --no-default-features --release --target x86_64-unknown-linux-musl -Zbuild-std-features=optimize_for_size,panic_immediate_abort
```

### Build release for `ARM 64 bit` Alpine Linux Container

```sh
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-Zlocation-detail=none -Zfmt-debug=none" cross +nightly build --features alpine --no-default-features --release --target aarch64-unknown-linux-musl -Zbuild-std-features=optimize_for_size,panic_immediate_abort
```

### Build release for `MIPS`

You must have [cross](https://github.com/cross-rs/cross?tab=readme-ov-file#dependencies) installed with all its dependencies. Unless you are building natively on `MIPS`.

```sh
CARGO_TARGET_MIPS_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-Zlocation-detail=none -Zfmt-debug=none" cross +nightly build --features rustls,postgres,syslog --no-default-features --release --target mips-unknown-linux-musl -Zbuild-std-features=optimize_for_size,panic_immediate_abort

# Then compress the executable further with upx
upx --brute target/mips-unknown-linux-musl/release/meshtastic-telemetry-daemon-rs
```

Sometimes, running a cross compile after a normal compile will fail spitting out some errors about `GLIBC` versions. This is most likely due to build caching of some sort (I am not entirely sure). If you just run `cargo clean` and then re-run the cross compilation command above it should work.

### Build debug for `MIPS`

This is not recommended due to `MIPS` devices typically having little free disk space and the debug binary being large.

## Running

### Debug

To run the debug version (native TLS, postgres, and logging to stdout):

```sh
cargo run
```

This will use the `example_config.toml` file provided in the root of this repository.

Then specify the serial device connected via USB when prompted.

If you want to see Meshtastic packets as `json` output add `--features print-packets` to the above command.

### Release

Run the release version with:

```sh
cargo run --features alpine --no-default-features --release
```

But this is also not recommended due to logs going straight to `journalctl` or wherever your system logs things by default.

## Documentation

To view documentation on this, run the following after you install Rust and clone this repo:

```sh
cargo doc --features alpine --release --no-deps --document-private-items --open
```

Your browser should then open to this repo's documentation as generated from the Rust docstrings.

