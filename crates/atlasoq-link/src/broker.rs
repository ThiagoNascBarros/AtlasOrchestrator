//! Broker: estado compartilhado das conexões entre agentes. O app implementa [`Host`]; o broker
//! decide quem pode falar com quem e entrega.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use atlasoq_core::ErrorKind;
use atlasoq_store::{ConnState, Connection, PauseReason};

use crate::ask::AskTable;
use crate::audit::{self, AuditEvent};
use crate::auth::TokenStore;
use crate::delivery::{Queues, SUBMIT_DELAY};
use crate::loop_guard::LoopGuard;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeInfo {
    pub id: String,
    pub name: String,
    pub role: String,
    pub is_note: bool,
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    /// `starting`, `working`, `idle` ou `exited`.
    pub status: &'static str,
    pub exited: bool,
    pub bracketed_paste: bool,
    pub last_user_input: Option<Instant>,
}

#[derive(Debug, Clone)]
pub enum LinkEvent {
    /// Uma mensagem passou pela conexão (a UI anima a linha).
    Message {
        conn_id: String,
        from_id: String,
        to_id: String,
        kind: &'static str,
    },
}

/// O que o broker precisa do app: workspace, sessões e saída para a UI.
pub trait Host: Send + Sync {
    fn nodes(&self) -> Vec<NodeInfo>;
    fn connections(&self) -> Vec<Connection>;
    fn session(&self, node_id: &str) -> Option<SessionInfo>;
    fn write(&self, node_id: &str, data: &[u8]) -> Result<(), String>;
    fn screen_text(&self, node_id: &str, lines: usize) -> Result<String, String>;
    fn set_connection_state(&self, conn_id: &str, state: ConnState, reason: Option<PauseReason>);
    fn logs_dir(&self) -> Option<PathBuf>;
    fn emit(&self, event: LinkEvent);
}

pub struct Broker {
    pub(crate) host: Arc<dyn Host>,
    pub(crate) tokens: TokenStore,
    pub(crate) guards: Mutex<HashMap<String, LoopGuard>>,
    pub(crate) asks: AskTable,
    pub(crate) queues: Queues,
    pub(crate) next_msg: AtomicU64,
    pub(crate) closing: AtomicBool,
    pub(crate) submit_delay: Duration,
    pub(crate) tick: Duration,
}

impl Broker {
    pub fn new(host: Arc<dyn Host>) -> Self {
        Self::with_timing(host, SUBMIT_DELAY, Duration::from_millis(500))
    }

    /// Como `new`, com atraso do Enter e intervalo de verificação do `ask` configuráveis (testes).
    pub fn with_timing(host: Arc<dyn Host>, submit_delay: Duration, tick: Duration) -> Self {
        Self {
            host,
            tokens: TokenStore::default(),
            guards: Mutex::new(HashMap::new()),
            asks: AskTable::default(),
            queues: Queues::default(),
            next_msg: AtomicU64::new(1),
            closing: AtomicBool::new(false),
            submit_delay,
            tick,
        }
    }

    /// Token novo para a sessão que está nascendo (invalida o anterior do mesmo agente).
    pub fn issue_token(&self, node_id: &str) -> String {
        self.tokens.issue(node_id)
    }

    pub fn revoke_token(&self, node_id: &str) {
        self.tokens.revoke(node_id);
    }

    /// A conexão foi pausada ou apagada: perguntas em espera nela terminam com o motivo.
    pub fn connection_stopped(&self, conn_id: &str, kind: ErrorKind, message: &str) {
        self.asks
            .cancel_where(|c, _, _| c == conn_id, kind, message);
    }

    /// Um agente foi encerrado ou apagado: perguntas que dependem dele terminam.
    pub fn node_stopped(&self, node_id: &str) {
        self.asks.cancel_where(
            |_, from, to| from == node_id || to == node_id,
            ErrorKind::TargetExited,
            "o agente foi encerrado",
        );
    }

    /// O usuário reativou a conexão: contadores de loop voltam a zero.
    pub fn reset_guard(&self, conn_id: &str) {
        self.guards.lock().unwrap().remove(conn_id);
    }

    /// O app está fechando: nenhuma operação nova, esperas terminam.
    pub fn shutdown(&self) {
        self.closing.store(true, Ordering::SeqCst);
        self.asks
            .cancel_all(ErrorKind::AppClosing, "o AtlasOQ está fechando");
    }

    pub fn audit(&self, ev: &AuditEvent) {
        if let Some(dir) = self.host.logs_dir() {
            audit::append(&dir, ev);
        }
    }

    pub fn history(&self, conn_id: &str, limit: usize) -> Vec<serde_json::Value> {
        self.host
            .logs_dir()
            .map(|d| audit::history(&d, conn_id, limit))
            .unwrap_or_default()
    }
}
