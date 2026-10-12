//! Broker de comunicação entre agentes: socket local, autenticação, ACL, entrega e auditoria.

pub mod acl;
pub mod ask;
pub mod audit;
pub mod auth;
pub mod broker;
pub mod delivery;
mod handlers;
pub mod loop_guard;
pub mod sanitize;
pub mod server;

pub use broker::{Broker, Host, LinkEvent, NodeInfo, SessionInfo};
pub use server::{socket_path, Server};
