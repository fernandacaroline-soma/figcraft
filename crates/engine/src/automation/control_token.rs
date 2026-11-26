//! The control channel's session token: `figcraft --control PORT` serves only clients that send it
//! first (`{"method": "auth", "params": {"token": …}}`), so another program on the same machine can't
//! drive the app (run commands, read and write files) through its loopback port.
//!
//! The app makes a random token when it starts and writes it to [`token_path`] (only its user can
//! read it), unless [`TOKEN_ENV`] gives one. Clients (the MCP server's attached backend, scripts)
//! read it from the same places. Here: only the names and the comparison; the app writes the file
//! and the clients read it (no automation roots apply: it is the app's own secret, not a document).

use std::path::PathBuf;

/// A token chosen by whoever starts the app (scripts, CI): the app uses it instead of a random one,
/// and clients read it before the file.
pub const TOKEN_ENV: &str = "FIGCRAFT_CONTROL_TOKEN";
/// The method a connection starts with.
pub const AUTH_METHOD: &str = "auth";

/// Where the app keeps the token of the control port `port`: the user's temporary folder (per user
/// on macOS and Windows; the file is the user's alone on Linux).
pub fn token_path(port: u16) -> PathBuf {
    std::env::temp_dir().join(format!("figcraft-control-{port}.token"))
}

/// Whether `given` is the token `expected`, in time that doesn't depend on where they differ.
pub fn matches(given: &str, expected: &str) -> bool {
    let (a, b) = (given.as_bytes(), expected.as_bytes());
    if a.len() != b.len() || b.is_empty() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}
