//! Códigos de saída da CLI e identificadores de erro do protocolo (contracts/cli.md,
//! contracts/socket-protocol.md). Ambos são estáveis: mudar exige emenda da constituição.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "i32", try_from = "i32")]
pub enum ExitCode {
    Ok = 0,
    Usage = 2,
    NoPermission = 3,
    Unavailable = 4,
    Timeout = 5,
    OutsideApp = 6,
    Refused = 7,
}

impl From<ExitCode> for i32 {
    fn from(value: ExitCode) -> Self {
        value as i32
    }
}

impl TryFrom<i32> for ExitCode {
    type Error = String;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => ExitCode::Ok,
            2 => ExitCode::Usage,
            3 => ExitCode::NoPermission,
            4 => ExitCode::Unavailable,
            5 => ExitCode::Timeout,
            6 => ExitCode::OutsideApp,
            7 => ExitCode::Refused,
            other => return Err(format!("código de saída desconhecido: {other}")),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    BadRequest,
    UnknownTarget,
    NoConnection,
    WrongDirection,
    ConnectionPaused,
    TargetExited,
    TooLarge,
    SecretDetected,
    MissingPrefix,
    NoPendingAsk,
    AmbiguousReply,
    AskExpired,
    Timeout,
    Unauthorized,
    AppClosing,
    UnsupportedVersion,
}

impl ErrorKind {
    pub fn exit_code(self) -> ExitCode {
        match self {
            ErrorKind::BadRequest | ErrorKind::AmbiguousReply | ErrorKind::UnsupportedVersion => {
                ExitCode::Usage
            }
            ErrorKind::NoConnection
            | ErrorKind::WrongDirection
            | ErrorKind::ConnectionPaused
            | ErrorKind::NoPendingAsk
            | ErrorKind::AskExpired => ExitCode::NoPermission,
            ErrorKind::UnknownTarget | ErrorKind::TargetExited | ErrorKind::AppClosing => {
                ExitCode::Unavailable
            }
            ErrorKind::Timeout => ExitCode::Timeout,
            ErrorKind::Unauthorized => ExitCode::OutsideApp,
            ErrorKind::TooLarge | ErrorKind::SecretDetected | ErrorKind::MissingPrefix => {
                ExitCode::Refused
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erros_serializam_em_snake_case() {
        let s = serde_json::to_string(&ErrorKind::NoPendingAsk).unwrap();
        assert_eq!(s, "\"no_pending_ask\"");
    }

    #[test]
    fn codigos_de_saida_estaveis() {
        assert_eq!(i32::from(ErrorKind::NoConnection.exit_code()), 3);
        assert_eq!(i32::from(ErrorKind::Timeout.exit_code()), 5);
        assert_eq!(i32::from(ErrorKind::SecretDetected.exit_code()), 7);
        assert_eq!(i32::from(ErrorKind::TargetExited.exit_code()), 4);
        assert_eq!(i32::from(ErrorKind::AmbiguousReply.exit_code()), 2);
    }
}
