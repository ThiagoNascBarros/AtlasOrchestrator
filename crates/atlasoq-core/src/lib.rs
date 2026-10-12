//! Tipos compartilhados do AtlasOQ: nomes, ids, protocolo CLI ↔ app, erros e textos.

pub mod errors;
pub mod i18n;
pub mod ids;
pub mod protocol;

pub use protocol::{Op, Request, Response, PROTOCOL_VERSION};

pub use errors::{ErrorKind, ExitCode};
pub use ids::{new_id, IdKind, NameError, NodeName, RESERVED_MEMORY};
