//! Sugestão de comando por distância de edição.

pub const COMMANDS: [&str; 8] = [
    "learn", "list", "send", "read", "ask", "reply", "note", "help",
];

/// Comando válido mais próximo (distância ≤ 2), se houver.
pub fn closest(word: &str) -> Option<&'static str> {
    COMMANDS
        .iter()
        .map(|c| (strsim::levenshtein(word, c), *c))
        .filter(|(d, _)| *d <= 2)
        .min_by_key(|(d, _)| *d)
        .map(|(_, c)| c)
}
