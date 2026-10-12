//! Abrir arquivos e links clicados no terminal (FR-011).

use std::path::Path;
use std::process::{Command, Stdio};

use tauri::State;

use crate::state::AppState;

fn spawn_detached(program: &str, args: &[String]) -> Result<(), String> {
    Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("não foi possível executar {program}: {e}"))
}

/// Abre uma URL http(s) no navegador do sistema.
#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err("só links http e https podem ser abertos".into());
    }
    spawn_detached("xdg-open", &[url])
}

/// Abre `path` (relativo ao diretório do agente ou absoluto) no editor gráfico do usuário
/// (`$VISUAL`) na linha indicada, ou com `xdg-open`.
#[tauri::command]
pub fn open_path(
    state: State<'_, AppState>,
    agent_id: String,
    path: String,
    line: Option<u32>,
) -> Result<(), String> {
    let base = state.with_ws(|open| {
        let agent = open
            .ws
            .agents()
            .find(|a| a.id == agent_id)
            .ok_or("agente inexistente")?;
        Ok(open.root().join(&agent.terminal.cwd))
    })?;
    let full = if Path::new(&path).is_absolute() {
        Path::new(&path).to_path_buf()
    } else {
        base.join(&path)
    };
    if !full.exists() {
        return Err(format!("{} não existe", full.display()));
    }
    let full = full.to_string_lossy().into_owned();
    match std::env::var("VISUAL")
        .ok()
        .filter(|v| !v.trim().is_empty())
    {
        Some(editor) => {
            let program = editor
                .split_whitespace()
                .next()
                .unwrap_or_default()
                .to_owned();
            let mut args: Vec<String> = editor
                .split_whitespace()
                .skip(1)
                .map(str::to_owned)
                .collect();
            let name = Path::new(&program)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            match (name, line) {
                ("code" | "codium" | "code-insiders", Some(l)) => {
                    args.extend(["-g".into(), format!("{full}:{l}")])
                }
                ("subl" | "zed" | "gnome-text-editor", Some(l)) => args.push(format!("{full}:{l}")),
                _ => args.push(full),
            }
            spawn_detached(&program, &args)
        }
        None => spawn_detached("xdg-open", &[full]),
    }
}
