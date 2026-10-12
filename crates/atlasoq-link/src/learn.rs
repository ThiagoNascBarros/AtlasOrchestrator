//! Estado do aprendizado de cada agente (`pending`, `done`, `failed`, `off`) e os disparos por
//! modo. Cada início de sessão recebe uma geração; respostas de sessões antigas são ignoradas.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use atlasoq_store::LearnMode;

use crate::broker::{Broker, LinkEvent};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LearnState {
    Pending,
    Done,
    Failed(String),
    Off,
}

impl LearnState {
    pub fn status(&self) -> &'static str {
        match self {
            LearnState::Pending => "pending",
            LearnState::Done => "done",
            LearnState::Failed(_) => "failed",
            LearnState::Off => "off",
        }
    }

    pub fn reason(&self) -> Option<String> {
        match self {
            LearnState::Failed(r) => Some(r.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
struct Entry {
    gen: u64,
    state: LearnState,
}

#[derive(Default)]
pub(crate) struct LearnTable {
    entries: HashMap<String, Entry>,
    next_gen: u64,
}

impl Broker {
    /// Estado atual do aprendizado do agente, se ele já teve uma sessão.
    pub fn learn_state(&self, node_id: &str) -> Option<LearnState> {
        self.learn
            .lock()
            .unwrap()
            .entries
            .get(node_id)
            .map(|e| e.state.clone())
    }

    /// Começa o aprendizado de uma sessão nova; `text` é o conteúdo entregue no modo `type`.
    /// Devolve a geração da sessão.
    pub fn learn_begin(
        self: &Arc<Self>,
        node_id: &str,
        mode: LearnMode,
        timeout: Duration,
        text: String,
    ) -> u64 {
        let (gen, state) = {
            let mut t = self.learn.lock().unwrap();
            t.next_gen += 1;
            let gen = t.next_gen;
            let state = match mode {
                LearnMode::Off => LearnState::Off,
                // `prompt`: o texto já foi passado ao processo, que iniciou sem erro.
                LearnMode::Prompt => LearnState::Done,
                LearnMode::Hook | LearnMode::Type => LearnState::Pending,
            };
            t.entries.insert(
                node_id.to_owned(),
                Entry {
                    gen,
                    state: state.clone(),
                },
            );
            (gen, state)
        };
        self.announce(node_id, &state);

        match mode {
            LearnMode::Hook => {
                let broker = self.clone();
                let node = node_id.to_owned();
                std::thread::spawn(move || {
                    std::thread::sleep(timeout);
                    broker.set_state(
                        &node,
                        Some(gen),
                        LearnState::Failed("o agente não rodou o hook de início a tempo".into()),
                    );
                });
            }
            LearnMode::Type => {
                let broker = self.clone();
                let node = node_id.to_owned();
                std::thread::spawn(move || broker.run_type(&node, gen, timeout, &text));
            }
            LearnMode::Prompt | LearnMode::Off => {}
        }
        gen
    }

    /// `atlasoq learn --hook` chegou: o aprendizado está concluído.
    pub fn learn_done(&self, node_id: &str) {
        self.set_state(node_id, None, LearnState::Done);
    }

    /// Só sai de `pending`. Com `gen`, só vale para a sessão daquela geração.
    pub(crate) fn set_state(&self, node_id: &str, gen: Option<u64>, state: LearnState) -> bool {
        let changed = {
            let mut t = self.learn.lock().unwrap();
            match t.entries.get_mut(node_id) {
                Some(e) if gen.is_none_or(|g| g == e.gen) && e.state == LearnState::Pending => {
                    e.state = state.clone();
                    true
                }
                _ => false,
            }
        };
        if changed {
            self.announce(node_id, &state);
        }
        changed
    }

    pub(crate) fn is_pending(&self, node_id: &str, gen: u64) -> bool {
        self.learn
            .lock()
            .unwrap()
            .entries
            .get(node_id)
            .is_some_and(|e| e.gen == gen && e.state == LearnState::Pending)
    }

    fn announce(&self, node_id: &str, state: &LearnState) {
        self.host.emit(LinkEvent::Learn {
            node_id: node_id.to_owned(),
            status: state.status(),
            reason: state.reason(),
        });
    }
}
