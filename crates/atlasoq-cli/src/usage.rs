//! Textos de ajuda: `atlasoq help` e `atlasoq help <comando>`.

pub struct Command {
    pub name: &'static str,
    pub usage: &'static str,
    pub summary: &'static str,
    pub example: &'static str,
}

pub const COMMANDS: [Command; 8] = [
    Command {
        name: "learn",
        usage: "atlasoq learn",
        summary: "mostra o tutorial de como usar o atlasoq",
        example: "atlasoq learn",
    },
    Command {
        name: "list",
        usage: "atlasoq list",
        summary: "mostra com quem você pode falar e perguntas pendentes",
        example: "atlasoq list",
    },
    Command {
        name: "send",
        usage: "atlasoq send <agente> <texto>",
        summary: "envia uma mensagem e continua",
        example: "atlasoq send revisor \"revise o arquivo src/main.rs\"",
    },
    Command {
        name: "read",
        usage: "atlasoq read <agente> [--lines N]",
        summary: "lê o texto recente do terminal de um agente",
        example: "atlasoq read coder --lines 40",
    },
    Command {
        name: "ask",
        usage: "atlasoq ask <agente> <texto> [--timeout S]",
        summary: "pergunta e espera a resposta",
        example: "atlasoq ask coder \"quantos testes falharam?\"",
    },
    Command {
        name: "reply",
        usage: "atlasoq reply [agente] <texto>",
        summary: "responde à pergunta pendente",
        example: "atlasoq reply \"3 falharam, todos em parser\"",
    },
    Command {
        name: "note",
        usage: "atlasoq note <nota> [texto]",
        summary: "lê ou acrescenta numa nota (memory é a sua memória)",
        example: "atlasoq note plano \"feito: testes\"",
    },
    Command {
        name: "help",
        usage: "atlasoq help [comando]",
        summary: "mostra esta ajuda",
        example: "atlasoq help send",
    },
];

pub fn find(name: &str) -> Option<&'static Command> {
    COMMANDS.iter().find(|c| c.name == name)
}

pub fn overview() -> String {
    let mut out = String::from("uso: atlasoq <comando> [alvo] [texto]\n\ncomandos:\n");
    for c in &COMMANDS {
        out.push_str(&format!("  {:<6} {}\n", c.name, c.summary));
    }
    out.push_str("\nUse --json em qualquer comando para saída em JSON.\nDetalhes: atlasoq help <comando>\n");
    out
}

pub fn detail(c: &Command) -> String {
    format!(
        "uso: {}\n{}\nexemplo: {}\n",
        c.usage, c.summary, c.example
    )
}
