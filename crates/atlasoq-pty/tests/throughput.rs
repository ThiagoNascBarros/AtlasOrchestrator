mod common;

use std::time::{Duration, Instant};

use atlasoq_pty::{LocalSessionHost, SessionEvent, SessionHost};
use common::*;

#[test]
fn saida_volumosa_nao_bloqueia() {
    let host = LocalSessionHost::default();
    // ~50 MB de saída.
    let (id, rx) = spawn_and_subscribe(
        &host,
        sh("head -c 52428800 /dev/zero | tr '\\0' 'x'; echo; echo FIM"),
    );
    let start = Instant::now();
    let mut total = 0usize;
    let mut code = None;
    while start.elapsed() < Duration::from_secs(120) {
        match rx.recv_timeout(Duration::from_secs(30)) {
            Ok(SessionEvent::Output(b)) => total += b.len(),
            Ok(SessionEvent::Exited(c)) => {
                code = Some(c);
                break;
            }
            Ok(SessionEvent::Status(_)) => {}
            Err(_) => break,
        }
    }
    assert_eq!(code, Some(Some(0)));
    assert!(total >= 50 * 1024 * 1024, "recebidos {total} bytes");
    assert!(host.screen_text(id, 3).unwrap().contains("FIM"));
    // A UI continua respondendo: snapshot é imediato.
    let t = Instant::now();
    host.snapshot(id).unwrap();
    assert!(t.elapsed() < Duration::from_millis(200));
}
