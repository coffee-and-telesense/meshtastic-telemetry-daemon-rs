# Meshtastic Telemetry Daemon

Reads packets from a USB-connected Meshtastic node and writes telemetry to a
PostgreSQL database. Designed for long-running unattended deployment.

## Requirements

* Rust nightly toolchain, `rustup toolchain install nightly`
* PostgreSQL instance
* Meshtastic node connected via USB serial

Cross-compilation requires [cross](https://github.com/cross-rs/cross).

## Configuration

On first run the daemon creates an example config at
`~/.config/meshtastic_telemetry/config.toml`.

Edit it in advance to ensure the daemon connects properly:

```toml
[postgres]
user = "postgres"
password = "postgres"
port = 5431
host = "localhost"
dbname = "meshtastic"
max_connections = 8 # also half of in-flight task capacity (max 32)
min_connections = 1

[serial]
port = "/dev/tty915" # leave blank to be prompted at startup

[deployment]
location = "my-site" # scopes db queries to specific locations/tests
```

See [example_config.toml](./src/util/example_config.toml) for comments about
settings.

## Features

| Feature        | Description                                          |
|----------------|-------------------------------------------------------|
| `debug`        | Backtraces and per-node packet count logging         |
| `native-tls`   | System TLS for Postgres connections                  |
| `mimalloc`     | [mimalloc](https://github.com/microsoft/mimalloc) v3 global allocator                         |
| `rustls`       | Pure-Rust TLS (no system OpenSSL required)           |
| `journald`     | Write structured logs directly to the systemd journal|
| `log_perf`     | Log tokio runtime metrics on every packet            |
| `print-packets`| Pretty-print decoded packets as JSON to stdout       |
| `trace`        | Verbose logging of all Meshtastic packet types       |
| `tokio-console`| tokio-console async task inspector                   |
| `alpine`       | Shorthand: `native-tls` + `debug`                   |
| `beaglebone`   | Shorthand: `rustls` + `debug` + `mimalloc`           |

## GitHub Releases

A release binary is built for each version tag across four targets:

| Target                        | Target |
|-------------------------------|--------------|
| `x86_64-unknown-linux-musl`   | OpenWRT/AREDN devices    |
| `aarch64-unknown-linux-musl`  | OpenWRT/AREDN devices     |
| `armv7-unknown-linux-gnueabihf` | Beaglebone Black |
| `aarch64-unknown-linux-gnu`   | Raspberry Pi4 |

These releases ship with the `systemd` service files to deploy 915MHz and 433MHz
side by side, example configs, and an untested `install.sh` install script for
Pis.

In the future releases should probably be targeted for specific embedded devices
with CPU specific targets in compilation.

Also: builds no longer UPX compress binaries by default, so the OpenWRT/AREDN
builds may be too large and require manual UPX compression before installation.
Overall they've been deprecated in favor of using Pis attached to AREDN devices.

## Documentation
```sh
cargo doc --features debug --no-deps --document-private-items --open
```
