//! Terminais na UI: conectar a uma sessão (snapshot + saída contínua), digitar e redimensionar.

use std::sync::atomic::Ordering;
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;

use atlasoq_pty::{Activity, SessionEvent, SessionHost};
use atlasoq_store::Node;
use serde::Serialize;
use tauri::ipc::{Channel, InvokeResponseBody};
use tauri::State;

use crate::state::AppState;

#[derive(Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StatusEvent {
    Status { value: &'static str },
    Exited { code: Option<i32> },
}

pub fn activity_name(a: Activity) -> &'static str {
    match a {
        Activity::Starting => "starting",
        Activity::Working => "working",
        Activity::Idle => "idle",
        Activity::Exited(_) => "exited",
    }
}

pub fn to_event(a: Activity) -> StatusEvent {
    match a {
        Activity::Exited(code) => StatusEvent::Exited { code },
        other => StatusEvent::Status {
            value: activity_name(other),
        },
    }
}

#[derive(Serialize)]
pub struct AttachInfo {
    pub attachment: u64,
    pub cols: u16,
    pub rows: u16,
}

/// Conecta a UI à sessão atual do agente. O primeiro envio em `output` é a tela atual; depois
/// vem a saída contínua, até `session_detach` ou o fim do processo.
#[tauri::command]
pub fn session_attach(
    state: State<'_, AppState>,
    agent_id: String,
    output: Channel<InvokeResponseBody>,
) -> Result<AttachInfo, String> {
    let sid = state.with_ws(|open| {
        open.sessions
            .get(&agent_id)
            .copied()
            .ok_or_else(|| "agente sem sessão".into())
    })?;
    let (snap, rx) = state.host.attach(sid).map_err(|e| e.to_string())?;
    output
        .send(InvokeResponseBody::Raw(snap.formatted))
        .map_err(|e| e.to_string())?;
    let (attachment, stop) = state.new_attachment();

    std::thread::Builder::new()
        .name("atlasoq-attach".into())
        .spawn(move || loop {
            if stop.load(Ordering::SeqCst) {
                break;
            }
            match rx.recv_timeout(Duration::from_millis(500)) {
                Ok(SessionEvent::Output(bytes)) => {
                    if output.send(InvokeResponseBody::Raw(bytes)).is_err() {
                        break;
                    }
                }
                Ok(SessionEvent::Status(_)) | Err(RecvTimeoutError::Timeout) => {}
                Ok(SessionEvent::Exited(_)) | Err(RecvTimeoutError::Disconnected) => break,
            }
        })
        .map_err(|e| e.to_string())?;

    Ok(AttachInfo {
        attachment,
        cols: snap.cols,
        rows: snap.rows,
    })
}

/// O terminal saiu da tela (LOD, apagado ou reconectado): para de enviar saída a ele.
#[tauri::command]
pub fn session_detach(state: State<'_, AppState>, attachment: u64) {
    state.end_attachment(attachment);
}

/// Teclas digitadas pelo usuário no terminal do agente.
#[tauri::command]
pub fn session_write(
    state: State<'_, AppState>,
    agent_id: String,
    data: String,
) -> Result<(), String> {
    let sid = state.with_ws(|open| {
        open.sessions
            .get(&agent_id)
            .copied()
            .ok_or_else(|| "agente sem sessão".into())
    })?;
    state
        .host
        .write_user(sid, data.as_bytes())
        .map_err(|e| e.to_string())
}

/// Novo tamanho do terminal (o usuário redimensionou o nó).
#[tauri::command]
pub fn session_resize(
    state: State<'_, AppState>,
    agent_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    let cols = cols.clamp(20, 500);
    let rows = rows.clamp(5, 200);
    let sid = state.with_ws(|open| {
        if let Some(Node::Agent(a)) = open.ws.nodes.iter_mut().find(|n| n.id() == agent_id) {
            if (a.terminal.cols, a.terminal.rows) != (cols, rows) {
                a.terminal.cols = cols;
                a.terminal.rows = rows;
                open.touch();
            }
        }
        Ok(open.sessions.get(&agent_id).copied())
    })?;
    if let Some(sid) = sid {
        state
            .host
            .resize(sid, cols, rows)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Texto recente da tela (usado pelos cartões de LOD).
#[tauri::command]
pub fn session_text(
    state: State<'_, AppState>,
    agent_id: String,
    lines: usize,
) -> Result<String, String> {
    let sid = state.with_ws(|open| {
        open.sessions
            .get(&agent_id)
            .copied()
            .ok_or_else(|| "agente sem sessão".into())
    })?;
    state
        .host
        .screen_text(sid, lines.min(200))
        .map_err(|e| e.to_string())
}
