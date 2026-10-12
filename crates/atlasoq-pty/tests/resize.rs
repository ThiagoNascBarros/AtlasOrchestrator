mod common;

use std::time::Duration;

use atlasoq_pty::{LocalSessionHost, SessionHost, SessionSpec};
use common::*;

#[test]
fn resize_reflete_em_stty_size() {
    let host = LocalSessionHost::default();
    let mut spec = SessionSpec::new("sh", std::env::temp_dir());
    spec.cols = 80;
    spec.rows = 24;
    let (id, _rx) = spawn_and_subscribe(&host, spec);
    host.resize(id, 132, 40).unwrap();
    host.write(id, b"stty size; exit\r").unwrap();
    assert!(wait_until(Duration::from_secs(10), || {
        host.screen_text(id, 50).unwrap().contains("40 132")
    }));
    let snap = host.snapshot(id).unwrap();
    assert_eq!((snap.cols, snap.rows), (132, 40));
}
