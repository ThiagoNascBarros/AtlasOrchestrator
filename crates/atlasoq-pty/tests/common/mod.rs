#![allow(dead_code)]

use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use atlasoq_pty::{LocalSessionHost, SessionEvent, SessionHost, SessionId, SessionSpec};

pub fn sh(script: &str) -> SessionSpec {
    let mut spec = SessionSpec::new("sh", std::env::temp_dir());
    spec.args = vec!["-c".into(), script.into()];
    spec
}

/// Junta a saída até o fim do processo (ou até `timeout`). Devolve (saída, código).
pub fn collect(rx: &Receiver<SessionEvent>, timeout: Duration) -> (Vec<u8>, Option<Option<i32>>) {
    let deadline = Instant::now() + timeout;
    let mut out = Vec::new();
    while let Some(left) = deadline.checked_duration_since(Instant::now()) {
        match rx.recv_timeout(left) {
            Ok(SessionEvent::Output(b)) => out.extend(b),
            Ok(SessionEvent::Exited(code)) => return (out, Some(code)),
            Ok(SessionEvent::Status(_)) => {}
            Err(_) => break,
        }
    }
    (out, None)
}

pub fn spawn_and_subscribe(
    host: &LocalSessionHost,
    spec: SessionSpec,
) -> (SessionId, Receiver<SessionEvent>) {
    let id = host.spawn(spec).expect("spawn");
    let rx = host.subscribe(id).expect("subscribe");
    (id, rx)
}

/// Espera até `cond` ser verdadeira ou estourar o tempo.
pub fn wait_until(timeout: Duration, mut cond: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if cond() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    cond()
}

pub fn process_alive(pid: u32) -> bool {
    // Zumbis contam como mortos: o processo terminou, só falta o reap.
    match std::fs::read_to_string(format!("/proc/{pid}/stat")) {
        Ok(stat) => !stat
            .rsplit(')')
            .next()
            .is_some_and(|rest| rest.trim_start().starts_with('Z')),
        Err(_) => false,
    }
}
