//! Conteúdo de aprendizado entregue a cada agente (contracts/learn-hook.md), na ordem: identidade,
//! ligações, tutorial, regras do time e memória. Limitado a 9.000 caracteres.

use std::fs;
use std::path::Path;

use atlasoq_store::{ConnState, Connection};

use crate::acl;
use crate::broker::NodeInfo;
use crate::sanitize::sanitize;

/// Limite total do conteúdo, em caracteres (abaixo do limite de 10.000 do SessionStart).
pub const MAX_CHARS: usize = 9_000;
/// Regras do time (`.atlasoq/learn.md`), em bytes.
pub const LEARN_MD_MAX: usize = 4 * 1024;
/// Anotações de memória entregues, em bytes e em quantidade.
pub const MEMORY_MAX: usize = 2 * 1024;
pub const MEMORY_MAX_ENTRIES: usize = 20;

/// Tutorial para agentes (docs/agentes/tutorial.md), embutido no binário.
pub const TUTORIAL: &str = include_str!("../../../docs/agentes/tutorial.md");

/// Arquivos que entram no conteúdo: regras do time e memória do agente.
#[derive(Debug, Clone, Default)]
pub struct LearnFiles {
    pub learn_md: Option<String>,
    pub memory: Option<String>,
}

impl LearnFiles {
    pub fn read(learn_md: &Path, memory: &Path) -> Self {
        Self {
            learn_md: fs::read_to_string(learn_md).ok(),
            memory: fs::read_to_string(memory).ok(),
        }
    }
}

/// Linhas com as ligações do agente, no mesmo sentido que `atlasoq list`.
pub fn links_lines(me: &NodeInfo, nodes: &[NodeInfo], conns: &[Connection]) -> Vec<String> {
    let lines: Vec<String> = nodes
        .iter()
        .filter(|n| n.id != me.id && !n.is_note)
        .filter_map(|n| {
            let c = conns.iter().find(|c| acl::links(c, &me.id, &n.id))?;
            let i_can = acl::direction_allows(c, &me.id, &n.id);
            let they_can = acl::direction_allows(c, &n.id, &me.id);
            let relation = match (i_can, they_can) {
                (true, true) => "vocês podem se enviar e perguntar",
                (true, false) => "você pode enviar e perguntar",
                _ => "pode te enviar e perguntar",
            };
            let state = match c.state {
                ConnState::Active => "",
                ConnState::Paused => " (pausada pelo usuário)",
                ConnState::AutoPaused => " (pausada automaticamente)",
            };
            Some(format!("- {}: {relation}{state}", n.name))
        })
        .collect();
    if lines.is_empty() {
        vec!["- nenhuma ligação ainda; peça ao usuário para ligar você a outro agente.".into()]
    } else {
        lines
    }
}

/// Últimas anotações da memória (linhas que começam com `- `), até o limite de entradas e bytes.
fn memory_block(text: &str) -> Option<String> {
    let entries: Vec<&str> = text.lines().filter(|l| l.starts_with("- ")).collect();
    if entries.is_empty() {
        return None;
    }
    let total = entries.len();
    let mut kept: Vec<&str> = Vec::new();
    let mut size = 0;
    for e in entries.iter().rev() {
        if kept.len() == MEMORY_MAX_ENTRIES || size + e.len() + 1 > MEMORY_MAX {
            break;
        }
        size += e.len() + 1;
        kept.push(e);
    }
    kept.reverse();
    let mut out = format!("Suas últimas anotações:\n{}", kept.join("\n"));
    let older = total - kept.len();
    if older > 0 {
        out.push_str(&format!(
            "\n(há {older} anotações mais antigas; leia com atlasoq note memory)"
        ));
    }
    Some(out)
}

/// Corta `s` no maior prefixo de no máximo `max` bytes, em fronteira de caractere.
fn cut_bytes(s: &str, max: usize) -> &str {
    if s.len() <= max {
        return s;
    }
    let mut end = max;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

/// Monta o texto de aprendizado de um agente.
pub fn build(
    name: &str,
    role: &str,
    links: &[String],
    learn_md: Option<&str>,
    memory: Option<&str>,
) -> String {
    let role = if role.trim().is_empty() {
        "não definido"
    } else {
        role.trim()
    };
    let mut out = format!("Você é o agente \"{name}\" no AtlasOQ. Papel: {role}.\n\n");
    out.push_str("Suas ligações agora (o mesmo que atlasoq list):\n");
    out.push_str(&links.join("\n"));
    out.push_str("\n\n");
    out.push_str(TUTORIAL.trim_end());
    out.push_str("\n\n");
    if let Some(md) = learn_md.map(str::trim).filter(|m| !m.is_empty()) {
        out.push_str("Regras do time:\n");
        out.push_str(cut_bytes(md, LEARN_MD_MAX));
        out.push_str("\n\n");
    }
    if let Some(block) = memory.and_then(memory_block) {
        out.push_str(&block);
        out.push('\n');
    }
    sanitize(&out).chars().take(MAX_CHARS).collect()
}

/// Texto de aprendizado de `me`, a partir do workspace já carregado.
pub fn text_for(
    me: &NodeInfo,
    nodes: &[NodeInfo],
    conns: &[Connection],
    files: &LearnFiles,
) -> String {
    build(
        &me.name,
        &me.role,
        &links_lines(me, nodes, conns),
        files.learn_md.as_deref(),
        files.memory.as_deref(),
    )
}
