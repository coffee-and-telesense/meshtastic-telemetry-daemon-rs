# Meshtastic Telemetry Daemon

Make sure you have Rust installed:
`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

## Building

`cargo build`

## Running

To run: `cargo run`

Then specify the serial device connected via USB.

## Notes

[`tokio`](https://docs.rs/tokio/1.32.0/tokio/index.html) is the async runtime, learn it, it's great and already installed here.

[`meshtastic`](https://docs.rs/meshtastic/0.1.6/meshtastic/) is their library for Rust. They also have a protobuf library. This crate is also already installed.

[`sqlx`](https://crates.io/crates/sqlx) is the definitive SQL toolkit for Rust.

[`reqwest`](https://crates.io/crates/reqwest) is a crate for sending HTTP post requests etc. If we need to send our data to a database, perhaps we use this on the serial connection side to send it to a daemon that is receiving requests to put into the database.

[`serde`](https://crates.io/crates/serde) is the serialization/deserialization library, you can use it to parse the packets and grab what data we want and then do whatever with it.
