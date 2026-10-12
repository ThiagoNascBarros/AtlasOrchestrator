//! Integração Tauri do AtlasOQ: comandos, canais de saída e ciclo de vida do app.

mod agents;
mod broker;
mod commands;
mod state;

use commands::{agent, connection, open, session, workspace};
use state::AppState;
use tauri::{Manager, RunEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .setup(|app| {
            broker::start(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            workspace::workspace_create,
            workspace::workspace_open,
            workspace::workspace_close,
            workspace::workspace_current,
            workspace::workspace_recent,
            workspace::workspace_set_viewport,
            workspace::node_set_rect,
            workspace::profiles_list,
            agent::agent_create,
            agent::agent_restart,
            agent::agent_stop,
            agent::agent_duplicate,
            agent::node_delete,
            connection::connection_create,
            connection::connection_set_direction,
            connection::connection_pause,
            connection::connection_resume,
            connection::connection_delete,
            connection::connection_history,
            session::session_attach,
            session::session_detach,
            session::session_write,
            session::session_resize,
            session::session_text,
            open::open_url,
            open::open_path,
        ])
        .build(tauri::generate_context!())
        .expect("falha ao iniciar o AtlasOQ");

    app.run(|handle, event| {
        if let RunEvent::Exit = event {
            handle.state::<AppState>().shutdown();
        }
    });
}
