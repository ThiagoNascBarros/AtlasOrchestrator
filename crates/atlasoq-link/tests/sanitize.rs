use atlasoq_link::sanitize::{looks_like_secret, sanitize};

#[test]
fn remove_sequencias_de_escape_e_controles() {
    let payloads = [
        "\x1b[31mvermelho\x1b[0m",
        "\x1b]0;título\x07texto",
        "\x1b]8;;http://x\x1b\\link\x1b]8;;\x1b\\",
        "\x1bPq#0;2;0;0;0\x1b\\depois",
        "\x1b[200~colado\x1b[201~",
        "\x03\x04\x07\x08\x0b\x0c\x0e\x7f",
        "\u{9b}31m\u{90}dcs\u{9c}",
        "\x1b(Bdepois",
        "\x1b",
    ];
    for p in payloads {
        let out = sanitize(p);
        assert!(
            !out.chars().any(|c| (c as u32) < 0x20 && c != '\n' && c != '\t'),
            "controle sobrou em {p:?} → {out:?}"
        );
        assert!(!out.contains('\x1b') && !out.contains('\x7f'), "{p:?} → {out:?}");
        assert!(!out.chars().any(|c| (0x80..=0x9f).contains(&(c as u32))), "{p:?}");
    }
}

#[test]
fn preserva_texto_comum_e_normaliza_quebras() {
    assert_eq!(sanitize("olá, mundo — ação\tok"), "olá, mundo — ação\tok");
    assert_eq!(sanitize("a\r\nb\rc\nd"), "a\nb\nc\nd");
    assert_eq!(sanitize("\x1b[31mvermelho\x1b[0m!"), "vermelho!");
    assert_eq!(sanitize("\x1b]0;título\x07texto"), "texto");
}

#[test]
fn detecta_segredos_comuns() {
    assert!(looks_like_secret("minha chave é sk-abcdefghijklmnopqrstuvwxyz123456"));
    assert!(looks_like_secret("ghp_abcdefghijklmnopqrstuvwxyz0123456789"));
    assert!(looks_like_secret("-----BEGIN RSA PRIVATE KEY-----"));
    assert!(looks_like_secret("password=hunter2"));
    assert!(!looks_like_secret("feito: migração do banco concluída"));
}
