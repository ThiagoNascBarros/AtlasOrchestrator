//! Modo `type`: espera o agente ficar ocioso, não entrega enquanto o usuário digita e então envia
//! o texto como mensagem do próprio app, pelo caminho normal de entrega.

use std::time::{Duration, Instant};

use crate::audit::AuditEvent;
use crate::broker::Broker;
use crate::delivery;
use crate::learn::LearnState;
use crate::sanitize::sanitize;

/// Intervalo entre as verificações do estado do agente.
const POLL: Duration = Duration::from_millis(200);
/// Tempo sem digitação do usuário antes de entregar.
pub const USER_QUIET: Duration = Duration::from_secs(2);
/// Nome do remetente nas mensagens do próprio app.
pub const APP_SENDER: &str = "app";

impl Broker {
    pub(crate) fn run_type(&self, node: &str, gen: u64, timeout: Duration, text: &str) {
        let deadline = Instant::now() + timeout;
        let fail = |reason: &str| {
            self.set_state(node, Some(gen), LearnState::Failed(reason.into()));
        };
        let gone = "o agente encerrou antes de ficar pronto";

        // 1. Espera o agente ficar ocioso (pronto para receber texto).
        loop {
            if !self.is_pending(node, gen) {
                return;
            }
            match self.host.session(node) {
                Some(s) if !s.exited && s.status == "idle" => break,
                Some(s) if !s.exited => {}
                _ => return fail(gone),
            }
            if Instant::now() >= deadline {
                return fail("tempo esgotado aguardando o agente ficar pronto");
            }
            std::thread::sleep(POLL);
        }

        // 2. Não entrega enquanto o usuário digita.
        loop {
            let Some(s) = self.host.session(node) else {
                return fail(gone);
            };
            if s.exited {
                return fail(gone);
            }
            let typing = s.last_user_input.is_some_and(|t| t.elapsed() < USER_QUIET);
            if !typing {
                break;
            }
            if Instant::now() >= deadline {
                return fail("usuário digitando");
            }
            std::thread::sleep(POLL);
        }
        if !self.is_pending(node, gen) {
            return;
        }

        // 3. Entrega como mensagem do app (sanitizada, bracketed paste, Enter).
        let bracketed = self.host.session(node).is_some_and(|s| s.bracketed_paste);
        let id = self.next_msg_id();
        let payload = delivery::format_message(APP_SENDER, id, &sanitize(text));
        match delivery::deliver(
            &*self.host,
            &self.queues,
            node,
            &payload,
            bracketed,
            self.submit_delay,
        ) {
            Ok(()) => {
                let mut ev = AuditEvent::new("learn");
                ev.msg_id = Some(id);
                ev.from = Some(APP_SENDER.into());
                ev.to = Some(node.into());
                ev.bytes = Some(text.len());
                ev.result = Some("delivered".into());
                self.audit(&ev);
                self.set_state(node, Some(gen), LearnState::Done);
            }
            Err(e) => fail(&format!("falha ao entregar: {e}")),
        }
    }
}
