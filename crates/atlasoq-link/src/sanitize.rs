//! Sanitização de texto entre agentes: nada que um agente envie pode virar comando do terminal
//! de outro (sequências de escape, controles, Ctrl-C).

/// Tamanho máximo de uma mensagem, em bytes.
pub const MAX_TEXT_BYTES: usize = 16 * 1024;

/// Remove C0/C1 (exceto `\n` e `\t`), sequências ESC/CSI/OSC/DCS e normaliza quebras de linha.
pub fn sanitize(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut it = input.chars().peekable();
    while let Some(c) = it.next() {
        match c {
            '\u{1b}' => skip_escape(&mut it),
            '\r' => {
                if it.peek() == Some(&'\n') {
                    it.next();
                }
                out.push('\n');
            }
            '\n' | '\t' => out.push(c),
            c if (c as u32) < 0x20 || c as u32 == 0x7f || (0x80..=0x9f).contains(&(c as u32)) => {}
            c => out.push(c),
        }
    }
    out
}

/// Consome o resto de uma sequência iniciada por ESC.
fn skip_escape(it: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    match it.next() {
        // CSI: parâmetros e intermediários até um byte final 0x40–0x7E.
        Some('[') => {
            for c in it.by_ref() {
                if ('\u{40}'..='\u{7e}').contains(&c) {
                    break;
                }
            }
        }
        // OSC, DCS, SOS, PM, APC: até BEL ou ST (ESC \).
        Some(']' | 'P' | 'X' | '^' | '_') => {
            while let Some(c) = it.next() {
                if c == '\u{7}' {
                    break;
                }
                if c == '\u{1b}' {
                    if it.peek() == Some(&'\\') {
                        it.next();
                    }
                    break;
                }
            }
        }
        // Sequência de escape simples (ESC + um caractere; com intermediários 0x20–0x2F).
        Some(c) if ('\u{20}'..='\u{2f}').contains(&c) => {
            for c in it.by_ref() {
                if ('\u{30}'..='\u{7e}').contains(&c) {
                    break;
                }
            }
        }
        _ => {}
    }
}

/// Verdadeiro se o texto parece conter um segredo (chaves de API, tokens, chaves privadas).
pub fn looks_like_secret(text: &str) -> bool {
    const PREFIXES: [&str; 8] = [
        "sk-",
        "ghp_",
        "gho_",
        "github_pat_",
        "AKIA",
        "xoxb-",
        "xoxp-",
        "AIza",
    ];
    if text.contains("-----BEGIN") && text.contains("PRIVATE KEY") {
        return true;
    }
    let lower = text.to_lowercase();
    if ["password=", "senha=", "token=", "api_key=", "secret="]
        .iter()
        .any(|k| lower.contains(k))
    {
        return true;
    }
    text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '-'))
        .any(|w| w.len() >= 20 && PREFIXES.iter().any(|p| w.starts_with(p)))
}
