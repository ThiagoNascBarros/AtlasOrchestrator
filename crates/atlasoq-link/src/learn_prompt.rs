//! Modo `prompt`: o texto de aprendizado vai como argumento inicial da CLI (ex.: `codex "<texto>"`).

/// Argumentos extras para a CLI. Com `flag` (ex.: `--prompt`), o texto segue a flag; sem ela, é
/// posicional.
pub fn args(flag: Option<&str>, text: &str) -> Vec<String> {
    match flag.filter(|f| !f.is_empty()) {
        Some(f) => vec![f.to_owned(), text.to_owned()],
        None => vec![text.to_owned()],
    }
}
