//! Nomes visíveis de nós e ids internos (data-model.md).

use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Nome reservado: em `atlasoq note memory`, sempre a memória de quem executa.
pub const RESERVED_MEMORY: &str = "memory";

/// Tamanho máximo de um nome de nó (FR-007).
pub const MAX_NAME_LEN: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum NameError {
    #[error("o nome não pode ser vazio")]
    Empty,
    #[error("o nome pode ter no máximo {MAX_NAME_LEN} caracteres")]
    TooLong,
    #[error("o nome deve começar com uma letra minúscula")]
    InvalidStart,
    #[error("o caractere {0:?} não é permitido; use letras minúsculas, números e hífen")]
    InvalidChar(char),
    #[error("\"memory\" é um nome reservado")]
    Reserved,
}

/// Nome único de um agente ou nota no workspace: `^[a-z][a-z0-9-]{0,31}$`, exceto `memory`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct NodeName(String);

impl NodeName {
    pub fn parse(raw: &str) -> Result<Self, NameError> {
        let mut chars = raw.chars();
        let first = chars.next().ok_or(NameError::Empty)?;
        if raw.chars().count() > MAX_NAME_LEN {
            return Err(NameError::TooLong);
        }
        if !first.is_ascii_lowercase() {
            return Err(NameError::InvalidStart);
        }
        if let Some(bad) =
            chars.find(|c| !(c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-'))
        {
            return Err(NameError::InvalidChar(bad));
        }
        if raw == RESERVED_MEMORY {
            return Err(NameError::Reserved);
        }
        Ok(Self(raw.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for NodeName {
    type Error = NameError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<NodeName> for String {
    fn from(value: NodeName) -> Self {
        value.0
    }
}

impl fmt::Display for NodeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Prefixos dos ids internos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdKind {
    Workspace,
    Agent,
    Note,
    Connection,
}

impl IdKind {
    fn prefix(self) -> &'static str {
        match self {
            IdKind::Workspace => "ws",
            IdKind::Agent => "a",
            IdKind::Note => "n",
            IdKind::Connection => "c",
        }
    }
}

/// Gera um id `<prefixo>_<ulid>`.
pub fn new_id(kind: IdKind) -> String {
    format!("{}_{}", kind.prefix(), ulid::Ulid::generate())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aceita_nomes_validos() {
        for n in ["a", "coder", "planner-2", "x0-y1"] {
            assert!(NodeName::parse(n).is_ok(), "{n}");
        }
        assert!(NodeName::parse(&"a".repeat(32)).is_ok());
    }

    #[test]
    fn recusa_nomes_invalidos() {
        assert_eq!(NodeName::parse(""), Err(NameError::Empty));
        assert_eq!(NodeName::parse(&"a".repeat(33)), Err(NameError::TooLong));
        assert_eq!(NodeName::parse("2coder"), Err(NameError::InvalidStart));
        assert_eq!(NodeName::parse("-coder"), Err(NameError::InvalidStart));
        assert_eq!(NodeName::parse("Coder"), Err(NameError::InvalidStart));
        assert_eq!(NodeName::parse("co der"), Err(NameError::InvalidChar(' ')));
        assert_eq!(NodeName::parse("coder_1"), Err(NameError::InvalidChar('_')));
        assert_eq!(NodeName::parse("memory"), Err(NameError::Reserved));
    }

    #[test]
    fn ids_tem_prefixo() {
        assert!(new_id(IdKind::Agent).starts_with("a_"));
        assert!(new_id(IdKind::Note).starts_with("n_"));
        assert!(new_id(IdKind::Connection).starts_with("c_"));
        assert!(new_id(IdKind::Workspace).starts_with("ws_"));
        assert_ne!(new_id(IdKind::Agent), new_id(IdKind::Agent));
    }
}
