//! Encerramento de grupos de processos (research.md R12).

use std::thread;
use std::time::{Duration, Instant};

/// Envia SIGHUP ao grupo, espera até `grace` por `is_done`, e então manda SIGKILL.
pub(crate) fn terminate_group(pgid: u32, grace: Duration, is_done: impl Fn() -> bool) {
    let pgid = pgid as libc::pid_t;
    // SAFETY: killpg só envia um sinal; um pgid inválido resulta em erro ignorado.
    unsafe {
        libc::killpg(pgid, libc::SIGHUP);
        libc::killpg(pgid, libc::SIGCONT);
    }
    let deadline = Instant::now() + grace;
    while Instant::now() < deadline {
        if is_done() && !group_alive(pgid) {
            return;
        }
        thread::sleep(Duration::from_millis(20));
    }
    // SAFETY: idem.
    unsafe {
        libc::killpg(pgid, libc::SIGKILL);
    }
}

/// Se ainda existe algum processo no grupo.
fn group_alive(pgid: libc::pid_t) -> bool {
    // SAFETY: sinal 0 só verifica a existência do grupo.
    unsafe { libc::killpg(pgid, 0) == 0 }
}
