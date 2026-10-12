//! Protocolo CLI ↔ app, versão 1 (contracts/socket-protocol.md): um objeto JSON por linha.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::errors::ErrorKind;

pub const PROTOCOL_VERSION: u32 = 1;

/// Tamanho máximo de uma linha do protocolo.
pub const MAX_LINE_BYTES: usize = 64 * 1024;

/// Variáveis de ambiente que o app injeta em cada agente.
pub const ENV_SOCKET: &str = "ATLASOQ_SOCKET";
pub const ENV_NODE_ID: &str = "ATLASOQ_NODE_ID";
pub const ENV_NODE_TOKEN: &str = "ATLASOQ_NODE_TOKEN";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Request {
    pub v: u32,
    pub node: String,
    pub token: String,
    #[serde(flatten)]
    pub op: Op,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", content = "args", rename_all = "snake_case")]
pub enum Op {
    Learn {
        #[serde(default)]
        hook: bool,
    },
    List {},
    Send {
        to: String,
        text: String,
    },
    Read {
        from: String,
        #[serde(default = "default_lines")]
        lines: u32,
    },
    Ask {
        to: String,
        text: String,
        #[serde(default = "default_timeout")]
        timeout_s: u64,
    },
    Reply {
        #[serde(default)]
        to: Option<String>,
        text: String,
    },
    NoteRead {
        note: String,
    },
    NoteAppend {
        note: String,
        text: String,
    },
}

pub const DEFAULT_READ_LINES: u32 = 60;
pub const MAX_READ_LINES: u32 = 2000;
pub const DEFAULT_ASK_TIMEOUT_S: u64 = 600;

fn default_lines() -> u32 {
    DEFAULT_READ_LINES
}

fn default_timeout() -> u64 {
    DEFAULT_ASK_TIMEOUT_S
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response {
    pub v: u32,
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

impl Response {
    pub fn ok(data: Value) -> Self {
        Self {
            v: PROTOCOL_VERSION,
            ok: true,
            data: Some(data),
            code: None,
            error: None,
            message: None,
            hint: None,
        }
    }

    pub fn err(kind: ErrorKind, message: impl Into<String>, hint: impl Into<String>) -> Self {
        let hint = hint.into();
        Self {
            v: PROTOCOL_VERSION,
            ok: false,
            data: None,
            code: Some(kind.exit_code().into()),
            error: Some(kind),
            message: Some(message.into()),
            hint: (!hint.is_empty()).then_some(hint),
        }
    }

    /// Erro com dados extras (ex.: a tela recente quando o `ask` esgota o tempo).
    pub fn err_with_data(
        kind: ErrorKind,
        message: impl Into<String>,
        hint: impl Into<String>,
        data: Value,
    ) -> Self {
        Self {
            data: Some(data),
            ..Self::err(kind, message, hint)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn requisicao_no_formato_do_contrato() {
        let raw = json!({"v": 1, "op": "send", "node": "a_1", "token": "t", "args": {"to": "revisor", "text": "oi"}});
        let req: Request = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(
            req.op,
            Op::Send {
                to: "revisor".into(),
                text: "oi".into()
            }
        );
        assert_eq!(serde_json::to_value(&req).unwrap(), raw);
    }

    #[test]
    fn padroes_de_read_e_ask() {
        let req: Request = serde_json::from_value(
            json!({"v":1,"op":"read","node":"a","token":"t","args":{"from":"coder"}}),
        )
        .unwrap();
        assert_eq!(
            req.op,
            Op::Read {
                from: "coder".into(),
                lines: 60
            }
        );
        let req: Request = serde_json::from_value(
            json!({"v":1,"op":"ask","node":"a","token":"t","args":{"to":"c","text":"x"}}),
        )
        .unwrap();
        assert_eq!(
            req.op,
            Op::Ask {
                to: "c".into(),
                text: "x".into(),
                timeout_s: 600
            }
        );
    }

    #[test]
    fn list_sem_argumentos() {
        let req: Request =
            serde_json::from_value(json!({"v":1,"op":"list","node":"a","token":"t","args":{}}))
                .unwrap();
        assert_eq!(req.op, Op::List {});
    }

    #[test]
    fn resposta_de_erro_tem_codigo() {
        let r = Response::err(ErrorKind::NoConnection, "sem conexão", "peça ao usuário");
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["code"], 3);
        assert_eq!(v["error"], "no_connection");
        assert_eq!(v["ok"], false);
    }
}
