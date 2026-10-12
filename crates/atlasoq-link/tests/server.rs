mod common;

use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;

use atlasoq_link::Server;
use atlasoq_store::Direction;
use common::*;
use serde_json::{json, Value};

#[test]
fn socket_com_permissoes_restritas_e_uma_requisicao_por_conexao() {
    let host = FakeHost::new(vec![conn("c1", "a_p", "a_c", Direction::Forward)]);
    let b = broker(&host);
    let token = b.issue_token("a_p");
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("atlasoq").join("t.sock");
    let server = Server::start(b, path.clone()).unwrap();

    let mode = |p: &std::path::Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode(path.parent().unwrap()), 0o700);
    assert_eq!(mode(&path), 0o600);

    let mut s = UnixStream::connect(&path).unwrap();
    writeln!(
        s,
        "{}",
        request("a_p", &token, "send", json!({"to": "coder", "text": "oi"}))
    )
    .unwrap();
    let mut line = String::new();
    BufReader::new(&s).read_line(&mut line).unwrap();
    let r: Value = serde_json::from_str(&line).unwrap();
    assert_eq!(r["ok"], true, "{r}");
    assert!(host.text_for("a_c").contains("oi"));

    // Linha acima de 64 KiB é recusada sem derrubar o servidor.
    let mut s = UnixStream::connect(&path).unwrap();
    let _ = s.write_all(&vec![b'x'; 70 * 1024]);
    let _ = s.write_all(b"\n");
    let mut line = String::new();
    let _ = BufReader::new(&s).read_line(&mut line);
    assert!(line.contains("too_large"), "{line}");

    drop(server);
    assert!(!path.exists());
}
