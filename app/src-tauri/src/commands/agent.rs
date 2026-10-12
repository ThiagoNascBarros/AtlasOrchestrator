//! Comandos de agentes: criar, reiniciar, duplicar, encerrar e apagar.

use std::collections::BTreeMap;

use atlasoq_core::{new_id, IdKind, NodeName};
use atlasoq_pty::{Activity, SessionHost};
use atlasoq_store::profiles::{self};
use atlasoq_store::{paths, Agent, Node, Rect, Terminal};
use serde::Deserialize;
use tauri::{AppHandle, Emitter, State};

use super::EVENT_AGENT_SESSION;
use crate::agents;
use crate::state::{AppState, OpenWorkspace, KILL_GRACE};

pub const DEFAULT_W: f64 = 720.0;
pub const DEFAULT_H: f64 = 460.0;

#[derive(Deserialize)]
pub struct NewAgent {
    pub profile: String,
    pub name: String,
    #[serde(default)]
    pub role: String,
    #[serde(default = "dot")]
    pub cwd: String,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub env_secret_keys: Vec<String>,
    pub x: f64,
    pub y: f64,
}

fn dot() -> String {
    ".".into()
}

/// Sugere um nome livre a partir de `base` (ex.: `coder-2`).
pub fn suggest_name(open: &OpenWorkspace, base: &str) -> String {
    let base: String = base.chars().take(29).collect();
    (2..1000)
        .map(|i| format!("{base}-{i}"))
        .find(|n| open.ws.node_by_name(n).is_none())
        .unwrap_or_else(|| format!("{base}-x"))
}

fn check_name(open: &OpenWorkspace, raw: &str) -> Result<NodeName, String> {
    let name = NodeName::parse(raw).map_err(|e| format!("nome inválido: {e}"))?;
    if open.ws.node_by_name(raw).is_some() {
        return Err(format!(
            "já existe um nó chamado \"{raw}\"; sugestão: \"{}\"",
            suggest_name(open, raw)
        ));
    }
    Ok(name)
}

fn find_agent(open: &OpenWorkspace, id: &str) -> Result<Agent, String> {
    match open.ws.node(id) {
        Some(Node::Agent(a)) => Ok(a.clone()),
        _ => Err("agente inexistente".into()),
    }
}

fn notify_session(app: &AppHandle, agent_id: &str) {
    let _ = app.emit(EVENT_AGENT_SESSION, agent_id);
}

#[tauri::command]
pub fn agent_create(
    app: AppHandle,
    state: State<'_, AppState>,
    req: NewAgent,
) -> Result<Agent, String> {
    let ws_profiles = state.workspace().as_ref().map(|o| o.dir.profiles_dir());
    let all = profiles::resolve(
        &paths::config_dir().join("profiles"),
        ws_profiles.as_deref(),
    )
    .map_err(|e| e.to_string())?;
    let profile = all
        .into_iter()
        .find(|p| p.name == req.profile)
        .ok_or("perfil inexistente")?;

    let agent = state.with_ws(|open| {
        let name = check_name(open, &req.name)?;
        let mut env = profile.env.clone();
        env.extend(req.env.clone());
        for k in &req.env_secret_keys {
            env.remove(k);
        }
        let cwd = match req.cwd.trim() {
            "" => ".".to_owned(),
            c => c.to_owned(),
        };
        let agent = Agent {
            id: new_id(IdKind::Agent),
            name,
            role: req.role.trim().to_owned(),
            profile: profile.name.clone(),
            rect: Rect {
                x: req.x,
                y: req.y,
                w: DEFAULT_W,
                h: DEFAULT_H,
            },
            terminal: Terminal {
                command: profile.command.clone(),
                args: profile.args.clone(),
                cwd,
                env,
                env_secret_keys: req.env_secret_keys.clone(),
                cols: 100,
                rows: 30,
            },
            learn: profile.learn.clone(),
        };
        open.ws.nodes.push(Node::Agent(agent.clone()));
        if let Some(problem) = atlasoq_store::validate::validate(&open.ws)
            .into_iter()
            .next()
        {
            open.ws.nodes.pop();
            return Err(problem);
        }
        agents::launch(&app, &state.host, open, &agent)?;
        open.touch();
        Ok(agent)
    })?;
    notify_session(&app, &agent.id);
    Ok(agent)
}

#[tauri::command]
pub fn agent_restart(app: AppHandle, state: State<'_, AppState>, id: String) -> Result<(), String> {
    let old = state.with_ws(|open| Ok(open.sessions.get(&id).copied()))?;
    if let Some(sid) = old {
        if !matches!(state.host.activity(sid), Ok(Activity::Exited(_))) {
            state
                .host
                .kill(sid, KILL_GRACE)
                .map_err(|e| e.to_string())?;
        }
    }
    state.with_ws(|open| {
        let agent = find_agent(open, &id)?;
        agents::launch(&app, &state.host, open, &agent).map(|_| ())
    })?;
    notify_session(&app, &id);
    Ok(())
}

#[tauri::command]
pub fn agent_stop(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let sid = state.with_ws(|open| {
        open.sessions
            .get(&id)
            .copied()
            .ok_or_else(|| "agente sem sessão".into())
    })?;
    state.host.kill(sid, KILL_GRACE).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_duplicate(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<Agent, String> {
    let agent = state.with_ws(|open| {
        let src = find_agent(open, &id)?;
        let name =
            NodeName::parse(&suggest_name(open, src.name.as_str())).map_err(|e| e.to_string())?;
        let agent = Agent {
            id: new_id(IdKind::Agent),
            name,
            rect: Rect {
                x: src.rect.x + 40.0,
                y: src.rect.y + 40.0,
                ..src.rect
            },
            ..src
        };
        open.ws.nodes.push(Node::Agent(agent.clone()));
        agents::launch(&app, &state.host, open, &agent)?;
        open.touch();
        Ok(agent)
    })?;
    notify_session(&app, &agent.id);
    Ok(agent)
}

/// Apaga um nó (agente ou nota) e as conexões dele. Agentes são encerrados.
#[tauri::command]
pub fn node_delete(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let sid = state.with_ws(|open| {
        let before = open.ws.nodes.len();
        open.ws.nodes.retain(|n| n.id() != id);
        if open.ws.nodes.len() == before {
            return Err("nó inexistente".into());
        }
        open.ws.connections.retain(|c| c.from != id && c.to != id);
        open.touch();
        Ok(open.sessions.remove(&id))
    })?;
    if let Some(sid) = sid {
        let host = state.host.clone();
        std::thread::spawn(move || host.kill(sid, KILL_GRACE));
    }
    Ok(())
}
