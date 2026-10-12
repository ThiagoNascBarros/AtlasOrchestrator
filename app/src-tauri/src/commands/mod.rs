pub mod agent;
pub mod connection;
pub mod open;
pub mod session;
pub mod workspace;

/// Evento emitido quando um agente ganha uma nova sessão (a UI reconecta o terminal).
pub const EVENT_AGENT_SESSION: &str = "agent-session";
