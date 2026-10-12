//! Início dos processos dos agentes a partir do workspace.

use std::sync::Arc;

use atlasoq_pty::{LocalSessionHost, SessionEvent, SessionHost, SessionId, SessionSpec};
use std::time::Duration;

use atlasoq_link::{learn_content, learn_hook, learn_prompt};
use atlasoq_store::{Agent, LearnMode};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::broker::{cli_dir, learn_files, node_infos};
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

/// Texto de aprendizado do agente, montado agora (modos `prompt` e `type`).
fn learn_text(open: &OpenWorkspace, agent: &Agent) -> String {
    let nodes = node_infos(&open.ws);
    let Some(me) = nodes.iter().find(|n| n.id == agent.id) else {
        return String::new();
    };
    let files = learn_files(open, agent.name.as_str());
    learn_content::text_for(me, &nodes, &open.ws.connections, &files)
}

/// Sessão atual do agente é `sid`? (Saídas de sessões antigas não devem mexer no estado.)
fn is_current(app: &AppHandle, agent_id: &str, sid: SessionId) -> bool {
    app.state::<AppState>()
        .workspace()
        .as_ref()
        .and_then(|o| o.sessions.get(agent_id).copied())
        == Some(sid)
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
    let mode = agent.learn.mode;
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
    // Aprendizado: o hook é um arquivo de configuração; o prompt vai na linha de comando.
    let text = match mode {
        LearnMode::Prompt | LearnMode::Type => learn_text(open, agent),
        LearnMode::Hook | LearnMode::Off => String::new(),
    };
    match mode {
        LearnMode::Hook => {
            let dir = open.dir.agents_dir().join(agent.name.as_str());
            let path = learn_hook::write(&dir).map_err(|e| {
                format!("não foi possível gravar o hook de \"{}\": {e}", agent.name)
            })?;
            spec.args.push("--settings".into());
            spec.args.push(path.display().to_string());
        }
        LearnMode::Prompt => spec.args.extend(learn_prompt::args(
            agent.learn.prompt_flag.as_deref(),
            &text,
        )),
        LearnMode::Type | LearnMode::Off => {}
    }

    broker.node_stopped(&agent.id);
    let id = host
        .spawn(spec)
        .map_err(|e| format!("não foi possível iniciar \"{}\": {e}", agent.name))?;
    open.sessions.insert(agent.id.clone(), id);

    let rx = host.subscribe(id).map_err(|e| e.to_string())?;
    let (app, agent_id) = (app.clone(), agent.id.clone());
    let exit_broker = broker.clone();
    let exit_app = app.clone();
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
                        // Só a sessão atual conta: a saída de uma sessão antiga não mexe no estado.
                        if is_current(&exit_app, &node_id, id) {
                            exit_broker.node_stopped(&node_id);
                        }
                        emit(StatusEvent::Exited { code });
                        break;
                    }
                    SessionEvent::Output(_) => {}
                }
            }
        })
        .map_err(|e| e.to_string())?;
    broker.learn_begin(
        &agent.id,
        mode,
        Duration::from_secs(agent.learn.ready_timeout_s),
        text,
    );
    Ok(id)
}
