//! Persistência do workspace em `<raiz>/.atlasoq/` (contracts/workspace-format.md): formato,
//! validação, escrita atômica, lock, salvamento automático e perfis.

pub mod atomic;
pub mod autosave;
pub mod error;
pub mod lock;
pub mod model;
pub mod paths;
pub mod profiles;
pub mod validate;
pub mod workspace;

pub use error::StoreError;
pub use model::*;
pub use workspace::WorkspaceDir;
