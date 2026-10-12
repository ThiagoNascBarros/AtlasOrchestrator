//! Status do agente por atividade de saída (research.md R6).

use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Activity {
    /// Processo iniciado, ainda sem saída.
    Starting,
    /// Houve saída recentemente.
    Working,
    /// Sem saída há `idle_after`.
    Idle,
    /// Processo terminou.
    Exited(Option<i32>),
}

#[derive(Debug, Clone, Copy)]
pub struct ActivityConfig {
    pub idle_after: Duration,
}

impl Default for ActivityConfig {
    fn default() -> Self {
        Self {
            idle_after: Duration::from_secs(3),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ActivityTracker {
    config: ActivityConfig,
    last_output: Option<Instant>,
    exited: Option<Option<i32>>,
}

impl ActivityTracker {
    pub fn new(config: ActivityConfig) -> Self {
        Self {
            config,
            last_output: None,
            exited: None,
        }
    }

    pub fn on_output(&mut self, at: Instant) {
        self.last_output = Some(at);
    }

    pub fn on_exit(&mut self, code: Option<i32>) {
        self.exited = Some(code);
    }

    pub fn current(&self, now: Instant) -> Activity {
        if let Some(code) = self.exited {
            return Activity::Exited(code);
        }
        match self.last_output {
            None => Activity::Starting,
            Some(t) if now.duration_since(t) < self.config.idle_after => Activity::Working,
            Some(_) => Activity::Idle,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transicoes() {
        let mut t = ActivityTracker::new(ActivityConfig {
            idle_after: Duration::from_secs(3),
        });
        let t0 = Instant::now();
        assert_eq!(t.current(t0), Activity::Starting);
        t.on_output(t0);
        assert_eq!(t.current(t0 + Duration::from_secs(1)), Activity::Working);
        assert_eq!(t.current(t0 + Duration::from_secs(3)), Activity::Idle);
        t.on_exit(Some(0));
        assert_eq!(t.current(t0), Activity::Exited(Some(0)));
    }
}
