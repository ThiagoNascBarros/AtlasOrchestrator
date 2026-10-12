//! Início dos processos dos agentes a partir do workspace.

use std::sync::Arc;

use atlasoq_pty::{LocalSessionHost, SessionEvent, SessionHost, SessionId, SessionSpec};
use atlasoq_store::Agent;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::broker::cli_dir;
use crate::commands::session::{to_event, StatusEvent};
use crate::state::{AppState, OpenWorkspace};

/// Evento global com o status de um agente (cartões e barras de título usam).
pub const EVENT_AGENT_STATUS: &str = "agent-status";

#[derive(Clone, Serialize)]
struct AgentStatus {
    agent_id: String,
    session: SessionId,
    #[serde(flatten)]
    status: StatusEvent,
}

/// Monta o `SessionSpec` de um agente: comando, diretório (relativo à raiz) e ambiente.
pub fn session_spec(open: &OpenWorkspace, agent: &Agent) -> SessionSpec {
    let mut spec = SessionSpec::new(
        agent.terminal.command.clone(),
        open.root().join(&agent.terminal.cwd),
    );
    spec.args = agent.terminal.args.clone();
    spec.cols = agent.terminal.cols;
    spec.rows = agent.terminal.rows;
    spec.env = agent
        .terminal
        .env
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    spec
}

/// Inicia (ou reinicia) o processo do agente, registra a sessão e publica o status dela.
pub fn launch(
    app: &AppHandle,
    host: &Arc<LocalSessionHost>,
    open: &mut OpenWorkspace,
    agent: &Agent,
) -> Result<SessionId, String> {
    let mut spec = session_spec(open, agent);
    let state = app.state::<AppState>();
    let broker = state.broker();
    // Cada sessão nasce com token novo; o do processo anterior deixa de valer.
    if let Some(socket) = state.socket_path() {
        spec.env
            .push(("ATLASOQ_SOCKET".into(), socket.display().to_string()));
        spec.env.push(("ATLASOQ_NODE_ID".into(), agent.id.clone()));
        spec.env
            .push(("ATLASOQ_NODE_TOKEN".into(), broker.issue_token(&agent.id)));
    }
    if let Some(dir) = cli_dir() {
        let old = std::env::var("PATH").unwrap_or_default();
        spec.env
            .push(("PATH".into(), format!("{}:{old}", dir.display())));
    }
    broker.node_stopped(&agent.id);
    let id = host
        .spawn(spec)
        .map_err(|e| format!("não foi possível iniciar \"{}\": {e}", agent.name))?;
    open.sessions.insert(agent.id.clone(), id);

    let rx = host.subscribe(id).map_err(|e| e.to_string())?;
    let (app, agent_id) = (app.clone(), agent.id.clone());
    let exit_broker = broker.clone();
    let node_id = agent.id.clone();
    let emit = move |status: StatusEvent| {
        let _ = app.emit(
            EVENT_AGENT_STATUS,
            AgentStatus {
                agent_id: agent_id.clone(),
                session: id,
                status,
            },
        );
    };
    emit(to_event(host.activity(id).map_err(|e| e.to_string())?));
    std::thread::Builder::new()
        .name("atlasoq-status".into())
        .spawn(move || {
            for ev in rx {
                match ev {
                    SessionEvent::Status(a) => emit(to_event(a)),
                    SessionEvent::Exited(code) => {
                        exit_broker.node_stopped(&node_id);
                        emit(StatusEvent::Exited { code });
                        break;
                    }
                    SessionEvent::Output(_) => {}
                }
            }
        })
        .map_err(|e| e.to_string())?;
    Ok(id)
}
