//! Comandos de conexão: criar, mudar direção, pausar, reativar, apagar e ver o histórico.

use atlasoq_core::{new_id, ErrorKind, IdKind};
use atlasoq_link::audit::AuditEvent;
use atlasoq_store::{ConnState, Connection, Direction};
use serde::Deserialize;
use serde_json::Value;
use tauri::State;

use crate::state::{AppState, OpenWorkspace};

#[derive(Deserialize)]
pub struct NewConnection {
    pub from: String,
    pub to: String,
    #[serde(default = "forward")]
    pub direction: Direction,
}

fn forward() -> Direction {
    Direction::Forward
}

fn name_of(open: &OpenWorkspace, id: &str) -> String {
    open.ws
        .node(id)
        .map_or_else(|| id.to_owned(), |n| n.name().as_str().to_owned())
}

fn audit(state: &AppState, event: &'static str, open_names: (String, String), conn: &Connection) {
    let mut ev = AuditEvent::new(event);
    ev.conn_id = Some(conn.id.clone());
    ev.from = Some(open_names.0);
    ev.to = Some(open_names.1);
    ev.result = Some(format!("{:?}", conn.direction).to_lowercase());
    state.broker().audit(&ev);
}

fn update<T>(
    state: &AppState,
    id: &str,
    f: impl FnOnce(&mut Connection) -> T,
) -> Result<(Connection, (String, String), T), String> {
    state.with_ws(|open| {
        let pos = open
            .ws
            .connections
            .iter()
            .position(|c| c.id == id)
            .ok_or("conexão inexistente")?;
        let out = f(&mut open.ws.connections[pos]);
        let conn = open.ws.connections[pos].clone();
        let names = (name_of(open, &conn.from), name_of(open, &conn.to));
        open.touch();
        Ok((conn, names, out))
    })
}

/// Cria a conexão `from → to`. Só o usuário cria conexões; auto-conexão e duplicatas são recusadas.
#[tauri::command]
pub fn connection_create(
    state: State<'_, AppState>,
    req: NewConnection,
) -> Result<Connection, String> {
    let (conn, names) = state.with_ws(|open| {
        if req.from == req.to {
            return Err("um nó não pode se conectar a si mesmo".to_owned());
        }
        for id in [&req.from, &req.to] {
            if open.ws.node(id).is_none() {
                return Err("nó inexistente".to_owned());
            }
        }
        let a_note = open.ws.node(&req.from).is_some_and(|n| n.is_note());
        let b_note = open.ws.node(&req.to).is_some_and(|n| n.is_note());
        if a_note && b_note {
            return Err("duas notas não podem ser conectadas".to_owned());
        }
        if open
            .ws
            .connections
            .iter()
            .any(|c| atlasoq_link::acl::links(c, &req.from, &req.to))
        {
            return Err("esses nós já estão conectados; mude a direção da conexão existente".to_owned());
        }
        let conn = Connection {
            id: new_id(IdKind::Connection),
            from: req.from.clone(),
            to: req.to.clone(),
            direction: req.direction,
            state: ConnState::Active,
            pause_reason: None,
            limits: open.ws.settings.limits_default,
        };
        open.ws.connections.push(conn.clone());
        open.touch();
        let names = (name_of(open, &conn.from), name_of(open, &conn.to));
        Ok((conn, names))
    })?;
    audit(&state, "connection_created", names, &conn);
    Ok(conn)
}

#[tauri::command]
pub fn connection_set_direction(
    state: State<'_, AppState>,
    id: String,
    direction: Direction,
) -> Result<Connection, String> {
    let (conn, _, ()) = update(&state, &id, |c| c.direction = direction)?;
    Ok(conn)
}

#[tauri::command]
pub fn connection_pause(state: State<'_, AppState>, id: String) -> Result<Connection, String> {
    let (conn, names, ()) = update(&state, &id, |c| {
        c.state = ConnState::Paused;
        c.pause_reason = None;
    })?;
    state.broker().connection_stopped(
        &id,
        ErrorKind::ConnectionPaused,
        "a conexão foi pausada pelo usuário.",
    );
    audit(&state, "connection_paused", names, &conn);
    Ok(conn)
}

#[tauri::command]
pub fn connection_resume(state: State<'_, AppState>, id: String) -> Result<Connection, String> {
    let (conn, names, ()) = update(&state, &id, |c| {
        c.state = ConnState::Active;
        c.pause_reason = None;
    })?;
    state.broker().reset_guard(&id);
    audit(&state, "connection_resumed", names, &conn);
    Ok(conn)
}

#[tauri::command]
pub fn connection_delete(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let (conn, names) = state.with_ws(|open| {
        let pos = open
            .ws
            .connections
            .iter()
            .position(|c| c.id == id)
            .ok_or("conexão inexistente")?;
        let conn = open.ws.connections.remove(pos);
        let names = (name_of(open, &conn.from), name_of(open, &conn.to));
        open.touch();
        Ok((conn, names))
    })?;
    state.broker().connection_stopped(
        &id,
        ErrorKind::NoConnection,
        "a conexão foi apagada pelo usuário.",
    );
    state.broker().reset_guard(&id);
    audit(&state, "connection_deleted", names, &conn);
    Ok(())
}

#[tauri::command]
pub fn connection_history(
    state: State<'_, AppState>,
    id: String,
    limit: Option<usize>,
) -> Vec<Value> {
    state.broker().history(&id, limit.unwrap_or(200))
}
