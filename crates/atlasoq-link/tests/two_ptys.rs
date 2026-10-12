//! Dois PTYs reais (bash): a mensagem chega no terminal do destino com o cabeçalho e é executada
//! como uma linha digitada; o sentido inverso é recusado.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use atlasoq_core::{ErrorKind, Request, Response};
use atlasoq_link::{Broker, Host, LinkEvent, NodeInfo, SessionInfo};
use atlasoq_pty::{Activity, LocalSessionHost, SessionHost, SessionSpec};
use atlasoq_store::{ConnState, Connection, ConnectionLimits, Direction, PauseReason};
use serde_json::json;

struct PtyHost {
    host: LocalSessionHost,
    sessions: HashMap<String, u64>,
    conns: Mutex<Vec<Connection>>,
}

impl Host for PtyHost {
    fn nodes(&self) -> Vec<NodeInfo> {
        ["planner", "coder"]
            .iter()
            .map(|n| NodeInfo {
                id: format!("a_{n}"),
                name: (*n).into(),
                role: String::new(),
                is_note: false,
            })
            .collect()
    }
    fn connections(&self) -> Vec<Connection> {
        self.conns.lock().unwrap().clone()
    }
    fn session(&self, id: &str) -> Option<SessionInfo> {
        let sid = *self.sessions.get(id)?;
        let exited = matches!(self.host.activity(sid), Ok(Activity::Exited(_)));
        Some(SessionInfo {
            status: if exited { "exited" } else { "idle" },
            exited,
            bracketed_paste: self.host.bracketed_paste(sid).unwrap_or(false),
            last_user_input: None,
        })
    }
    fn write(&self, id: &str, data: &[u8]) -> Result<(), String> {
        self.host
            .write(self.sessions[id], data)
            .map_err(|e| e.to_string())
    }
    fn screen_text(&self, id: &str, lines: usize) -> Result<String, String> {
        self.host
            .screen_text(self.sessions[id], lines)
            .map_err(|e| e.to_string())
    }
    fn set_connection_state(&self, id: &str, state: ConnState, reason: Option<PauseReason>) {
        for c in self.conns.lock().unwrap().iter_mut().filter(|c| c.id == id) {
            c.state = state;
            c.pause_reason = reason;
        }
    }
    fn logs_dir(&self) -> Option<PathBuf> {
        None
    }
    fn emit(&self, _: LinkEvent) {}
}

fn call(b: &Broker, node: &str, token: &str, op: &str, args: serde_json::Value) -> Response {
    let line = json!({"v": 1, "op": op, "node": node, "token": token, "args": args}).to_string();
    let _: Request = serde_json::from_str(&line).unwrap();
    b.handle(&line)
}

fn wait_for(host: &PtyHost, id: &str, needle: &str) -> bool {
    let end = Instant::now() + Duration::from_secs(5);
    while Instant::now() < end {
        if host
            .screen_text(id, 50)
            .unwrap_or_default()
            .contains(needle)
        {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    false
}

#[test]
fn mensagem_entre_dois_terminais_reais() {
    let local = LocalSessionHost::default();
    let mut sessions = HashMap::new();
    for n in ["planner", "coder"] {
        let mut spec = SessionSpec::new("bash", std::env::temp_dir());
        spec.args = vec!["--norc".into(), "--noprofile".into()];
        sessions.insert(format!("a_{n}"), local.spawn(spec).unwrap());
    }
    let host = Arc::new(PtyHost {
        host: local,
        sessions,
        conns: Mutex::new(vec![Connection {
            id: "c1".into(),
            from: "a_planner".into(),
            to: "a_coder".into(),
            direction: Direction::Forward,
            state: ConnState::Active,
            pause_reason: None,
            limits: ConnectionLimits::default(),
        }]),
    });
    let b = Arc::new(Broker::new(host.clone()));
    let (tp, tc) = (b.issue_token("a_planner"), b.issue_token("a_coder"));

    // O bash executa a linha entregue; o `echo` prova que ela chegou inteira e foi submetida.
    let r = call(
        &b,
        "a_planner",
        &tp,
        "send",
        json!({"to": "coder", "text": "echo OLA-DO-PLANNER"}),
    );
    assert!(r.ok, "{r:?}");
    assert!(
        wait_for(&host, "a_coder", "OLA-DO-PLANNER"),
        "{}",
        host.screen_text("a_coder", 50).unwrap()
    );

    let r = call(
        &b,
        "a_planner",
        &tp,
        "read",
        json!({"from": "coder", "lines": 20}),
    );
    assert!(r.data.unwrap()["text"]
        .as_str()
        .unwrap()
        .contains("atlasoq de:planner"));

    // Sentido inverso é recusado e nada chega no planner.
    let r = call(
        &b,
        "a_coder",
        &tc,
        "send",
        json!({"to": "planner", "text": "echo NAO-DEVE-CHEGAR"}),
    );
    assert_eq!(r.error, Some(ErrorKind::WrongDirection));
    std::thread::sleep(Duration::from_millis(300));
    assert!(!host
        .screen_text("a_planner", 50)
        .unwrap()
        .contains("NAO-DEVE-CHEGAR"));

    host.host.shutdown_all(Duration::from_secs(1));
}
