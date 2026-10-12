//! Contrato de `atlasoq learn` (contracts/cli.md): dentro do app e fora dele.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;

use assert_cmd::Command;
use serde_json::{json, Value};

const GENERIC_START: &str = "# Você é um agente do AtlasOQ";

fn cli() -> Command {
    let mut c = Command::cargo_bin("atlasoq").unwrap();
    c.env_remove("ATLASOQ_SOCKET")
        .env_remove("ATLASOQ_NODE_ID")
        .env_remove("ATLASOQ_NODE_TOKEN");
    c
}

/// App de mentira que responde `learn` com `text` e devolve a requisição recebida.
fn fake_app(text: &str) -> (tempfile::TempDir, std::sync::mpsc::Receiver<Value>, String) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("s.sock");
    let listener = UnixListener::bind(&path).unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    let reply = json!({"v": 1, "ok": true, "data": {"text": text}}).to_string();
    std::thread::spawn(move || {
        if let Ok((stream, _)) = listener.accept() {
            let mut line = String::new();
            BufReader::new(&stream).read_line(&mut line).unwrap();
            tx.send(serde_json::from_str(&line).unwrap()).unwrap();
            let mut w = &stream;
            writeln!(w, "{reply}").unwrap();
        }
    });
    let sock = path.display().to_string();
    (dir, rx, sock)
}

#[test]
fn fora_do_app_imprime_o_tutorial_generico_com_codigo_0() {
    let o = cli().arg("learn").output().unwrap();
    assert_eq!(o.status.code(), Some(0));
    let s = String::from_utf8_lossy(&o.stdout);
    assert!(s.starts_with(GENERIC_START));
    assert!(s.lines().count() <= 60);
    assert!(o.stderr.is_empty());
}

#[test]
fn dentro_do_app_imprime_o_texto_do_agente() {
    let (_dir, rx, sock) = fake_app("Você é o agente \"coder\" no AtlasOQ. Papel: x.");
    let o = cli()
        .arg("learn")
        .env("ATLASOQ_SOCKET", sock)
        .env("ATLASOQ_NODE_ID", "a_c")
        .env("ATLASOQ_NODE_TOKEN", "t")
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&o.stdout),
        "Você é o agente \"coder\" no AtlasOQ. Papel: x."
    );
    let req = rx.recv().unwrap();
    assert_eq!(req["op"], "learn");
    assert_eq!(req["args"]["hook"], false);
}

#[test]
fn hook_pede_ao_app_que_marque_o_aprendizado() {
    let (_dir, rx, sock) = fake_app("texto");
    let o = cli()
        .args(["learn", "--hook"])
        .env("ATLASOQ_SOCKET", sock)
        .env("ATLASOQ_NODE_ID", "a_c")
        .env("ATLASOQ_NODE_TOKEN", "t")
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(0));
    assert_eq!(rx.recv().unwrap()["args"]["hook"], true);
}

#[test]
fn app_inacessivel_cai_no_tutorial_generico_sem_falhar() {
    let o = cli()
        .arg("learn")
        .env("ATLASOQ_SOCKET", "/tmp/nao-existe-atlasoq.sock")
        .env("ATLASOQ_NODE_ID", "a_c")
        .env("ATLASOQ_NODE_TOKEN", "t")
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(0), "o hook não pode quebrar a sessão");
    assert!(String::from_utf8_lossy(&o.stdout).starts_with(GENERIC_START));
}
