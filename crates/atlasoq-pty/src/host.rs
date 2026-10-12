use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use thiserror::Error;

use crate::activity::Activity;

pub type SessionId = u64;

#[derive(Debug, Error)]
pub enum PtyError {
    #[error("sessão {0} não existe")]
    UnknownSession(SessionId),
    #[error("falha ao iniciar o processo: {0}")]
    Spawn(String),
    #[error("erro de E/S no terminal: {0}")]
    Io(String),
}

/// O que executar e com que tamanho de terminal.
#[derive(Debug, Clone)]
pub struct SessionSpec {
    pub command: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub env: Vec<(String, String)>,
    pub cols: u16,
    pub rows: u16,
    /// Linhas guardadas no scrollback do parser headless.
    pub scrollback: usize,
}

impl SessionSpec {
    pub fn new(command: impl Into<String>, cwd: impl Into<PathBuf>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            cwd: cwd.into(),
            env: Vec::new(),
            cols: 100,
            rows: 30,
            scrollback: 10_000,
        }
    }
}

/// Eventos publicados para quem assina uma sessão.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionEvent {
    /// Saída agrupada (no máximo a cada ~16 ms).
    Output(Vec<u8>),
    /// Mudança de status (trabalhando, ocioso).
    Status(Activity),
    /// O processo terminou; código de saída, se houver.
    Exited(Option<i32>),
}

/// Estado da tela suficiente para remontar um terminal na UI.
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub cols: u16,
    pub rows: u16,
    /// Bytes que reproduzem a tela visível (formatação incluída).
    pub formatted: Vec<u8>,
}

pub trait SessionHost: Send + Sync {
    fn spawn(&self, spec: SessionSpec) -> Result<SessionId, PtyError>;
    /// Escreve bytes vindos do app (mensagens, hooks). Não conta como digitação do usuário.
    fn write(&self, id: SessionId, data: &[u8]) -> Result<(), PtyError>;
    /// Escreve bytes digitados pelo usuário e registra a hora da última tecla.
    fn write_user(&self, id: SessionId, data: &[u8]) -> Result<(), PtyError>;
    fn resize(&self, id: SessionId, cols: u16, rows: u16) -> Result<(), PtyError>;
    /// Encerra o grupo de processos: SIGHUP, espera `grace`, depois SIGKILL.
    fn kill(&self, id: SessionId, grace: Duration) -> Result<(), PtyError>;
    fn subscribe(&self, id: SessionId) -> Result<Receiver<SessionEvent>, PtyError>;
    fn snapshot(&self, id: SessionId) -> Result<Snapshot, PtyError>;
    /// Últimas `lines` linhas de scrollback + tela, em texto puro.
    fn screen_text(&self, id: SessionId, lines: usize) -> Result<String, PtyError>;
    /// Se o programa no terminal ativou o modo bracketed paste (2004).
    fn bracketed_paste(&self, id: SessionId) -> Result<bool, PtyError>;
    fn activity(&self, id: SessionId) -> Result<Activity, PtyError>;
    /// Instante (monotônico) da última tecla do usuário, se houver.
    fn last_user_input(&self, id: SessionId) -> Result<Option<std::time::Instant>, PtyError>;
    fn pid(&self, id: SessionId) -> Result<Option<u32>, PtyError>;
    /// Encerra todas as sessões em paralelo (fechamento do app).
    fn shutdown_all(&self, grace: Duration);
}
