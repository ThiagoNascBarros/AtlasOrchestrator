//! Comandos de workspace: criar, abrir, recentes e alterações de layout.

use std::path::PathBuf;

use atlasoq_store::profiles::{self, Profile};
use atlasoq_store::recent::{self, RecentEntry};
use atlasoq_store::{paths, Node, Rect, Viewport, Workspace, WorkspaceDir};
use serde::Serialize;
use tauri::{AppHandle, State};

use crate::agents;
use crate::state::{AppState, OpenWorkspace};

#[derive(Serialize)]
pub struct WorkspaceView {
    pub root: PathBuf,
    pub workspace: Workspace,
}

fn recent_file() -> PathBuf {
    paths::state_dir().join("recent.json")
}

fn view(open: &OpenWorkspace) -> WorkspaceView {
    WorkspaceView {
        root: open.root(),
        workspace: open.ws.clone(),
    }
}

fn activate(
    app: &AppHandle,
    state: &AppState,
    dir: WorkspaceDir,
    ws: Workspace,
) -> Result<WorkspaceView, String> {
    let _ = recent::touch(&recent_file(), dir.root(), &ws.name);
    let mut open = OpenWorkspace::new(dir, ws);
    let agents: Vec<_> = open.ws.agents().cloned().collect();
    for agent in &agents {
        if let Err(e) = agents::launch(app, &state.host, &mut open, agent) {
            eprintln!("atlasoq: {e}");
        }
    }
    let v = view(&open);
    *state.workspace() = Some(open);
    Ok(v)
}

#[tauri::command]
pub fn workspace_create(
    app: AppHandle,
    state: State<'_, AppState>,
    root: String,
    name: String,
) -> Result<WorkspaceView, String> {
    state.close();
    let (dir, ws) =
        WorkspaceDir::create(&PathBuf::from(root), name.trim()).map_err(|e| e.to_string())?;
    activate(&app, &state, dir, ws)
}

#[tauri::command]
pub fn workspace_open(
    app: AppHandle,
    state: State<'_, AppState>,
    root: String,
) -> Result<WorkspaceView, String> {
    state.close();
    let (dir, ws) = WorkspaceDir::open(&PathBuf::from(root)).map_err(|e| e.to_string())?;
    activate(&app, &state, dir, ws)
}

#[tauri::command]
pub fn workspace_close(state: State<'_, AppState>) {
    state.close();
}

#[tauri::command]
pub fn workspace_current(state: State<'_, AppState>) -> Option<WorkspaceView> {
    state.workspace().as_ref().map(view)
}

#[tauri::command]
pub fn workspace_recent() -> Vec<RecentEntry> {
    recent::load(&recent_file())
        .into_iter()
        .filter(|e| e.root.exists())
        .collect()
}

#[tauri::command]
pub fn workspace_set_viewport(
    state: State<'_, AppState>,
    viewport: Viewport,
) -> Result<(), String> {
    state.with_ws(|open| {
        open.ws.viewport = viewport;
        open.touch();
        Ok(())
    })
}

#[tauri::command]
pub fn node_set_rect(state: State<'_, AppState>, id: String, rect: Rect) -> Result<(), String> {
    state.with_ws(|open| {
        let node = open
            .ws
            .nodes
            .iter_mut()
            .find(|n| n.id() == id)
            .ok_or("nó inexistente")?;
        match node {
            Node::Agent(a) => a.rect = rect,
            Node::Note(n) => n.rect = rect,
        }
        open.touch();
        Ok(())
    })
}

#[tauri::command]
pub fn profiles_list(state: State<'_, AppState>) -> Result<Vec<Profile>, String> {
    let ws_dir = state.workspace().as_ref().map(|o| o.dir.profiles_dir());
    profiles::resolve(&paths::config_dir().join("profiles"), ws_dir.as_deref())
        .map_err(|e| e.to_string())
}
