//! Conteúdo de aprendizado: ordem, limites e sanitização (contracts/learn-hook.md).

use atlasoq_link::learn_content::{build, links_lines, LEARN_MD_MAX, MAX_CHARS, TUTORIAL};
use atlasoq_link::Host;
use atlasoq_link::{learn_hook, learn_prompt};
use atlasoq_store::Direction;
use common::*;

mod common;

#[test]
fn identidade_ligacoes_tutorial_nessa_ordem() {
    let host = FakeHost::new(vec![conn("c1", "a_p", "a_c", Direction::Forward)]);
    let nodes = host.nodes();
    let me = &nodes[0];
    let links = links_lines(me, &nodes, &host.connections());
    let text = build("planner", "coordena", &links, None, None);
    let id = text
        .find("Você é o agente \"planner\" no AtlasOQ. Papel: coordena.")
        .unwrap();
    let li = text.find("Suas ligações agora").unwrap();
    let tu = text.find("# Você é um agente do AtlasOQ").unwrap();
    assert!(id < li && li < tu, "ordem errada");
    assert!(text.contains("- coder: você pode enviar e perguntar"));
    assert!(
        !text.contains("- revisor:"),
        "sem ligação não aparece na lista"
    );
}

#[test]
fn sem_ligacoes_diz_o_que_fazer() {
    let host = FakeHost::new(vec![]);
    let nodes = host.nodes();
    let lines = links_lines(&nodes[0], &nodes, &host.connections());
    assert!(lines[0].contains("nenhuma ligação"));
}

#[test]
fn tutorial_tem_no_maximo_60_linhas() {
    assert!(
        TUTORIAL.lines().count() <= 60,
        "{} linhas",
        TUTORIAL.lines().count()
    );
    for cmd in [
        "atlasoq list",
        "atlasoq send",
        "atlasoq ask",
        "atlasoq reply",
        "atlasoq read",
        "atlasoq note",
    ] {
        if cmd == "atlasoq note" {
            continue; // `note` entra com a US4; o tutorial cobre o que já funciona.
        }
        assert!(TUTORIAL.contains(cmd), "falta {cmd}");
    }
}

#[test]
fn regras_do_time_sao_cortadas_em_4_kib() {
    let grande = "regra ç\n".repeat(2000);
    let text = build("coder", "", &["- x: y".into()], Some(&grande), None);
    assert!(text.contains("Regras do time:"));
    assert!(text.chars().count() <= MAX_CHARS);
    let trecho = text.split("Regras do time:\n").nth(1).unwrap();
    assert!(trecho.len() <= LEARN_MD_MAX + 1000, "regras grandes demais");
}

#[test]
fn texto_total_nunca_passa_de_9000_caracteres() {
    let muitas: Vec<String> = (0..500)
        .map(|i| format!("- agente-{i}: pode te enviar e perguntar"))
        .collect();
    let md = "x".repeat(4000);
    let mem = "- decisão: algo longo\n".repeat(400);
    let text = build("coder", "papel", &muitas, Some(&md), Some(&mem));
    assert!(
        text.chars().count() <= MAX_CHARS,
        "{}",
        text.chars().count()
    );
}

#[test]
fn memoria_mostra_as_ultimas_e_avisa_das_antigas() {
    let mem: String = (1..=25).map(|i| format!("- feito: tarefa {i}\n")).collect();
    let text = build("coder", "papel", &[], None, Some(&mem));
    assert!(text.contains("Suas últimas anotações:"));
    assert!(text.contains("tarefa 25"));
    assert!(!text.contains("tarefa 1\n"), "as mais antigas não entram");
    assert!(
        text.contains("há 5 anotações mais antigas; leia com atlasoq note memory"),
        "{text}"
    );
}

#[test]
fn regras_com_escape_sao_sanitizadas() {
    let text = build("coder", "papel", &[], Some("\x1b[31mregra\x1b[0m"), None);
    assert!(!text.contains('\x1b'));
    assert!(text.contains("regra"));
}

#[test]
fn hook_gera_settings_com_sessionstart_e_permissao() {
    let v = learn_hook::settings();
    assert_eq!(
        v["hooks"]["SessionStart"][0]["matcher"],
        "startup|resume|clear|compact"
    );
    assert_eq!(
        v["hooks"]["SessionStart"][0]["hooks"][0]["command"],
        "atlasoq learn --hook"
    );
    assert_eq!(v["permissions"]["allow"][0], "Bash(atlasoq:*)");
    let dir = tempfile::tempdir().unwrap();
    let path = learn_hook::write(&dir.path().join("coder")).unwrap();
    assert!(path.ends_with("coder/claude-settings.json"));
    let lido: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(lido, v);
}

#[test]
fn prompt_usa_flag_quando_existe() {
    assert_eq!(learn_prompt::args(None, "oi"), vec!["oi"]);
    assert_eq!(
        learn_prompt::args(Some("--prompt"), "oi"),
        vec!["--prompt", "oi"]
    );
    assert_eq!(learn_prompt::args(Some(""), "oi"), vec!["oi"]);
}
