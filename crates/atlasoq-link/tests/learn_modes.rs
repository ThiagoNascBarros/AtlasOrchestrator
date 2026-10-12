//! Modos de aprendizado (`hook`, `prompt`, `type`, `off`) com um host de mentira.

mod common;

use std::time::{Duration, Instant};

use atlasoq_link::LearnState;
use atlasoq_store::{Direction, LearnMode};
use common::*;
use serde_json::json;

fn setup() -> (
    std::sync::Arc<FakeHost>,
    std::sync::Arc<atlasoq_link::Broker>,
) {
    let host = FakeHost::new(vec![conn("c1", "a_p", "a_c", Direction::Forward)]);
    let b = broker(&host);
    (host, b)
}

fn wait_state(b: &atlasoq_link::Broker, node: &str, want: &str) -> LearnState {
    let end = Instant::now() + Duration::from_secs(5);
    loop {
        let s = b.learn_state(node).unwrap();
        if s.status() == want || Instant::now() > end {
            return s;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn hook_fica_pendente_ate_o_agente_chamar_learn_hook() {
    let (_, b) = setup();
    let gen = b.learn_begin(
        "a_c",
        LearnMode::Hook,
        Duration::from_secs(10),
        String::new(),
    );
    assert_eq!(b.learn_state("a_c"), Some(LearnState::Pending));
    let tok = b.issue_token("a_c");
    let r = call(&b, "a_c", &tok, "learn", json!({"hook": true}));
    assert!(r.ok, "{r:?}");
    assert!(r.data.unwrap()["text"]
        .as_str()
        .unwrap()
        .contains("Você é o agente \"coder\""));
    assert_eq!(b.learn_state("a_c"), Some(LearnState::Done));
    assert!(gen >= 1);
}

#[test]
fn learn_sem_hook_so_devolve_o_texto() {
    let (_, b) = setup();
    b.learn_begin(
        "a_c",
        LearnMode::Hook,
        Duration::from_secs(10),
        String::new(),
    );
    let tok = b.issue_token("a_c");
    call(&b, "a_c", &tok, "learn", json!({"hook": false}));
    assert_eq!(b.learn_state("a_c"), Some(LearnState::Pending));
}

#[test]
fn hook_sem_chamada_falha_por_tempo() {
    let (_, b) = setup();
    b.learn_begin(
        "a_c",
        LearnMode::Hook,
        Duration::from_millis(100),
        String::new(),
    );
    let s = wait_state(&b, "a_c", "failed");
    assert_eq!(s.status(), "failed");
    assert!(s.reason().unwrap().contains("hook"));
}

#[test]
fn prompt_conclui_ao_iniciar_e_off_nao_faz_nada() {
    let (host, b) = setup();
    b.learn_begin(
        "a_c",
        LearnMode::Prompt,
        Duration::from_secs(1),
        "texto".into(),
    );
    assert_eq!(b.learn_state("a_c"), Some(LearnState::Done));
    b.learn_begin(
        "a_r",
        LearnMode::Off,
        Duration::from_secs(1),
        "texto".into(),
    );
    assert_eq!(b.learn_state("a_r"), Some(LearnState::Off));
    std::thread::sleep(Duration::from_millis(100));
    assert!(host.text_for("a_c").is_empty() && host.text_for("a_r").is_empty());
    assert!(host
        .learn_events()
        .iter()
        .any(|(n, s, _)| n == "a_r" && *s == "off"));
}

#[test]
fn type_entrega_como_mensagem_do_app_quando_ocioso() {
    let (host, b) = setup();
    b.learn_begin(
        "a_c",
        LearnMode::Type,
        Duration::from_secs(5),
        "Você é o agente \"coder\".".into(),
    );
    let s = wait_state(&b, "a_c", "done");
    assert_eq!(s.status(), "done");
    let entregue = host.text_for("a_c");
    assert!(entregue.contains("[atlasoq de:app msg:"), "{entregue:?}");
    assert!(entregue.contains("Você é o agente \"coder\"."));
    assert!(entregue.ends_with('\r'));
}

#[test]
fn type_espera_o_usuario_parar_de_digitar_e_falha_se_nao_parar() {
    let (host, b) = setup();
    *host.last_input.lock().unwrap() = Some(Instant::now());
    b.learn_begin(
        "a_c",
        LearnMode::Type,
        Duration::from_millis(600),
        "x".into(),
    );
    let s = wait_state(&b, "a_c", "failed");
    assert_eq!(s.reason().as_deref(), Some("usuário digitando"));
    assert!(
        host.text_for("a_c").is_empty(),
        "não pode entregar enquanto o usuário digita"
    );
}

#[test]
fn type_falha_se_o_agente_nao_fica_ocioso() {
    let (host, b) = setup();
    *host.status.lock().unwrap() = "starting";
    b.learn_begin(
        "a_c",
        LearnMode::Type,
        Duration::from_millis(400),
        "x".into(),
    );
    let s = wait_state(&b, "a_c", "failed");
    assert!(s.reason().unwrap().contains("tempo esgotado"));
    assert!(host.text_for("a_c").is_empty());
}

#[test]
fn encerrar_o_agente_durante_o_aprendizado_falha_o_aprendizado() {
    let (_, b) = setup();
    b.learn_begin(
        "a_c",
        LearnMode::Hook,
        Duration::from_secs(10),
        String::new(),
    );
    b.node_stopped("a_c");
    assert_eq!(b.learn_state("a_c").unwrap().status(), "failed");
}

#[test]
fn sessao_antiga_nao_responde_pela_nova() {
    let (_, b) = setup();
    let antiga = b.learn_begin(
        "a_c",
        LearnMode::Hook,
        Duration::from_millis(150),
        String::new(),
    );
    let nova = b.learn_begin(
        "a_c",
        LearnMode::Hook,
        Duration::from_secs(10),
        String::new(),
    );
    assert_ne!(antiga, nova);
    std::thread::sleep(Duration::from_millis(300));
    // O timer da sessão antiga expirou, mas a nova continua pendente.
    assert_eq!(b.learn_state("a_c"), Some(LearnState::Pending));
}

#[test]
fn eventos_de_aprendizado_sao_publicados() {
    let (host, b) = setup();
    b.learn_begin(
        "a_c",
        LearnMode::Hook,
        Duration::from_secs(10),
        String::new(),
    );
    let tok = b.issue_token("a_c");
    call(&b, "a_c", &tok, "learn", json!({"hook": true}));
    let ev = host.learn_events();
    assert_eq!(ev.last().unwrap().1, "done");
    assert_eq!(ev[0].1, "pending");
}
