//! Conversa com o app pelo socket local: uma requisição, uma resposta.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

use atlasoq_core::{Response, PROTOCOL_VERSION};
use serde_json::{json, Value};

pub struct Env {
    pub socket: String,
    pub node: String,
    pub token: String,
}

/// Falha de transporte (app fechado, socket sumiu, resposta inválida).
pub struct Unreachable;

pub fn call(env: &Env, op: &str, args: Value) -> Result<Response, Unreachable> {
    let mut stream = UnixStream::connect(&env.socket).map_err(|_| Unreachable)?;
    let req = json!({
        "v": PROTOCOL_VERSION,
        "op": op,
        "node": env.node,
        "token": env.token,
        "args": args,
    });
    let mut line = serde_json::to_vec(&req).map_err(|_| Unreachable)?;
    line.push(b'\n');
    stream.write_all(&line).map_err(|_| Unreachable)?;
    let mut reply = String::new();
    BufReader::new(stream)
        .read_line(&mut reply)
        .map_err(|_| Unreachable)?;
    serde_json::from_str(reply.trim_end()).map_err(|_| Unreachable)
}
