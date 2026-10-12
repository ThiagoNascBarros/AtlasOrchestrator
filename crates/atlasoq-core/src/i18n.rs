//! Textos pt-BR mostrados a agentes e usuários. Toda mensagem de erro traz o próximo passo
//! (constituição, princípio VIII).

/// Mensagem de erro com o próximo passo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Msg {
    pub message: String,
    pub hint: String,
}

fn msg(message: impl Into<String>, hint: impl Into<String>) -> Msg {
    Msg {
        message: message.into(),
        hint: hint.into(),
    }
}

fn list_or_none(names: &[String]) -> String {
    if names.is_empty() {
        "ninguém".to_owned()
    } else {
        names.join(", ")
    }
}

pub fn unknown_target(name: &str, available: &[String]) -> Msg {
    msg(
        format!("não existe agente ou nota chamado \"{name}\"."),
        format!(
            "Nomes disponíveis para você: {}. Veja com: atlasoq list",
            list_or_none(available)
        ),
    )
}

pub fn no_connection(me: &str, target: &str, can_send_to: &[String]) -> Msg {
    msg(
        format!("não existe conexão de \"{me}\" para \"{target}\"."),
        format!(
            "Você só pode enviar para: {}. Para falar com \"{target}\", peça ao usuário para conectar {me} → {target} no canvas.",
            list_or_none(can_send_to)
        ),
    )
}

pub fn connection_paused(me: &str, target: &str, reason: Option<&str>) -> Msg {
    let why = match reason {
        Some("max_exchanges") => " automaticamente: muitas trocas seguidas sem intervenção humana",
        Some("rate_limit") => " automaticamente: mensagens demais por minuto",
        Some("ping_pong") => " automaticamente: vai-e-vem repetitivo",
        _ => " pelo usuário",
    };
    msg(
        format!("a conexão entre \"{me}\" e \"{target}\" está pausada{why}."),
        "Só o usuário pode reativá-la no canvas. Avise-o no seu próprio terminal se precisar.",
    )
}

pub fn target_exited(target: &str) -> Msg {
    msg(
        format!("o agente \"{target}\" está encerrado."),
        "Peça ao usuário para reiniciá-lo no canvas.",
    )
}

pub fn too_large(limit_bytes: usize) -> Msg {
    msg(
        format!("a mensagem passa do limite de {} KiB.", limit_bytes / 1024),
        "Resuma o texto ou coloque o conteúdo longo numa nota e envie só o nome dela.",
    )
}

pub fn empty_text() -> Msg {
    msg(
        "o texto está vazio.",
        "Informe o texto como argumento ou pela entrada padrão.",
    )
}

pub fn no_pending_ask() -> Msg {
    msg(
        "não há pergunta para responder.",
        "Para avisar alguém, use: atlasoq send <agente> \"...\"",
    )
}

pub fn ask_expired(asker: &str) -> Msg {
    msg(
        format!("a pergunta de \"{asker}\" expirou."),
        format!("Se ainda quiser avisá-lo, use: atlasoq send {asker} \"...\""),
    )
}

pub fn ambiguous_reply(askers: &[String]) -> Msg {
    msg(
        format!(
            "há perguntas pendentes de mais de um agente: {}.",
            askers.join(", ")
        ),
        "Indique a quem responde, por exemplo: atlasoq reply <agente> \"...\"",
    )
}

pub fn ask_timeout(target: &str, secs: u64) -> Msg {
    msg(
        format!("sem resposta de \"{target}\" em {secs}s; tela recente abaixo."),
        "Tente de novo mais tarde ou leia a tela com: atlasoq read <agente>",
    )
}

pub fn unauthorized() -> Msg {
    msg(
        "esta requisição não veio de um agente do AtlasOQ.",
        "O atlasoq só funciona dentro de um terminal de agente criado no canvas.",
    )
}

pub fn outside_app() -> Msg {
    msg(
        "o atlasoq só funciona dentro de um agente do AtlasOQ.",
        "Abra o AtlasOQ e use o terminal de um agente do canvas.",
    )
}

pub fn app_unreachable() -> Msg {
    msg(
        "não foi possível falar com o AtlasOQ (o app foi fechado?).",
        "Verifique se o app está aberto; se ele foi reiniciado, este agente também será.",
    )
}

pub fn secret_detected() -> Msg {
    msg(
        "o texto parece conter um segredo (chave, token ou senha).",
        "Nunca registre segredos. Reescreva sem o valor sensível.",
    )
}

pub fn missing_prefix() -> Msg {
    msg(
        "anotações de memória precisam começar com feito:, decisão: ou pendente:.",
        "Exemplo: atlasoq note memory \"feito: validação de login com testes\"",
    )
}

pub fn memory_too_long(limit: usize) -> Msg {
    msg(
        format!("a anotação passa de {limit} caracteres."),
        "Resuma em uma frase curta; detalhes longos vão numa nota.",
    )
}

pub fn unsupported_version(v: u32) -> Msg {
    msg(
        format!("versão de protocolo {v} não suportada por este AtlasOQ."),
        "Use a CLI atlasoq instalada junto com o app.",
    )
}

pub fn bad_request(detail: &str) -> Msg {
    msg(
        format!("requisição inválida: {detail}."),
        "Veja o uso com: atlasoq help",
    )
}

pub fn wrong_direction(me: &str, target: &str) -> Msg {
    msg(
        format!("a conexão entre \"{me}\" e \"{target}\" não permite que você inicie a conversa."),
        format!("Só \"{target}\" pode falar com você por ela. Peça ao usuário para mudar a direção no canvas."),
    )
}

pub fn note_read_only(me: &str, note: &str) -> Msg {
    msg(
        format!("você só pode ler a nota \"{note}\"."),
        format!("Para acrescentar texto, peça ao usuário para conectar {me} → {note} no canvas."),
    )
}

pub fn memory_of_other() -> Msg {
    msg(
        "a memória de cada agente é privada.",
        "Use atlasoq note memory para a sua própria memória.",
    )
}
