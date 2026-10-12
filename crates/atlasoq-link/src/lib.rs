//! Broker de comunicação entre agentes: socket local, autenticação, ACL, entrega e auditoria.

pub mod acl;
pub mod ask;
pub mod audit;
pub mod auth;
pub mod broker;
pub mod delivery;
mod handlers;
pub mod learn;
pub mod learn_content;
pub mod learn_hook;
pub mod learn_prompt;
mod learn_type;
pub mod loop_guard;
pub mod sanitize;
pub mod server;

pub use broker::{Broker, Host, LinkEvent, NodeInfo, SessionInfo};
pub use learn::LearnState;
pub use learn_content::LearnFiles;
pub use server::{socket_path, Server};
