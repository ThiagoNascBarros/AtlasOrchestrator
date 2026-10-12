//! Permissões a partir das conexões: quem pode agir sobre quem (data-model.md).

use atlasoq_store::{ConnState, Connection, Direction, PauseReason};

/// Resultado negativo da verificação, com o que o handler precisa para montar a mensagem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Denied {
    NoConnection,
    WrongDirection,
    Paused(Option<PauseReason>),
}

/// `actor` pode agir (enviar, perguntar, ler) sobre `target` por esta conexão?
pub fn direction_allows(conn: &Connection, actor: &str, target: &str) -> bool {
    match conn.direction {
        Direction::Both => true,
        Direction::Forward => conn.from == actor && conn.to == target,
        Direction::Backward => conn.to == actor && conn.from == target,
    }
}

/// A conexão liga exatamente esse par de nós (em qualquer sentido)?
pub fn links(conn: &Connection, a: &str, b: &str) -> bool {
    (conn.from == a && conn.to == b) || (conn.from == b && conn.to == a)
}

/// Verifica se `actor` pode agir sobre `target`; devolve a conexão que autoriza.
pub fn authorize<'a>(
    conns: &'a [Connection],
    actor: &str,
    target: &str,
) -> Result<&'a Connection, Denied> {
    let pair: Vec<&Connection> = conns.iter().filter(|c| links(c, actor, target)).collect();
    if pair.is_empty() {
        return Err(Denied::NoConnection);
    }
    let Some(conn) = pair
        .iter()
        .find(|c| direction_allows(c, actor, target))
        .copied()
    else {
        return Err(Denied::WrongDirection);
    };
    match conn.state {
        ConnState::Active => Ok(conn),
        ConnState::Paused => Err(Denied::Paused(None)),
        ConnState::AutoPaused => Err(Denied::Paused(conn.pause_reason)),
    }
}
