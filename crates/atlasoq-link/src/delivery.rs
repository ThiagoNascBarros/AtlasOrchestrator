//! Entrega de texto no terminal do destino: cabeçalho, bracketed paste e Enter com atraso.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::broker::Host;

/// Espera entre o texto colado e o Enter, para o programa do terminal processar a colagem.
pub const SUBMIT_DELAY: Duration = Duration::from_millis(150);

pub fn format_message(from: &str, msg_id: u64, text: &str) -> String {
    format!("[atlasoq de:{from} msg:{msg_id}] {text}")
}

pub fn format_ask(from: &str, ask_id: u64, text: &str) -> String {
    format!("[atlasoq ask de:{from} id:{ask_id}] {text} — responda com: atlasoq reply \"...\"")
}

/// Bytes do texto como o terminal vai recebê-lo (sem o Enter final).
pub fn encode(payload: &str, bracketed: bool) -> Vec<u8> {
    if bracketed {
        let mut out = b"\x1b[200~".to_vec();
        out.extend_from_slice(payload.as_bytes());
        out.extend_from_slice(b"\x1b[201~");
        out
    } else {
        // Sem bracketed paste, uma quebra de linha enviaria o texto pela metade.
        payload.replace('\n', " ").into_bytes()
    }
}

/// Fila serial por destino: duas mensagens para o mesmo agente nunca se misturam.
#[derive(Default)]
pub struct Queues {
    by_node: Mutex<HashMap<String, Arc<Mutex<()>>>>,
}

impl Queues {
    pub fn lock_for(&self, node_id: &str) -> Arc<Mutex<()>> {
        self.by_node
            .lock()
            .unwrap()
            .entry(node_id.to_owned())
            .or_default()
            .clone()
    }
}

pub fn deliver(
    host: &dyn Host,
    queues: &Queues,
    node_id: &str,
    payload: &str,
    bracketed: bool,
    delay: Duration,
) -> Result<(), String> {
    let queue = queues.lock_for(node_id);
    let _turn = queue.lock().unwrap();
    host.write(node_id, &encode(payload, bracketed))?;
    std::thread::sleep(delay);
    host.write(node_id, b"\r")
}
