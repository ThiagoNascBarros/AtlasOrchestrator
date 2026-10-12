//! Diretórios do usuário (XDG) usados pelo app fora do projeto.

use std::path::PathBuf;

fn home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"))
}

fn xdg(var: &str, fallback: &str) -> PathBuf {
    std::env::var_os(var)
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| home().join(fallback))
}

/// `~/.config/atlasoq`
pub fn config_dir() -> PathBuf {
    xdg("XDG_CONFIG_HOME", ".config").join("atlasoq")
}

/// `~/.local/state/atlasoq`
pub fn state_dir() -> PathBuf {
    xdg("XDG_STATE_HOME", ".local/state").join("atlasoq")
}
