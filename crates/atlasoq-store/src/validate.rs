//! Validação do `workspace.json` na leitura (contracts/workspace-format.md).

use std::collections::HashSet;
use std::path::Path;

use crate::model::*;

/// Devolve todos os problemas encontrados; vazio significa válido.
pub fn validate(ws: &Workspace) -> Vec<String> {
    let mut errors = Vec::new();

    if ws.schema != SCHEMA_VERSION {
        errors.push(format!(
            "schema {} não suportado (esperado {SCHEMA_VERSION})",
            ws.schema
        ));
    }
    if !(ZOOM_MIN..=ZOOM_MAX).contains(&ws.viewport.zoom) {
        errors.push(format!(
            "zoom {} fora de {ZOOM_MIN}–{ZOOM_MAX}",
            ws.viewport.zoom
        ));
    }

    let mut ids = HashSet::new();
    let mut names = HashSet::new();
    for node in &ws.nodes {
        if !ids.insert(node.id()) {
            errors.push(format!("id de nó repetido: {}", node.id()));
        }
        if !names.insert(node.name().as_str()) {
            errors.push(format!("nome repetido: {}", node.name()));
        }
        let r = node.rect();
        if r.w < NODE_MIN_W || r.h < NODE_MIN_H {
            errors.push(format!(
                "{}: tamanho mínimo é {NODE_MIN_W}×{NODE_MIN_H}",
                node.name()
            ));
        }
        match node {
            Node::Agent(a) => validate_agent(a, &mut errors),
            Node::Note(n) => {
                if Path::new(&n.file).is_absolute() || n.file.split('/').any(|p| p == "..") {
                    errors.push(format!(
                        "{}: caminho da nota deve ficar dentro de .atlasoq/",
                        n.name
                    ));
                }
            }
        }
    }

    let mut conn_ids = HashSet::new();
    for c in &ws.connections {
        if !conn_ids.insert(c.id.as_str()) {
            errors.push(format!("id de conexão repetido: {}", c.id));
        }
        let (from, to) = (ws.node(&c.from), ws.node(&c.to));
        match (from, to) {
            (None, _) | (_, None) => {
                errors.push(format!("conexão {} aponta para nó inexistente", c.id));
            }
            (Some(f), Some(t)) => {
                if c.from == c.to {
                    errors.push(format!("conexão {} liga um nó a ele mesmo", c.id));
                }
                if f.is_note() && t.is_note() {
                    errors.push(format!("conexão {} liga duas notas", c.id));
                }
            }
        }
        if c.state == ConnState::AutoPaused && c.pause_reason.is_none() {
            errors.push(format!(
                "conexão {} pausada automaticamente sem motivo",
                c.id
            ));
        }
    }
    errors
}

fn validate_agent(a: &Agent, errors: &mut Vec<String>) {
    if a.role.chars().count() > ROLE_MAX_CHARS {
        errors.push(format!(
            "{}: papel com mais de {ROLE_MAX_CHARS} caracteres",
            a.name
        ));
    }
    if a.terminal.command.trim().is_empty() {
        errors.push(format!("{}: comando vazio", a.name));
    }
    if Path::new(&a.terminal.cwd).is_absolute() {
        errors.push(format!(
            "{}: cwd deve ser relativo à raiz do workspace",
            a.name
        ));
    }
    let (cmin, cmax) = COLS_RANGE;
    let (rmin, rmax) = ROWS_RANGE;
    if !(cmin..=cmax).contains(&a.terminal.cols) || !(rmin..=rmax).contains(&a.terminal.rows) {
        errors.push(format!(
            "{}: terminal fora de {cmin}–{cmax} colunas e {rmin}–{rmax} linhas",
            a.name
        ));
    }
    for key in &a.terminal.env_secret_keys {
        if a.terminal.env.contains_key(key) {
            errors.push(format!(
                "{}: a variável secreta {key} não pode ter valor salvo",
                a.name
            ));
        }
    }
}
