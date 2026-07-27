/// The current Codex CLI version as embedded at compile time.
#[cfg(not(test))]
pub const CODEX_CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Test builds pin the displayed version so version-bearing snapshots stay
/// reproducible across release bumps (upstream snapshots are recorded as v0.0.0).
#[cfg(test)]
pub const CODEX_CLI_VERSION: &str = "0.0.0";
