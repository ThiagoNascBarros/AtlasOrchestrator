mod common;

use std::time::Duration;

use atlasoq_pty::{LocalSessionHost, SessionHost};
use common::*;

#[test]
fn captura_saida_e_codigo_de_saida() {
    let host = LocalSessionHost::default();
    let (_, rx) = spawn_and_subscribe(&host, sh("printf ok"));
    let (out, code) = collect(&rx, Duration::from_secs(10));
    assert!(String::from_utf8_lossy(&out).contains("ok"));
    assert_eq!(code, Some(Some(0)));
}

#[test]
fn codigo_de_saida_diferente_de_zero() {
    let host = LocalSessionHost::default();
    let (_, rx) = spawn_and_subscribe(&host, sh("exit 7"));
    let (_, code) = collect(&rx, Duration::from_secs(10));
    assert_eq!(code, Some(Some(7)));
}

#[test]
fn escreve_na_entrada_e_le_a_tela() {
    let host = LocalSessionHost::default();
    let (id, rx) = spawn_and_subscribe(&host, sh("read linha; echo \"recebi: $linha\""));
    host.write_user(id, b"ola\r").unwrap();
    let (_, code) = collect(&rx, Duration::from_secs(10));
    assert_eq!(code, Some(Some(0)));
    assert!(host.screen_text(id, 10).unwrap().contains("recebi: ola"));
    assert!(host.last_user_input(id).unwrap().is_some());
}

#[test]
fn ambiente_do_terminal() {
    let host = LocalSessionHost::default();
    let mut spec = sh("echo \"$TERM $COLORTERM $ATLASOQ_TESTE\"");
    spec.env.push(("ATLASOQ_TESTE".into(), "valor".into()));
    let (id, rx) = spawn_and_subscribe(&host, spec);
    collect(&rx, Duration::from_secs(10));
    assert!(host
        .screen_text(id, 5)
        .unwrap()
        .contains("xterm-256color truecolor valor"));
}

#[test]
fn comando_inexistente_falha_ou_sai_com_erro() {
    let host = LocalSessionHost::default();
    let mut spec = sh("");
    spec.command = "/nao/existe/atlasoq-xyz".into();
    spec.args.clear();
    match host.spawn(spec) {
        Err(_) => {}
        Ok(id) => {
            let rx = host.subscribe(id).unwrap();
            let (_, code) = collect(&rx, Duration::from_secs(10));
            assert_ne!(code, Some(Some(0)));
        }
    }
}
