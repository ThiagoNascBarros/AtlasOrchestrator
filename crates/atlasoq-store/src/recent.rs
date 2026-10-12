//! Lista de workspaces recentes em `~/.local/state/atlasoq/recent.json`.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::atomic::write_atomic;
use crate::error::{Result, StoreError};

const MAX_RECENT: usize = 15;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecentEntry {
    pub root: PathBuf,
    pub name: String,
    /// Segundos desde a época Unix.
    pub opened_at: u64,
}

pub fn load(file: &Path) -> Vec<RecentEntry> {
    fs::read(file)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

/// Coloca o workspace no topo da lista e remove entradas cujas pastas não existem mais.
pub fn touch(file: &Path, root: &Path, name: &str) -> Result<Vec<RecentEntry>> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut list: Vec<RecentEntry> = load(file)
        .into_iter()
        .filter(|e| e.root != root && e.root.exists())
        .collect();
    list.insert(
        0,
        RecentEntry {
            root: root.to_path_buf(),
            name: name.to_owned(),
            opened_at: now,
        },
    );
    list.truncate(MAX_RECENT);
    if let Some(dir) = file.parent() {
        fs::create_dir_all(dir).map_err(|e| StoreError::io(dir, e))?;
    }
    let json = serde_json::to_vec_pretty(&list).map_err(|e| StoreError::Json {
        path: file.to_path_buf(),
        source: e,
    })?;
    write_atomic(file, &json)?;
    Ok(list)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mais_recente_primeiro_sem_duplicar() {
        let state = tempfile::tempdir().unwrap();
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        let file = state.path().join("recent.json");
        touch(&file, a.path(), "a").unwrap();
        touch(&file, b.path(), "b").unwrap();
        let list = touch(&file, a.path(), "a").unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "a");
        assert_eq!(load(&file), list);
    }
}
