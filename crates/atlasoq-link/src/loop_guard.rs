//! Proteção contra loops: limite de trocas sem entrada humana, taxa com rajada e vai-e-vem.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use atlasoq_store::{ConnectionLimits, PauseReason};

/// Sem trocas por este tempo, o contador volta a zero.
pub const WINDOW: Duration = Duration::from_secs(120);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Allow,
    Pause(PauseReason),
}

#[derive(Debug)]
pub struct LoopGuard {
    exchanges: u32,
    last_exchange: Option<Instant>,
    tokens: f64,
    last_refill: Instant,
    recent: VecDeque<u64>,
    seen_user_input: Option<Instant>,
}

impl LoopGuard {
    pub fn new(now: Instant, limits: &ConnectionLimits) -> Self {
        Self {
            exchanges: 0,
            last_exchange: None,
            tokens: f64::from(limits.burst),
            last_refill: now,
            recent: VecDeque::new(),
            seen_user_input: None,
        }
    }

    /// Zera os contadores (o usuário reativou a conexão).
    pub fn reset(&mut self, now: Instant, limits: &ConnectionLimits) {
        *self = Self {
            seen_user_input: self.seen_user_input,
            ..Self::new(now, limits)
        };
    }

    fn apply_user_input(&mut self, user_input: Option<Instant>) {
        if user_input.is_some() && user_input != self.seen_user_input {
            self.seen_user_input = user_input;
            self.exchanges = 0;
            self.recent.clear();
        }
    }

    /// Registra uma mensagem nova (`send` ou `ask`). Se devolver `Pause`, a mensagem não conta
    /// e MUST ser recusada.
    pub fn record(
        &mut self,
        now: Instant,
        text_hash: u64,
        user_input: Option<Instant>,
        limits: &ConnectionLimits,
    ) -> Verdict {
        self.apply_user_input(user_input);
        if self
            .last_exchange
            .is_some_and(|t| now.duration_since(t) > WINDOW)
        {
            self.exchanges = 0;
        }

        let elapsed = now.duration_since(self.last_refill).as_secs_f64() / 60.0;
        self.tokens =
            (self.tokens + elapsed * f64::from(limits.rate_per_min)).min(f64::from(limits.burst));
        self.last_refill = now;
        if self.tokens < 1.0 {
            return Verdict::Pause(PauseReason::RateLimit);
        }

        let n = limits.ping_pong_repeats as usize;
        if n > 0 {
            let repeated = self.recent.len() + 1 >= n
                && self
                    .recent
                    .iter()
                    .rev()
                    .take(n - 1)
                    .all(|h| *h == text_hash);
            if repeated {
                return Verdict::Pause(PauseReason::PingPong);
            }
        }

        if self.exchanges >= limits.max_exchanges {
            return Verdict::Pause(PauseReason::MaxExchanges);
        }

        self.tokens -= 1.0;
        self.exchanges += 1;
        self.last_exchange = Some(now);
        self.recent.push_back(text_hash);
        while self.recent.len() > n.max(1) {
            self.recent.pop_front();
        }
        Verdict::Allow
    }

    /// Conta uma resposta (`reply`) como troca, sem poder ser recusada.
    pub fn note_exchange(&mut self, now: Instant, user_input: Option<Instant>) {
        self.apply_user_input(user_input);
        self.exchanges += 1;
        self.last_exchange = Some(now);
    }

    pub fn exchanges(&self) -> u32 {
        self.exchanges
    }
}
