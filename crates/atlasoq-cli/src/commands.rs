//! Interpretação dos comandos, chamada ao app e formatação da saída.

use std::io::{Read, Write};

use atlasoq_core::protocol::{ENV_NODE_ID, ENV_NODE_TOKEN, ENV_SOCKET};
use atlasoq_core::{i18n, ErrorKind, ExitCode, Response};
use serde_json::{json, Value};

use crate::client::{self, Env};
use crate::{suggest, usage};

/// Tutorial genérico, mostrado fora de um agente do app (contracts/cli.md).
const GENERIC_TUTORIAL: &str = include_str!("../../../docs/agentes/tutorial.md");

const OK: i32 = 0;
const USAGE: i32 = 2;
const UNAVAILABLE: i32 = 4;
const OUTSIDE_APP: i32 = 6;

/// Argumentos já separados de flags.
struct Parsed {
    positional: Vec<String>,
    json: bool,
    lines: Option<u32>,
    timeout: Option<u64>,
    hook: bool,
}

fn parse(args: &[String]) -> Result<Parsed, String> {
    let mut p = Parsed {
        positional: Vec::new(),
        json: false,
        lines: None,
        timeout: None,
        hook: false,
    };
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--json" => p.json = true,
            "--hook" => p.hook = true,
            "--lines" | "--timeout" => {
                let v = it
                    .next()
                    .ok_or_else(|| format!("a opção {a} precisa de um número"))?;
                let n: u64 = v
                    .parse()
                    .map_err(|_| format!("a opção {a} precisa de um número, recebeu \"{v}\""))?;
                if a == "--lines" {
                    p.lines = Some(n.min(u64::from(u32::MAX)) as u32);
                } else {
                    p.timeout = Some(n);
                }
            }
            flag if flag.starts_with("--") => return Err(format!("opção desconhecida: {flag}")),
            _ => p.positional.push(a.clone()),
        }
    }
    Ok(p)
}

fn print_error(err: &mut dyn Write, message: &str, hint: &str) {
    let _ = writeln!(err, "erro: {message}");
    if !hint.is_empty() {
        let _ = writeln!(err, "      {hint}");
    }
}

fn usage_error(err: &mut dyn Write, message: &str, cmd: &str) -> i32 {
    let hint = usage::find(cmd).map_or_else(String::new, |c| format!("uso: {}", c.usage));
    print_error(err, message, &hint);
    USAGE
}

/// Texto do comando: argumentos unidos por espaço ou, sem eles, a entrada padrão (se não for
/// um terminal).
fn text_from(words: &[String], stdin: &mut dyn Read, stdin_tty: bool) -> Option<String> {
    if !words.is_empty() {
        return Some(words.join(" "));
    }
    if stdin_tty {
        return None;
    }
    let mut buf = String::new();
    stdin.read_to_string(&mut buf).ok()?;
    let text = buf.trim_end_matches(['\n', '\r']).to_owned();
    (!text.is_empty()).then_some(text)
}

pub fn run(
    args: &[String],
    env: &dyn Fn(&str) -> Option<String>,
    stdin: &mut dyn Read,
    stdin_tty: bool,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> i32 {
    let Some(cmd) = args.first().map(String::as_str) else {
        let _ = write!(out, "{}", usage::overview());
        return USAGE;
    };
    if matches!(cmd, "-h" | "--help") {
        let _ = write!(out, "{}", usage::overview());
        return OK;
    }
    let parsed = match parse(&args[1..]) {
        Ok(p) => p,
        Err(e) => return usage_error(err, &e, cmd),
    };
    let rest = &parsed.positional;

    let Some(command) = usage::find(cmd) else {
        let hint = match suggest::closest(cmd) {
            Some(c) => format!(
                "Você quis dizer \"{c}\"?\n      uso: {}",
                usage::find(c).map_or("", |c| c.usage)
            ),
            None => "Veja os comandos com: atlasoq help".to_owned(),
        };
        print_error(err, &format!("comando \"{cmd}\" não existe."), &hint);
        return USAGE;
    };

    // `help` e `learn` não precisam do app.
    match command.name {
        "help" => {
            return match rest.first() {
                None => {
                    let _ = write!(out, "{}", usage::overview());
                    OK
                }
                Some(name) => match usage::find(name) {
                    Some(c) => {
                        let _ = write!(out, "{}", usage::detail(c));
                        OK
                    }
                    None => {
                        let hint = suggest::closest(name)
                            .map_or("Veja os comandos com: atlasoq help".to_owned(), |c| {
                                format!("Você quis dizer \"{c}\"?")
                            });
                        print_error(err, &format!("comando \"{name}\" não existe."), &hint);
                        USAGE
                    }
                },
            };
        }
        "learn" => {
            // Dentro do app, o texto é o de aprendizado do agente; fora, o tutorial genérico.
            let from_app = match (env(ENV_SOCKET), env(ENV_NODE_ID), env(ENV_NODE_TOKEN)) {
                (Some(socket), Some(node), Some(token)) => {
                    let env = Env {
                        socket,
                        node,
                        token,
                    };
                    client::call(&env, "learn", json!({ "hook": parsed.hook }))
                        .ok()
                        .filter(|r| r.ok)
                        .and_then(|r| r.data)
                        .and_then(|d| d["text"].as_str().map(str::to_owned))
                }
                _ => None,
            };
            let _ = write!(out, "{}", from_app.as_deref().unwrap_or(GENERIC_TUTORIAL));
            return OK;
        }
        _ => {}
    }

    let (op, args_json) = match build_request(command.name, rest, &parsed, stdin, stdin_tty) {
        Ok(r) => r,
        Err(msg) => return usage_error(err, &msg, command.name),
    };

    let (Some(socket), Some(node), Some(token)) =
        (env(ENV_SOCKET), env(ENV_NODE_ID), env(ENV_NODE_TOKEN))
    else {
        let m = i18n::outside_app();
        print_error(err, &m.message, &m.hint);
        return OUTSIDE_APP;
    };
    let env = Env {
        socket,
        node,
        token,
    };
    let Ok(resp) = client::call(&env, op, args_json) else {
        let m = i18n::app_unreachable();
        print_error(err, &m.message, &m.hint);
        return UNAVAILABLE;
    };
    render(command.name, rest, &parsed, &resp, out, err)
}

fn build_request(
    name: &str,
    rest: &[String],
    p: &Parsed,
    stdin: &mut dyn Read,
    stdin_tty: bool,
) -> Result<(&'static str, Value), String> {
    let need_text = |words: &[String], stdin: &mut dyn Read| {
        text_from(words, stdin, stdin_tty)
            .ok_or_else(|| "falta o texto: passe como argumento ou pela entrada padrão.".to_owned())
    };
    Ok(match name {
        "list" => {
            if !rest.is_empty() {
                return Err("o comando list não recebe argumentos.".into());
            }
            ("list", json!({}))
        }
        "send" => {
            let to = rest.first().ok_or("falta o nome do agente.")?;
            let text = need_text(&rest[1..], stdin)?;
            ("send", json!({ "to": to, "text": text }))
        }
        "read" => {
            let from = rest.first().ok_or("falta o nome do agente.")?;
            if rest.len() > 1 {
                return Err("o comando read recebe só o nome do agente.".into());
            }
            let lines = p
                .lines
                .unwrap_or(atlasoq_core::protocol::DEFAULT_READ_LINES);
            ("read", json!({ "from": from, "lines": lines }))
        }
        "ask" => {
            let to = rest.first().ok_or("falta o nome do agente.")?;
            let text = need_text(&rest[1..], stdin)?;
            let timeout = p
                .timeout
                .unwrap_or(atlasoq_core::protocol::DEFAULT_ASK_TIMEOUT_S);
            (
                "ask",
                json!({ "to": to, "text": text, "timeout_s": timeout }),
            )
        }
        "reply" => {
            // Com mais de uma palavra, a primeira pode ser o nome do perguntador; o app decide.
            let (to, words) = match rest.split_first() {
                Some((first, tail)) if !tail.is_empty() => (Some(first.clone()), tail.to_vec()),
                _ => (None, rest.to_vec()),
            };
            let text = if words.is_empty() && to.is_none() {
                need_text(&[], stdin)?
            } else {
                words.join(" ")
            };
            ("reply", json!({ "to": to, "text": text }))
        }
        "note" => {
            let note = rest.first().ok_or("falta o nome da nota.")?;
            if rest.len() > 1 {
                (
                    "note_append",
                    json!({ "note": note, "text": rest[1..].join(" ") }),
                )
            } else if !stdin_tty {
                match text_from(&[], stdin, stdin_tty) {
                    Some(t) => ("note_append", json!({ "note": note, "text": t })),
                    None => ("note_read", json!({ "note": note })),
                }
            } else {
                ("note_read", json!({ "note": note }))
            }
        }
        _ => return Err("comando desconhecido.".into()),
    })
}

fn render(
    name: &str,
    rest: &[String],
    p: &Parsed,
    resp: &Response,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> i32 {
    if p.json {
        let _ = writeln!(out, "{}", serde_json::to_string(resp).unwrap_or_default());
        return resp.code.unwrap_or(OK);
    }
    if !resp.ok {
        let code = resp.code.unwrap_or(i32::from(ExitCode::Usage));
        if resp.error == Some(ErrorKind::Timeout) {
            let who = rest.first().map_or("", String::as_str);
            let secs = p
                .timeout
                .unwrap_or(atlasoq_core::protocol::DEFAULT_ASK_TIMEOUT_S);
            let _ = writeln!(out, "sem resposta de {who} em {secs}s; tela recente:");
            let screen = resp
                .data
                .as_ref()
                .and_then(|d| d.get("screen"))
                .and_then(Value::as_str)
                .unwrap_or("");
            let _ = writeln!(out, "{screen}");
            return code;
        }
        print_error(
            err,
            resp.message.as_deref().unwrap_or("falha desconhecida."),
            resp.hint.as_deref().unwrap_or(""),
        );
        return code;
    }
    let data = resp.data.clone().unwrap_or(Value::Null);
    match name {
        "send" => {
            let _ = writeln!(
                out,
                "enviado para {} (msg {})",
                data["to"].as_str().unwrap_or(""),
                data["msg_id"]
            );
        }
        "read" | "note" => {
            let _ = writeln!(out, "{}", data["text"].as_str().unwrap_or(""));
        }
        "ask" => {
            let _ = writeln!(out, "{}", data["text"].as_str().unwrap_or(""));
        }
        "reply" => {
            let _ = writeln!(
                out,
                "resposta enviada para {}",
                data["to"].as_str().unwrap_or("")
            );
        }
        "list" => render_list(&data, out),
        _ => {}
    }
    OK
}

fn pad(s: &str, width: usize) -> String {
    let n = s.chars().count();
    format!("{s}{}", " ".repeat(width.saturating_sub(n)))
}

fn render_list(d: &Value, out: &mut dyn Write) {
    let you = &d["you"];
    let role = you["role"].as_str().filter(|r| !r.is_empty());
    let _ = writeln!(
        out,
        "você: {}{} ({})",
        you["name"].as_str().unwrap_or(""),
        role.map_or(String::new(), |r| format!(" — {r}")),
        you["status"].as_str().unwrap_or("")
    );
    let section = |out: &mut dyn Write, title: &str, items: &Value, note: bool| {
        let _ = writeln!(out, "{title}:");
        let items = items.as_array().cloned().unwrap_or_default();
        if items.is_empty() {
            let _ = writeln!(out, "  (nenhum conectado a você)");
        }
        for it in items {
            let text = match (note, it["relation"].as_str().unwrap_or("")) {
                (true, "both") => "↔ ler e acrescentar",
                (true, "outbound") => "→ você pode acrescentar",
                (true, _) => "← somente leitura",
                (false, "both") => "↔ vocês podem se enviar e perguntar",
                (false, "outbound") => "→ você pode enviar e perguntar",
                (false, _) => "← pode te enviar e perguntar",
            };
            let state = match it["state"].as_str().unwrap_or("active") {
                "paused" => "  [pausada pelo usuário]",
                "auto_paused" => "  [pausada automaticamente]",
                _ => "",
            };
            let status = if note {
                String::new()
            } else {
                it["status"].as_str().unwrap_or("").to_owned()
            };
            let _ = writeln!(
                out,
                "  {}{}{}{state}",
                pad(it["name"].as_str().unwrap_or(""), 10),
                pad(text, 38),
                status
            );
        }
    };
    section(out, "agentes", &d["agents"], false);
    if d["notes"].as_array().is_some_and(|n| !n.is_empty()) {
        section(out, "notas", &d["notes"], true);
    }
    let pending = d["pending"].as_array().cloned().unwrap_or_default();
    if !pending.is_empty() {
        let from: Vec<&str> = pending.iter().filter_map(|p| p["from"].as_str()).collect();
        let _ = writeln!(
            out,
            "perguntas pendentes para você: {} (de {}) — responda com: atlasoq reply \"...\"",
            pending.len(),
            from.join(", ")
        );
    }
}
