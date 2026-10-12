//! Modo `hook`: gera `claude-settings.json` dentro de `.atlasoq/agents/<nome>/`, com o hook
//! `SessionStart` que roda `atlasoq learn --hook` e a permissão para o `atlasoq`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

/// Eventos que reinjetam o tutorial: início, retomada, limpeza e compactação de contexto.
pub const MATCHERS: &str = "startup|resume|clear|compact";

pub fn settings() -> Value {
    json!({
        "hooks": {
            "SessionStart": [{
                "matcher": MATCHERS,
                "hooks": [{ "type": "command", "command": "atlasoq learn --hook" }]
            }]
        },
        "permissions": { "allow": ["Bash(atlasoq:*)"] }
    })
}

/// Grava o arquivo de configuração do agente e devolve o caminho dele.
pub fn write(agent_dir: &Path) -> io::Result<PathBuf> {
    fs::create_dir_all(agent_dir)?;
    let path = agent_dir.join("claude-settings.json");
    let text = serde_json::to_string_pretty(&settings())
        .map_err(io::Error::other)?;
    fs::write(&path, text)?;
    Ok(path)
}
