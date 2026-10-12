mod common;

use atlasoq_core::ErrorKind;
use atlasoq_link::auth::{constant_time_eq, new_token, peer_allowed};
use atlasoq_store::Direction;
use common::*;
use serde_json::json;

#[test]
fn uid_diferente_ou_desconhecido_e_recusado() {
    assert!(peer_allowed(Some(1000), 1000));
    assert!(!peer_allowed(Some(1001), 1000));
    assert!(!peer_allowed(None, 1000));
}

#[test]
fn token_tem_256_bits_e_nao_repete() {
    let (a, b) = (new_token(), new_token());
    assert_eq!(a.len(), 43, "32 bytes em base64url sem preenchimento");
    assert_ne!(a, b);
    assert!(a
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
    assert!(constant_time_eq(a.as_bytes(), a.as_bytes()));
    assert!(!constant_time_eq(a.as_bytes(), b.as_bytes()));
}

#[test]
fn token_errado_no_inexistente_e_versao_nao_suportada() {
    let host = FakeHost::new(vec![conn("c1", "a_p", "a_c", Direction::Forward)]);
    let b = broker(&host);
    let token = b.issue_token("a_p");

    let ok = call(&b, "a_p", &token, "list", json!({}));
    assert!(ok.ok);

    let r = call(&b, "a_p", "token-errado", "list", json!({}));
    assert_eq!(r.error, Some(ErrorKind::Unauthorized));
    assert_eq!(r.code, Some(6));

    let r = call(&b, "a_x", &token, "list", json!({}));
    assert_eq!(r.error, Some(ErrorKind::Unauthorized));

    // O token de um agente não vale para outro.
    let r = call(&b, "a_c", &token, "list", json!({}));
    assert_eq!(r.error, Some(ErrorKind::Unauthorized));

    let r = b.handle(
        &json!({"v": 2, "op": "list", "node": "a_p", "token": token, "args": {}}).to_string(),
    );
    assert_eq!(r.error, Some(ErrorKind::UnsupportedVersion));

    let r = b.handle("isto não é json");
    assert_eq!(r.error, Some(ErrorKind::BadRequest));

    // Novo token invalida o anterior (agente reiniciado).
    let novo = b.issue_token("a_p");
    assert_eq!(
        call(&b, "a_p", &token, "list", json!({})).error,
        Some(ErrorKind::Unauthorized)
    );
    assert!(call(&b, "a_p", &novo, "list", json!({})).ok);
}
