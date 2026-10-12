//! Contrato da CLI (contracts/cli.md): comandos, saídas, códigos e erros, contra um app de mentira.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::sync::mpsc;

use assert_cmd::Command;
use serde_json::{json, Value};

/// App de mentira: responde uma requisição com `reply` e devolve a requisição recebida.
struct FakeApp {
    dir: tempfile::TempDir,
    rx: mpsc::Receiver<Value>,
}

fn fake_app(reply: Value) -> FakeApp {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("s.sock");
    let listener = UnixListener::bind(&path).unwrap();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        if let Ok((stream, _)) = listener.accept() {
            let mut line = String::new();
            BufReader::new(&stream).read_line(&mut line).unwrap();
            tx.send(serde_json::from_str(&line).unwrap()).unwrap();
            let mut w = &stream;
            writeln!(w, "{reply}").unwrap();
        }
    });
    FakeApp { dir, rx }
}

fn atlasoq(app: Option<&FakeApp>) -> Command {
    let mut c = Command::cargo_bin("atlasoq").unwrap();
    c.env_remove("ATLASOQ_SOCKET")
        .env_remove("ATLASOQ_NODE_ID")
        .env_remove("ATLASOQ_NODE_TOKEN");
    if let Some(app) = app {
        c.env("ATLASOQ_SOCKET", app.dir.path().join("s.sock"))
            .env("ATLASOQ_NODE_ID", "a_1")
            .env("ATLASOQ_NODE_TOKEN", "tok");
    }
    c
}

fn ok(data: Value) -> Value {
    json!({"v": 1, "ok": true, "data": data})
}

fn err(code: i32, error: &str, message: &str, hint: &str) -> Value {
    json!({"v": 1, "ok": false, "code": code, "error": error, "message": message, "hint": hint})
}

fn out(o: &std::process::Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}
fn errout(o: &std::process::Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

#[test]
fn send_junta_argumentos_e_imprime_a_confirmacao() {
    let app = fake_app(ok(json!({"msg_id": 42, "to": "revisor"})));
    let o = atlasoq(Some(&app))
        .args(["send", "revisor", "revise", "o", "main"])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(0));
    assert_eq!(out(&o), "enviado para revisor (msg 42)\n");
    let req = app.rx.recv().unwrap();
    assert_eq!(req["op"], "send");
    assert_eq!(req["node"], "a_1");
    assert_eq!(req["token"], "tok");
    assert_eq!(
        req["args"],
        json!({"to": "revisor", "text": "revise o main"})
    );
}

#[test]
fn send_le_o_texto_da_entrada_padrao() {
    let app = fake_app(ok(json!({"msg_id": 1, "to": "revisor"})));
    let o = atlasoq(Some(&app))
        .args(["send", "revisor"])
        .write_stdin("texto longo\ncom linhas\n")
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(0));
    assert_eq!(
        app.rx.recv().unwrap()["args"]["text"],
        "texto longo\ncom linhas"
    );
}

#[test]
fn json_imprime_a_resposta_em_stdout() {
    let app = fake_app(ok(json!({"msg_id": 7, "to": "revisor"})));
    let o = atlasoq(Some(&app))
        .args(["send", "revisor", "oi", "--json"])
        .output()
        .unwrap();
    let v: Value = serde_json::from_str(out(&o).trim()).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["msg_id"], 7);
}

#[test]
fn erro_do_app_vai_para_stderr_com_proximo_passo_e_codigo() {
    let app = fake_app(err(
        3,
        "no_connection",
        "não existe conexão de \"a\" para \"b\".",
        "peça ao usuário para conectar",
    ));
    let o = atlasoq(Some(&app))
        .args(["send", "b", "oi"])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(3));
    assert_eq!(
        errout(&o),
        "erro: não existe conexão de \"a\" para \"b\".\n      peça ao usuário para conectar\n"
    );
    assert_eq!(out(&o), "");
}

#[test]
fn ask_imprime_exatamente_a_resposta() {
    let app = fake_app(ok(json!({"text": "42 testes", "from": "coder"})));
    let o = atlasoq(Some(&app))
        .args(["ask", "coder", "quantos?", "--timeout", "30"])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(0));
    assert_eq!(out(&o), "42 testes\n");
    assert_eq!(app.rx.recv().unwrap()["args"]["timeout_s"], 30);
}

#[test]
fn ask_esgotado_imprime_a_tela_com_codigo_5() {
    let mut r = err(5, "timeout", "x", "y");
    r["data"] = json!({"screen": "linha 1\nlinha 2"});
    let app = fake_app(r);
    let o = atlasoq(Some(&app))
        .args(["ask", "coder", "oi?", "--timeout", "3"])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(5));
    assert_eq!(
        out(&o),
        "sem resposta de coder em 3s; tela recente:\nlinha 1\nlinha 2\n"
    );
}

#[test]
fn read_envia_linhas_padrao_60_e_imprime_o_texto() {
    let app = fake_app(ok(json!({"text": "saida do coder"})));
    let o = atlasoq(Some(&app))
        .args(["read", "coder"])
        .output()
        .unwrap();
    assert_eq!(out(&o), "saida do coder\n");
    assert_eq!(
        app.rx.recv().unwrap()["args"],
        json!({"from": "coder", "lines": 60})
    );
}

#[test]
fn reply_com_uma_palavra_e_texto_com_duas_envia_o_primeiro_como_agente() {
    let app = fake_app(ok(json!({"to": "planner"})));
    let o = atlasoq(Some(&app))
        .args(["reply", "tudo certo"])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(0));
    assert_eq!(
        app.rx.recv().unwrap()["args"],
        json!({"to": null, "text": "tudo certo"})
    );

    let app = fake_app(ok(json!({"to": "planner"})));
    atlasoq(Some(&app))
        .args(["reply", "planner", "tudo", "certo"])
        .output()
        .unwrap();
    assert_eq!(
        app.rx.recv().unwrap()["args"],
        json!({"to": "planner", "text": "tudo certo"})
    );
}

#[test]
fn list_formata_agentes_e_perguntas_pendentes() {
    let app = fake_app(ok(json!({
        "you": {"name": "coder", "role": "implementa", "status": "trabalhando"},
        "agents": [
            {"name": "planner", "relation": "inbound", "state": "active", "status": "ocioso"},
            {"name": "revisor", "relation": "outbound", "state": "auto_paused", "status": "trabalhando"}
        ],
        "notes": [],
        "pending": [{"from": "planner", "id": 17}]
    })));
    let o = atlasoq(Some(&app)).arg("list").output().unwrap();
    let s = out(&o);
    assert!(
        s.starts_with("você: coder — implementa (trabalhando)\nagentes:\n"),
        "{s}"
    );
    assert!(s.contains("planner   ← pode te enviar e perguntar"), "{s}");
    assert!(
        s.contains("revisor   → você pode enviar e perguntar"),
        "{s}"
    );
    assert!(s.contains("[pausada automaticamente]"), "{s}");
    assert!(
        s.contains(
            "perguntas pendentes para você: 1 (de planner) — responda com: atlasoq reply \"...\""
        ),
        "{s}"
    );
}

#[test]
fn fora_de_um_agente_e_codigo_6() {
    let o = atlasoq(None).arg("list").output().unwrap();
    assert_eq!(o.status.code(), Some(6));
    assert!(errout(&o).starts_with("erro: o atlasoq só funciona dentro de um agente"));
}

#[test]
fn app_fechado_e_codigo_4() {
    let mut c = atlasoq(None);
    c.env("ATLASOQ_SOCKET", "/tmp/nao-existe.sock")
        .env("ATLASOQ_NODE_ID", "a")
        .env("ATLASOQ_NODE_TOKEN", "t");
    let o = c.arg("list").output().unwrap();
    assert_eq!(o.status.code(), Some(4));
}

#[test]
fn uso_incorreto_e_codigo_2() {
    for args in [
        vec!["send"],
        vec!["send", "coder"],
        vec!["read"],
        vec!["list", "x"],
        vec!["send", "--foo", "x", "y"],
    ] {
        let o = atlasoq(None).args(&args).write_stdin("").output().unwrap();
        assert_eq!(o.status.code(), Some(2), "{args:?}");
        assert!(errout(&o).starts_with("erro: "), "{args:?}");
    }
}

#[test]
fn comando_com_erro_de_digitacao_sugere_o_correto() {
    let o = atlasoq(None).args(["sned", "x", "y"]).output().unwrap();
    assert_eq!(o.status.code(), Some(2));
    assert_eq!(
        errout(&o),
        "erro: comando \"sned\" não existe.\n      Você quis dizer \"send\"?\n      uso: atlasoq send <agente> <texto>\n"
    );
    // Longe demais de qualquer comando: sem sugestão.
    let o = atlasoq(None).arg("xyzabc").output().unwrap();
    assert!(errout(&o).contains("atlasoq help"));
    assert!(!errout(&o).contains("Você quis dizer"));
}

#[test]
fn help_lista_os_8_comandos_e_detalha_um() {
    let o = atlasoq(None).arg("help").output().unwrap();
    assert_eq!(o.status.code(), Some(0));
    let s = out(&o);
    for c in [
        "learn", "list", "send", "read", "ask", "reply", "note", "help",
    ] {
        assert!(s.contains(&format!("  {c}")), "falta {c}: {s}");
    }
    let o = atlasoq(None).args(["help", "send"]).output().unwrap();
    assert!(out(&o).contains("uso: atlasoq send <agente> <texto>"));
    assert!(out(&o).contains("exemplo:"));
}
