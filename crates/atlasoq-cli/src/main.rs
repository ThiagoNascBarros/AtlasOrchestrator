//! CLI `atlasoq`: interface dos agentes com o canvas (contracts/cli.md).

mod client;
mod commands;
mod suggest;
mod usage;

use std::io::{IsTerminal, Read};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let stdin_tty = std::io::stdin().is_terminal();
    let mut stdin = std::io::stdin();
    let code = commands::run(
        &args,
        &|k| std::env::var(k).ok(),
        &mut stdin as &mut dyn Read,
        stdin_tty,
        &mut std::io::stdout(),
        &mut std::io::stderr(),
    );
    std::process::exit(code);
}
