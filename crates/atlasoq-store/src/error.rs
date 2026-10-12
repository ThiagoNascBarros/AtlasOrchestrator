use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("erro de E/S em {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("arquivo {path} inválido: {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("workspace inválido: {0}")]
    Invalid(String),
    #[error("sem permissão de escrita em {0}")]
    NotWritable(PathBuf),
    #[error("este workspace já está aberto em outra janela do AtlasOQ")]
    AlreadyOpen,
    #[error("{0} não contém um workspace do AtlasOQ")]
    NotAWorkspace(PathBuf),
    #[error("já existe um workspace em {0}")]
    AlreadyExists(PathBuf),
}

impl StoreError {
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}

pub type Result<T> = std::result::Result<T, StoreError>;
