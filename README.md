# Embedded Nano Mesh Telemetry Daemon (Formerly Meshtastic Telemetry Daemon)

Reads packets from a USB-connected [embedded-nano-mesh](https://github.com/boondocklabs/embedded-nano-mesh)
node and writes telemetry to a PostgreSQL database. Designed for long-running
unattended deployment on AREDN mesh network nodes and companion devices.

Versions before `v0.4.0` read data from Meshtastic networks over USB serial.
Versions including and after `v0.4.0` read data from our custom Rust firmware
based on `embedded-nano-mesh`.

## Requirements

* Rust toolchain, `rustup toolchain install stable`
* PostgreSQL instance and `libpq` (Install it from your package repo)
* A node running `embedded-nano-mesh` firmware connected via USB serial

Cross-compilation requires [cross](https://github.com/cross-rs/cross) and Docker.

## Configuration

On first run the daemon creates an example config at
`$XDG_CONFIG_HOME/meshtastic_telemetry/config.toml`.

Edit it in advance to ensure the daemon connects properly:

```toml
[postgres]
user = "postgres"
password = "postgres"
port = 5431
host = "localhost"
dbname = "meshtastic"
max_connections = 8
min_connections = 1

[serial]
port = "/dev/tty915" # leave blank to be prompted at startup
baud = 9600
device_addr = 1
listen_period = 150

[deployment]
location = "my-site" # scopes db queries to specific locations/tests
```

See [example_config.toml](./src/util/example_config.toml) for comments about
settings.

## Features

| Feature        | Description                                           |
|----------------|-------------------------------------------------------|
| `debug`          | Backtraces and per-node packet count logging          |
| `mimalloc`       | [mimalloc](https://github.com/microsoft/mimalloc) v3 global allocator                          |
| `journald`       | Write structured logs directly to the systemd journal |
| `trace`          | Verbose packet logging                                |
| `alpine`         | Shorthand: `debug`                                      |
| `beaglebone`     | Shorthand: `debug` + `mimalloc`                           |

## GitHub Releases

A release binary is built for each version tag

## Architecture

Two-thread synchronous design, no async runtime required! Serial reader thread
-> sync channel (`mpsc`) bounded at 32 -> DB writer threadpool (`r2d2` +
`diesel`).

Bounded `mpsc` provides backpressure. Clean shutdown on `SIGINT` and `SIGTERM`

## Documentation
```sh
cargo doc --features debug --no-deps --document-private-items --open
```
