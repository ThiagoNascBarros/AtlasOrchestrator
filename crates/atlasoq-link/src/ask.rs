//! Perguntas pendentes (`ask` ↔ `reply`): `waiting` → `answered` | `timed_out` | `cancelled`.

use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};
use std::time::{Duration, Instant};

use atlasoq_core::ErrorKind;

const EXPIRED_KEPT: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
enum State {
    Waiting,
    Answered(String),
    Cancelled(ErrorKind, String),
}

#[derive(Debug, Clone)]
struct Pending {
    id: u64,
    from_id: String,
    from_name: String,
    to_id: String,
    conn_id: String,
    state: State,
}

/// Pergunta que ainda espera resposta, vista de quem deve responder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingInfo {
    pub id: u64,
    pub from_id: String,
    pub from_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expired {
    pub id: u64,
    pub from_name: String,
    pub to_id: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Answered(String),
    TimedOut,
    Cancelled(ErrorKind, String),
}

#[derive(Default)]
struct Inner {
    asks: Vec<Pending>,
    expired: VecDeque<Expired>,
    next: u64,
}

#[derive(Default)]
pub struct AskTable {
    inner: Mutex<Inner>,
    cv: Condvar,
}

impl AskTable {
    pub fn open(&self, from_id: &str, from_name: &str, to_id: &str, conn_id: &str) -> u64 {
        let mut g = self.inner.lock().unwrap();
        g.next += 1;
        let id = g.next;
        g.asks.push(Pending {
            id,
            from_id: from_id.to_owned(),
            from_name: from_name.to_owned(),
            to_id: to_id.to_owned(),
            conn_id: conn_id.to_owned(),
            state: State::Waiting,
        });
        id
    }

    /// Descarta uma pergunta que não chegou a ser entregue.
    pub fn remove(&self, id: u64) {
        self.inner.lock().unwrap().asks.retain(|a| a.id != id);
    }

    /// Perguntas que `to_id` ainda precisa responder.
    pub fn pending_for(&self, to_id: &str) -> Vec<PendingInfo> {
        self.inner
            .lock()
            .unwrap()
            .asks
            .iter()
            .filter(|a| a.to_id == to_id && a.state == State::Waiting)
            .map(|a| PendingInfo {
                id: a.id,
                from_id: a.from_id.clone(),
                from_name: a.from_name.clone(),
            })
            .collect()
    }

    /// Entrega a resposta ao `ask` que espera. Devolve a conexão da pergunta.
    pub fn answer(&self, id: u64, text: &str) -> Option<String> {
        let mut g = self.inner.lock().unwrap();
        let ask = g
            .asks
            .iter_mut()
            .find(|a| a.id == id && a.state == State::Waiting)?;
        ask.state = State::Answered(text.to_owned());
        let conn = ask.conn_id.clone();
        self.cv.notify_all();
        Some(conn)
    }

    /// Cancela as perguntas que satisfazem `pred` (pausa, remoção, encerramento).
    pub fn cancel_where(&self, pred: impl Fn(&str, &str, &str) -> bool, kind: ErrorKind, msg: &str) {
        let mut g = self.inner.lock().unwrap();
        for a in g.asks.iter_mut().filter(|a| a.state == State::Waiting) {
            if pred(&a.conn_id, &a.from_id, &a.to_id) {
                a.state = State::Cancelled(kind, msg.to_owned());
            }
        }
        self.cv.notify_all();
    }

    pub fn cancel_all(&self, kind: ErrorKind, msg: &str) {
        self.cancel_where(|_, _, _| true, kind, msg);
    }

    /// A última pergunta expirada endereçada a `to_id`, se houver.
    pub fn last_expired_for(&self, to_id: &str) -> Option<Expired> {
        self.inner
            .lock()
            .unwrap()
            .expired
            .iter()
            .rev()
            .find(|e| e.to_id == to_id)
            .cloned()
    }

    /// Espera o resultado. `check` é consultado a cada `tick` e pode cancelar a espera (conexão
    /// pausada, destino encerrado).
    pub fn wait(
        &self,
        id: u64,
        deadline: Instant,
        tick: Duration,
        mut check: impl FnMut() -> Option<(ErrorKind, String)>,
    ) -> Outcome {
        let mut g = self.inner.lock().unwrap();
        loop {
            let Some(pos) = g.asks.iter().position(|a| a.id == id) else {
                return Outcome::Cancelled(ErrorKind::BadRequest, "pergunta desconhecida".into());
            };
            match g.asks[pos].state.clone() {
                State::Answered(text) => {
                    g.asks.remove(pos);
                    return Outcome::Answered(text);
                }
                State::Cancelled(kind, msg) => {
                    g.asks.remove(pos);
                    return Outcome::Cancelled(kind, msg);
                }
                State::Waiting => {}
            }
            let now = Instant::now();
            if now >= deadline {
                let ask = g.asks.remove(pos);
                g.expired.push_back(Expired {
                    id: ask.id,
                    from_name: ask.from_name,
                    to_id: ask.to_id,
                });
                if g.expired.len() > EXPIRED_KEPT {
                    g.expired.pop_front();
                }
                return Outcome::TimedOut;
            }
            if let Some((kind, msg)) = check() {
                g.asks.remove(pos);
                return Outcome::Cancelled(kind, msg);
            }
            let wait = tick.min(deadline - now);
            g = self.cv.wait_timeout(g, wait).unwrap().0;
        }
    }
}
