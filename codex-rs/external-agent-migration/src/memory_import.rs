//! Titanium: external-agent memory import is hard-disabled.
use codex_rollout::StateDbHandle;
use std::io;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct MemoryImportOutcome {
    pub synchronized_projects: Vec<String>,
    pub failures: Vec<MemoryImportFailure>,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct MemoryImportFailure {
    pub project_key: String,
    pub message: String,
}

pub(super) async fn import(
    _codex_home: &Path,
    _external_agent_home: &Path,
    _state_db: Option<&StateDbHandle>,
    _selected_memory: &[String],
) -> io::Result<MemoryImportOutcome> {
    Ok(MemoryImportOutcome {
        synchronized_projects: Vec::new(),
        failures: Vec::new(),
    })
}

pub(super) fn resources_root(codex_home: &Path) -> PathBuf {
    codex_home
        .join("extensions")
        .join("external_agent_import")
        .join("resources")
}
