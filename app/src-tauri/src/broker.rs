//! Liga o broker de conexões ao app: o app é o `Host` do broker, que decide e entrega.

use std::path::PathBuf;
use std::sync::Arc;

use atlasoq_link::{socket_path, Broker, Host, LinkEvent, NodeInfo, Server, SessionInfo};
use atlasoq_pty::{Activity, SessionHost};
use atlasoq_store::{ConnState, Connection, Node, PauseReason};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::state::AppState;

/// Mudança de estado de uma conexão (pausa automática, retomada).
pub const EVENT_CONNECTION: &str = "connection-changed";
/// Uma mensagem passou por uma conexão (a UI anima a linha).
pub const EVENT_LINK_MESSAGE: &str = "link-message";

#[derive(Clone, Serialize)]
struct MessageEvent {
    conn_id: String,
    from_id: String,
    to_id: String,
    kind: &'static str,
}

struct AppHost {
    app: AppHandle,
}

impl AppHost {
    fn state(&self) -> tauri::State<'_, AppState> {
        self.app.state::<AppState>()
    }

    fn session_id(&self, node_id: &str) -> Option<u64> {
        self.state()
            .workspace()
            .as_ref()
            .and_then(|o| o.sessions.get(node_id).copied())
    }
}

impl Host for AppHost {
    fn nodes(&self) -> Vec<NodeInfo> {
        self.state()
            .workspace()
            .as_ref()
            .map(|o| {
                o.ws.nodes
                    .iter()
                    .map(|n| NodeInfo {
                        id: n.id().to_owned(),
                        name: n.name().as_str().to_owned(),
                        role: match n {
                            Node::Agent(a) => a.role.clone(),
                            Node::Note(_) => String::new(),
                        },
                        is_note: n.is_note(),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn connections(&self) -> Vec<Connection> {
        self.state()
            .workspace()
            .as_ref()
            .map(|o| o.ws.connections.clone())
            .unwrap_or_default()
    }

    fn session(&self, node_id: &str) -> Option<SessionInfo> {
        let sid = self.session_id(node_id)?;
        let host = &self.state().host;
        let activity = host.activity(sid).ok()?;
        Some(SessionInfo {
            status: match activity {
                Activity::Starting => "starting",
                Activity::Working => "working",
                Activity::Idle => "idle",
                Activity::Exited(_) => "exited",
            },
            exited: matches!(activity, Activity::Exited(_)),
            bracketed_paste: host.bracketed_paste(sid).unwrap_or(false),
            last_user_input: host.last_user_input(sid).ok().flatten(),
        })
    }

    fn write(&self, node_id: &str, data: &[u8]) -> Result<(), String> {
        let sid = self.session_id(node_id).ok_or("agente sem sessão")?;
        self.state().host.write(sid, data).map_err(|e| e.to_string())
    }

    fn screen_text(&self, node_id: &str, lines: usize) -> Result<String, String> {
        let sid = self.session_id(node_id).ok_or("agente sem sessão")?;
        self.state()
            .host
            .screen_text(sid, lines)
            .map_err(|e| e.to_string())
    }

    fn set_connection_state(&self, conn_id: &str, state: ConnState, reason: Option<PauseReason>) {
        let changed = self.state().with_ws(|open| {
            let conn = open
                .ws
                .connections
                .iter_mut()
                .find(|c| c.id == conn_id)
                .ok_or_else(|| "conexão inexistente".to_owned())?;
            conn.state = state;
            conn.pause_reason = reason;
            let conn = conn.clone();
            open.touch();
            Ok(conn)
        });
        if let Ok(conn) = changed {
            let _ = self.app.emit(EVENT_CONNECTION, conn);
        }
    }

    fn logs_dir(&self) -> Option<PathBuf> {
        self.state().workspace().as_ref().map(|o| o.dir.logs_dir())
    }

    fn emit(&self, event: LinkEvent) {
        let LinkEvent::Message {
            conn_id,
            from_id,
            to_id,
            kind,
        } = event;
        let _ = self.app.emit(
            EVENT_LINK_MESSAGE,
            MessageEvent {
                conn_id,
                from_id,
                to_id,
                kind,
            },
        );
    }
}

/// Diretório do binário `atlasoq` (ao lado do executável do app); entra no `PATH` dos agentes.
pub fn cli_dir() -> Option<PathBuf> {
    std::env::current_exe().ok()?.parent().map(PathBuf::from)
}

/// Inicia o broker e o servidor de socket, e guarda os dois no estado do app.
pub fn start(app: &AppHandle) -> Result<(), String> {
    let broker = Arc::new(Broker::new(Arc::new(AppHost { app: app.clone() })));
    let path = socket_path(&std::process::id().to_string());
    let server = Server::start(broker.clone(), path)
        .map_err(|e| format!("não foi possível abrir o socket do atlasoq: {e}"))?;
    app.state::<AppState>().set_broker(broker, server);
    Ok(())
}
