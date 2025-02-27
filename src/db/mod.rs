/// Functions for the database that take processed packets and insert them/etc
pub(crate) mod connection;
/// Functions that wrap a generic function to insert metric rows with no conflict handling
pub(crate) mod inserts;
/// `SQLite` module for writing data to tmpfs (RAM)
pub(crate) mod lite;
/// Postgres module for setting up the connection
pub(crate) mod postgres;
