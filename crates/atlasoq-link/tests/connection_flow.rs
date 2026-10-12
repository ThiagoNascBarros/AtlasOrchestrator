mod common;

use std::time::Duration;

use atlasoq_core::ErrorKind;
use atlasoq_store::{ConnState, Direction, PauseReason};
use common::*;
use serde_json::json;

fn setup(
    direction: Direction,
) -> (
    std::sync::Arc<FakeHost>,
    std::sync::Arc<atlasoq_link::Broker>,
    String,
    String,
) {
    let host = FakeHost::new(vec![conn("c1", "a_p", "a_c", direction)]);
    let b = broker(&host);
    let (tp, tc) = (b.issue_token("a_p"), b.issue_token("a_c"));
    (host, b, tp, tc)
}

#[test]
fn send_entrega_com_cabecalho_e_enter() {
    let (host, b, tp, _) = setup(Direction::Forward);
    let r = call(
        &b,
        "a_p",
        &tp,
        "send",
        json!({"to": "coder", "text": "oi\x1b[31m você"}),
    );
    assert!(r.ok, "{r:?}");
    let entregue = host.text_for("a_c");
    assert!(
        entregue.starts_with("[atlasoq de:planner msg:"),
        "{entregue:?}"
    );
    assert!(entregue.contains("oi você") && !entregue.contains('\x1b'));
    assert!(entregue.ends_with('\r'));
}

#[test]
fn sentido_inverso_e_sem_conexao_sao_recusados() {
    let (host, b, _, tc) = setup(Direction::Forward);
    let r = call(
        &b,
        "a_c",
        &tc,
        "send",
        json!({"to": "planner", "text": "oi"}),
    );
    assert_eq!(r.error, Some(ErrorKind::WrongDirection));
    assert_eq!(r.code, Some(3));
    assert!(host.text_for("a_p").is_empty());

    let r = call(
        &b,
        "a_c",
        &tc,
        "send",
        json!({"to": "revisor", "text": "oi"}),
    );
    assert_eq!(r.error, Some(ErrorKind::NoConnection));

    let r = call(
        &b,
        "a_c",
        &tc,
        "send",
        json!({"to": "fantasma", "text": "oi"}),
    );
    assert_eq!(r.error, Some(ErrorKind::UnknownTarget));
    assert_eq!(r.code, Some(4));
}

#[test]
fn conexao_bidirecional_vale_nos_dois_sentidos() {
    let (_, b, tp, tc) = setup(Direction::Both);
    assert!(call(&b, "a_p", &tp, "send", json!({"to": "coder", "text": "a"})).ok);
    assert!(
        call(
            &b,
            "a_c",
            &tc,
            "send",
            json!({"to": "planner", "text": "b"})
        )
        .ok
    );
}

#[test]
fn pausa_recusa_na_hora_e_apagar_tambem() {
    let (host, b, tp, _) = setup(Direction::Forward);
    host.set_connection_state_for_test("c1", ConnState::Paused);
    let r = call(&b, "a_p", &tp, "send", json!({"to": "coder", "text": "oi"}));
    assert_eq!(r.error, Some(ErrorKind::ConnectionPaused));
    host.conns.lock().unwrap().clear();
    let r = call(&b, "a_p", &tp, "send", json!({"to": "coder", "text": "oi"}));
    assert_eq!(r.error, Some(ErrorKind::NoConnection));
}

#[test]
fn texto_vazio_ou_grande_e_recusado() {
    let (_, b, tp, _) = setup(Direction::Forward);
    let r = call(
        &b,
        "a_p",
        &tp,
        "send",
        json!({"to": "coder", "text": "  \x1b[0m "}),
    );
    assert_eq!((r.error, r.code), (Some(ErrorKind::TooLarge), Some(7)));
    let grande = "x".repeat(16 * 1024 + 1);
    let r = call(
        &b,
        "a_p",
        &tp,
        "send",
        json!({"to": "coder", "text": grande}),
    );
    assert_eq!(r.error, Some(ErrorKind::TooLarge));
}

#[test]
fn destino_encerrado_e_indisponivel() {
    let (host, b, tp, _) = setup(Direction::Forward);
    host.exited.lock().unwrap().push("a_c".into());
    let r = call(&b, "a_p", &tp, "send", json!({"to": "coder", "text": "oi"}));
    assert_eq!((r.error, r.code), (Some(ErrorKind::TargetExited), Some(4)));
}

#[test]
fn read_exige_conexao_no_sentido_eu_para_agente() {
    let (_, b, tp, tc) = setup(Direction::Forward);
    let r = call(
        &b,
        "a_p",
        &tp,
        "read",
        json!({"from": "coder", "lines": 10}),
    );
    assert!(r.ok);
    assert_eq!(r.data.unwrap()["text"], "tela de a_c");
    let r = call(&b, "a_c", &tc, "read", json!({"from": "planner"}));
    assert_eq!(r.error, Some(ErrorKind::WrongDirection));
}

#[test]
fn list_mostra_so_quem_esta_conectado() {
    let (_, b, tp, _) = setup(Direction::Forward);
    let r = call(&b, "a_p", &tp, "list", json!({}));
    let d = r.data.unwrap();
    assert_eq!(d["you"]["name"], "planner");
    assert_eq!(d["agents"].as_array().unwrap().len(), 1);
    assert_eq!(d["agents"][0]["name"], "coder");
    assert_eq!(d["agents"][0]["relation"], "outbound");
}

#[test]
fn vinte_e_uma_mensagens_pausam_a_conexao() {
    let (host, b, tp, _) = setup(Direction::Forward);
    for i in 0..20 {
        let r = call(
            &b,
            "a_p",
            &tp,
            "send",
            json!({"to": "coder", "text": format!("msg {i}")}),
        );
        assert!(r.ok, "{i}: {r:?}");
    }
    let r = call(
        &b,
        "a_p",
        &tp,
        "send",
        json!({"to": "coder", "text": "msg 21"}),
    );
    assert_eq!(r.error, Some(ErrorKind::ConnectionPaused));
    let c = host.conns.lock().unwrap()[0].clone();
    assert_eq!(c.state, ConnState::AutoPaused);
    assert_eq!(c.pause_reason, Some(PauseReason::MaxExchanges));
}

#[test]
fn ask_e_reply_devolvem_o_texto_exato() {
    let (host, b, tp, tc) = setup(Direction::Forward);
    let b2 = b.clone();
    let asker = std::thread::spawn(move || {
        call(
            &b2,
            "a_p",
            &tp,
            "ask",
            json!({"to": "coder", "text": "quantos testes?", "timeout_s": 10}),
        )
    });
    for _ in 0..100 {
        if host.text_for("a_c").contains("[atlasoq ask de:planner id:") {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(host
        .text_for("a_c")
        .contains("quantos testes? — responda com: atlasoq reply"));

    // Coder responde: a pergunta pendente aparece no list.
    let l = call(&b, "a_c", &tc, "list", json!({})).data.unwrap();
    assert_eq!(l["pending"][0]["from"], "planner");
    let r = call(
        &b,
        "a_c",
        &tc,
        "reply",
        json!({"to": null, "text": "42 testes, 0 falhas"}),
    );
    assert!(r.ok, "{r:?}");

    let resposta = asker.join().unwrap();
    assert!(resposta.ok, "{resposta:?}");
    assert_eq!(resposta.data.unwrap()["text"], "42 testes, 0 falhas");
}

#[test]
fn ask_esgotado_devolve_a_tela_com_codigo_5_e_reply_depois_e_expirado() {
    let (_, b, tp, tc) = setup(Direction::Forward);
    let r = call(
        &b,
        "a_p",
        &tp,
        "ask",
        json!({"to": "coder", "text": "oi?", "timeout_s": 1}),
    );
    assert_eq!((r.error, r.code), (Some(ErrorKind::Timeout), Some(5)));
    assert_eq!(r.data.unwrap()["screen"], "tela de a_c");
    let r = call(
        &b,
        "a_c",
        &tc,
        "reply",
        json!({"to": null, "text": "tarde demais"}),
    );
    assert_eq!((r.error, r.code), (Some(ErrorKind::AskExpired), Some(3)));
}

#[test]
fn reply_sem_pergunta_e_codigo_3() {
    let (_, b, _, tc) = setup(Direction::Forward);
    let r = call(&b, "a_c", &tc, "reply", json!({"to": null, "text": "oi"}));
    assert_eq!((r.error, r.code), (Some(ErrorKind::NoPendingAsk), Some(3)));
}

#[test]
fn reply_ambiguo_lista_os_perguntadores_e_aceita_o_nome() {
    let host = FakeHost::new(vec![
        conn("c1", "a_p", "a_c", Direction::Forward),
        conn("c2", "a_r", "a_c", Direction::Forward),
    ]);
    let b = broker(&host);
    let (tp, tr, tc) = (
        b.issue_token("a_p"),
        b.issue_token("a_r"),
        b.issue_token("a_c"),
    );
    let (b1, b2) = (b.clone(), b.clone());
    let h1 = std::thread::spawn(move || {
        call(
            &b1,
            "a_p",
            &tp,
            "ask",
            json!({"to": "coder", "text": "q1", "timeout_s": 10}),
        )
    });
    let h2 = std::thread::spawn(move || {
        call(
            &b2,
            "a_r",
            &tr,
            "ask",
            json!({"to": "coder", "text": "q2", "timeout_s": 10}),
        )
    });
    for _ in 0..100 {
        if call(&b, "a_c", &tc, "list", json!({})).data.unwrap()["pending"]
            .as_array()
            .unwrap()
            .len()
            == 2
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let r = call(&b, "a_c", &tc, "reply", json!({"to": null, "text": "x"}));
    assert_eq!(
        (r.error, r.code),
        (Some(ErrorKind::AmbiguousReply), Some(2))
    );
    assert!(
        call(
            &b,
            "a_c",
            &tc,
            "reply",
            json!({"to": "planner", "text": "para o planner"})
        )
        .ok
    );
    assert!(
        call(
            &b,
            "a_c",
            &tc,
            "reply",
            json!({"to": "revisor", "text": "para o revisor"})
        )
        .ok
    );
    assert_eq!(h1.join().unwrap().data.unwrap()["text"], "para o planner");
    assert_eq!(h2.join().unwrap().data.unwrap()["text"], "para o revisor");
}

#[test]
fn pausar_cancela_o_ask_em_espera() {
    let (host, b, tp, _) = setup(Direction::Forward);
    let b2 = b.clone();
    let h = std::thread::spawn(move || {
        call(
            &b2,
            "a_p",
            &tp,
            "ask",
            json!({"to": "coder", "text": "oi?", "timeout_s": 30}),
        )
    });
    for _ in 0..100 {
        if host.text_for("a_c").contains("atlasoq ask") {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    host.set_connection_state_for_test("c1", ConnState::Paused);
    let r = h.join().unwrap();
    assert_eq!(
        (r.error, r.code),
        (Some(ErrorKind::ConnectionPaused), Some(3))
    );
}

#[test]
fn destino_encerrado_durante_o_ask_da_codigo_4() {
    let (host, b, tp, _) = setup(Direction::Forward);
    let b2 = b.clone();
    let h = std::thread::spawn(move || {
        call(
            &b2,
            "a_p",
            &tp,
            "ask",
            json!({"to": "coder", "text": "oi?", "timeout_s": 30}),
        )
    });
    for _ in 0..100 {
        if host.text_for("a_c").contains("atlasoq ask") {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    host.exited.lock().unwrap().push("a_c".into());
    let r = h.join().unwrap();
    assert_eq!((r.error, r.code), (Some(ErrorKind::TargetExited), Some(4)));
}
