use atlasoq_core::NodeName;
use atlasoq_store::validate::validate;
use atlasoq_store::*;

fn rect() -> Rect {
    Rect {
        x: 0.0,
        y: 0.0,
        w: 400.0,
        h: 300.0,
    }
}

fn agent(id: &str, name: &str) -> Node {
    Node::Agent(Agent {
        id: id.into(),
        name: NodeName::parse(name).unwrap(),
        role: String::new(),
        profile: "shell".into(),
        rect: rect(),
        terminal: Terminal {
            command: "bash".into(),
            args: vec![],
            cwd: ".".into(),
            env: Default::default(),
            env_secret_keys: vec![],
            cols: 100,
            rows: 30,
        },
        learn: LearnSettings::new(LearnMode::Off),
    })
}

fn note(id: &str, name: &str) -> Node {
    Node::Note(Note {
        id: id.into(),
        name: NodeName::parse(name).unwrap(),
        rect: rect(),
        file: format!("notes/{name}.md"),
    })
}

fn conn(id: &str, from: &str, to: &str) -> Connection {
    Connection {
        id: id.into(),
        from: from.into(),
        to: to.into(),
        direction: Direction::Forward,
        state: ConnState::Active,
        pause_reason: None,
        limits: ConnectionLimits::default(),
    }
}

fn base() -> Workspace {
    let mut ws = Workspace::new("ws_1".into(), "teste".into());
    ws.nodes = vec![
        agent("a_1", "planner"),
        agent("a_2", "coder"),
        note("n_1", "plano"),
        note("n_2", "outra"),
    ];
    ws.connections = vec![conn("c_1", "a_1", "a_2"), conn("c_2", "n_1", "a_2")];
    ws
}

#[test]
fn workspace_valido() {
    assert!(validate(&base()).is_empty(), "{:?}", validate(&base()));
}

#[test]
fn nomes_duplicados() {
    let mut ws = base();
    ws.nodes.push(agent("a_3", "coder"));
    assert!(validate(&ws).iter().any(|e| e.contains("nome repetido")));
}

#[test]
fn nome_invalido_nao_desserializa() {
    let json = r#"{"id":"a_9","kind":"agent","name":"Coder","profile":"x","rect":{"x":0,"y":0,"w":400,"h":300},
        "terminal":{"command":"bash","cwd":".","cols":100,"rows":30},"learn":{"mode":"off"}}"#;
    assert!(serde_json::from_str::<Node>(json).is_err());
}

#[test]
fn id_inexistente_e_auto_ligacao() {
    let mut ws = base();
    ws.connections.push(conn("c_3", "a_1", "a_99"));
    ws.connections.push(conn("c_4", "a_1", "a_1"));
    let errs = validate(&ws);
    assert!(errs.iter().any(|e| e.contains("inexistente")));
    assert!(errs.iter().any(|e| e.contains("ele mesmo")));
}

#[test]
fn nota_com_nota() {
    let mut ws = base();
    ws.connections.push(conn("c_5", "n_1", "n_2"));
    assert!(validate(&ws).iter().any(|e| e.contains("duas notas")));
}

#[test]
fn cwd_absoluto() {
    let mut ws = base();
    if let Node::Agent(a) = &mut ws.nodes[0] {
        a.terminal.cwd = "/home/x".into();
    }
    assert!(validate(&ws).iter().any(|e| e.contains("cwd")));
}

#[test]
fn segredo_com_valor_salvo() {
    let mut ws = base();
    if let Node::Agent(a) = &mut ws.nodes[0] {
        a.terminal.env.insert("API_KEY".into(), "123".into());
        a.terminal.env_secret_keys.push("API_KEY".into());
    }
    assert!(validate(&ws).iter().any(|e| e.contains("secreta")));
}

#[test]
fn zoom_e_limites_do_terminal() {
    let mut ws = base();
    ws.viewport.zoom = 9.0;
    if let Node::Agent(a) = &mut ws.nodes[0] {
        a.terminal.cols = 10;
    }
    let errs = validate(&ws);
    assert!(errs.iter().any(|e| e.contains("zoom")));
    assert!(errs.iter().any(|e| e.contains("colunas")));
}
