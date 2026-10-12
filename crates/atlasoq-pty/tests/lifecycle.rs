mod common;

use std::time::Duration;

use atlasoq_pty::{Activity, LocalSessionHost, SessionHost, SessionSpec};
use common::*;

#[test]
fn encerrar_nao_deixa_processos_do_grupo() {
    let host = LocalSessionHost::default();
    // Um filho em segundo plano que ignora SIGHUP: só o SIGKILL do grupo o encerra.
    let (id, rx) = spawn_and_subscribe(&host, sh("trap '' HUP; sleep 300 & echo filho=$!; wait"));
    assert!(wait_until(Duration::from_secs(10), || host
        .screen_text(id, 5)
        .unwrap()
        .contains("filho=")));
    let text = host.screen_text(id, 5).unwrap();
    let child: u32 = text
        .split("filho=")
        .nth(1)
        .unwrap()
        .trim()
        .lines()
        .next()
        .unwrap()
        .parse()
        .unwrap();
    let leader = host.pid(id).unwrap().unwrap();

    host.kill(id, Duration::from_millis(500)).unwrap();
    let (_, code) = collect(&rx, Duration::from_secs(10));
    assert!(code.is_some(), "a sessão deveria terminar");
    assert!(wait_until(Duration::from_secs(5), || !process_alive(
        leader
    ) && !process_alive(
        child
    )));
    assert!(matches!(host.activity(id).unwrap(), Activity::Exited(_)));
}

#[test]
fn shutdown_all_encerra_todas() {
    let host = LocalSessionHost::default();
    let ids: Vec<_> = (0..3)
        .map(|_| {
            let mut s = SessionSpec::new("sleep", "/");
            s.args = vec!["300".into()];
            host.spawn(s).unwrap()
        })
        .collect();
    let pids: Vec<u32> = ids.iter().filter_map(|id| host.pid(*id).unwrap()).collect();
    host.shutdown_all(Duration::from_millis(500));
    assert!(wait_until(Duration::from_secs(5), || pids
        .iter()
        .all(|p| !process_alive(*p))));
}

#[test]
fn status_vai_de_trabalhando_para_ocioso() {
    let host = LocalSessionHost::new(atlasoq_pty::ActivityConfig {
        idle_after: Duration::from_millis(300),
    });
    let (id, _rx) = spawn_and_subscribe(&host, sh("echo oi; sleep 30"));
    assert!(wait_until(Duration::from_secs(5), || host
        .activity(id)
        .unwrap()
        == Activity::Working));
    assert!(wait_until(Duration::from_secs(5), || host
        .activity(id)
        .unwrap()
        == Activity::Idle));
    host.kill(id, Duration::from_millis(200)).unwrap();
}
