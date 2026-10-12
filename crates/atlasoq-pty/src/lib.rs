//! Camada PTY do AtlasOQ: cria, alimenta, redimensiona e encerra processos em pseudo-terminais.
//!
//! O trait [`SessionHost`] isola a implementação: no MVP, [`LocalSessionHost`] roda dentro do
//! processo do app; depois do MVP, um host em daemon implementa a mesma interface.

mod activity;
mod host;
mod local;
mod reader;
mod screen;
mod shutdown;

pub use activity::{Activity, ActivityConfig};
pub use host::{PtyError, SessionEvent, SessionHost, SessionId, SessionSpec, Snapshot};
pub use local::LocalSessionHost;
