//! Estado do app: host de sessões e o workspace aberto.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use atlasoq_pty::{LocalSessionHost, SessionHost, SessionId};
use atlasoq_store::autosave::Autosaver;
use atlasoq_store::{Workspace, WorkspaceDir};

/// Tempo entre SIGHUP e SIGKILL ao encerrar agentes.
pub const KILL_GRACE: Duration = Duration::from_secs(3);

pub struct OpenWorkspace {
    pub dir: Arc<WorkspaceDir>,
    pub ws: Workspace,
    pub autosave: Autosaver,
    /// Sessão atual de cada agente (id do nó → sessão).
    pub sessions: HashMap<String, SessionId>,
}

impl OpenWorkspace {
    pub fn new(dir: WorkspaceDir, ws: Workspace) -> Self {
        let dir = Arc::new(dir);
        let autosave = {
            let dir = dir.clone();
            Autosaver::new(Duration::from_millis(500), move |ws| {
                if let Err(e) = dir.save(ws) {
                    eprintln!("atlasoq: falha ao salvar o workspace: {e}");
                }
            })
        };
        Self {
            dir,
            ws,
            autosave,
            sessions: HashMap::new(),
        }
    }

    /// Agenda o salvamento do estado atual.
    pub fn touch(&self) {
        self.autosave.schedule(self.ws.clone());
    }

    pub fn root(&self) -> PathBuf {
        self.dir.root().to_path_buf()
    }
}

pub struct AppState {
    pub host: Arc<LocalSessionHost>,
    open: Mutex<Option<OpenWorkspace>>,
    /// Terminais montados na UI: id da conexão → sinal de parada.
    attachments: Mutex<HashMap<u64, Arc<AtomicBool>>>,
    next_attachment: AtomicU64,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            host: Arc::new(LocalSessionHost::default()),
            open: Mutex::new(None),
            attachments: Mutex::new(HashMap::new()),
            next_attachment: AtomicU64::new(1),
        }
    }
}

impl AppState {
    /// Registra um terminal montado e devolve (id, sinal de parada).
    pub fn new_attachment(&self) -> (u64, Arc<AtomicBool>) {
        let id = self.next_attachment.fetch_add(1, Ordering::SeqCst);
        let stop = Arc::new(AtomicBool::new(false));
        self.attachments.lock().unwrap().insert(id, stop.clone());
        (id, stop)
    }

    pub fn end_attachment(&self, id: u64) {
        if let Some(stop) = self.attachments.lock().unwrap().remove(&id) {
            stop.store(true, Ordering::SeqCst);
        }
    }

    pub fn workspace(&self) -> MutexGuard<'_, Option<OpenWorkspace>> {
        self.open.lock().unwrap()
    }

    /// Executa `f` com o workspace aberto, ou devolve erro se não houver.
    pub fn with_ws<T>(
        &self,
        f: impl FnOnce(&mut OpenWorkspace) -> Result<T, String>,
    ) -> Result<T, String> {
        let mut guard = self.workspace();
        let open = guard
            .as_mut()
            .ok_or_else(|| "nenhum workspace aberto".to_owned())?;
        f(open)
    }

    /// Fecha o workspace atual: grava pendências e encerra os agentes.
    pub fn close(&self) {
        let old = self.workspace().take();
        if let Some(open) = old {
            open.autosave.flush();
            let handles: Vec<_> = open
                .sessions
                .values()
                .map(|id| {
                    let (host, id) = (self.host.clone(), *id);
                    std::thread::spawn(move || host.kill(id, KILL_GRACE))
                })
                .collect();
            for h in handles {
                let _ = h.join();
            }
        }
    }

    /// Fechamento do app.
    pub fn shutdown(&self) {
        if let Some(open) = self.workspace().as_ref() {
            open.autosave.flush();
        }
        self.host.shutdown_all(KILL_GRACE);
    }
}
