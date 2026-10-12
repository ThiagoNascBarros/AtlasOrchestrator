//! Perfis de agente em três camadas: prontos (no app), globais (`~/.config/atlasoq/profiles/`) e
//! do workspace (`.atlasoq/profiles/`). Em nomes iguais, a camada mais específica prevalece.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{Result, StoreError};
use crate::model::{LearnMode, LearnSettings};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileScope {
    Builtin,
    Global,
    Workspace,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    pub learn: LearnSettings,
    #[serde(skip, default = "default_scope")]
    pub scope: ProfileScope,
}

fn default_scope() -> ProfileScope {
    ProfileScope::Global
}

fn builtin(name: &str, command: &str, mode: LearnMode, prompt_flag: Option<&str>) -> Profile {
    let mut learn = LearnSettings::new(mode);
    learn.prompt_flag = prompt_flag.map(str::to_owned);
    Profile {
        name: name.to_owned(),
        command: command.to_owned(),
        args: Vec::new(),
        env: BTreeMap::new(),
        learn,
        scope: ProfileScope::Builtin,
    }
}

/// Perfis prontos (docs/spikes/r4-hooks-clis.md).
pub fn builtins() -> Vec<Profile> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_owned());
    vec![
        builtin("claude-code", "claude", LearnMode::Hook, None),
        builtin("codex", "codex", LearnMode::Prompt, None),
        builtin(
            "gemini",
            "gemini",
            LearnMode::Prompt,
            Some("--prompt-interactive"),
        ),
        builtin("opencode", "opencode", LearnMode::Prompt, Some("--prompt")),
        builtin("aider", "aider", LearnMode::Type, None),
        builtin("shell", &shell, LearnMode::Off, None),
    ]
}

fn load_dir(dir: &Path, scope: ProfileScope) -> Result<Vec<Profile>> {
    let mut out = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(out),
        Err(e) => return Err(StoreError::io(dir, e)),
    };
    for entry in entries {
        let path = entry.map_err(|e| StoreError::io(dir, e))?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let bytes = fs::read(&path).map_err(|e| StoreError::io(&path, e))?;
        let mut p: Profile = serde_json::from_slice(&bytes).map_err(|e| StoreError::Json {
            path: path.clone(),
            source: e,
        })?;
        p.scope = scope;
        out.push(p);
    }
    Ok(out)
}

/// Lista final de perfis, ordenada por nome.
pub fn resolve(global_dir: &Path, workspace_dir: Option<&Path>) -> Result<Vec<Profile>> {
    let mut by_name: BTreeMap<String, Profile> = BTreeMap::new();
    for p in builtins() {
        by_name.insert(p.name.clone(), p);
    }
    for p in load_dir(global_dir, ProfileScope::Global)? {
        by_name.insert(p.name.clone(), p);
    }
    if let Some(dir) = workspace_dir {
        for p in load_dir(dir, ProfileScope::Workspace)? {
            by_name.insert(p.name.clone(), p);
        }
    }
    Ok(by_name.into_values().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prontos_tem_os_modos_da_pesquisa() {
        let all = builtins();
        let mode = |n: &str| all.iter().find(|p| p.name == n).unwrap().learn.mode;
        assert_eq!(mode("claude-code"), LearnMode::Hook);
        assert_eq!(mode("codex"), LearnMode::Prompt);
        assert_eq!(mode("gemini"), LearnMode::Prompt);
        assert_eq!(mode("opencode"), LearnMode::Prompt);
        assert_eq!(mode("aider"), LearnMode::Type);
        assert_eq!(mode("shell"), LearnMode::Off);
    }

    #[test]
    fn workspace_prevalece_sobre_global_e_pronto() {
        let g = tempfile::tempdir().unwrap();
        let w = tempfile::tempdir().unwrap();
        fs::write(
            g.path().join("codex.json"),
            r#"{"name":"codex","command":"codex-global","learn":{"mode":"type"}}"#,
        )
        .unwrap();
        fs::write(
            w.path().join("codex.json"),
            r#"{"name":"codex","command":"codex-ws","learn":{"mode":"prompt"}}"#,
        )
        .unwrap();
        fs::write(
            w.path().join("revisor.json"),
            r#"{"name":"revisor","command":"claude","args":["--model","x"],"learn":{"mode":"hook"}}"#,
        )
        .unwrap();
        let all = resolve(g.path(), Some(w.path())).unwrap();
        let codex = all.iter().find(|p| p.name == "codex").unwrap();
        assert_eq!(codex.command, "codex-ws");
        assert_eq!(codex.scope, ProfileScope::Workspace);
        assert!(all.iter().any(|p| p.name == "revisor"));
        assert!(all
            .iter()
            .any(|p| p.name == "shell" && p.scope == ProfileScope::Builtin));
    }
}
