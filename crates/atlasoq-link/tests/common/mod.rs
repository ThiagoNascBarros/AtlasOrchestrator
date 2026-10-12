//! Host de mentira para testar o broker sem PTYs.
#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use atlasoq_core::{Request, Response};
use atlasoq_link::{Broker, Host, LinkEvent, NodeInfo, SessionInfo};
use atlasoq_store::{ConnState, Connection, ConnectionLimits, Direction, PauseReason};
use serde_json::{json, Value};

pub struct FakeHost {
    pub nodes: Mutex<Vec<NodeInfo>>,
    pub conns: Mutex<Vec<Connection>>,
    pub exited: Mutex<Vec<String>>,
    pub written: Mutex<Vec<(String, Vec<u8>)>>,
    pub bracketed: bool,
    pub events: Mutex<Vec<LinkEvent>>,
    pub logs: Option<PathBuf>,
}

pub fn agent(id: &str, name: &str) -> NodeInfo {
    NodeInfo {
        id: id.into(),
        name: name.into(),
        role: format!("papel de {name}"),
        is_note: false,
    }
}

pub fn conn(id: &str, from: &str, to: &str, direction: Direction) -> Connection {
    Connection {
        id: id.into(),
        from: from.into(),
        to: to.into(),
        direction,
        state: ConnState::Active,
        pause_reason: None,
        limits: ConnectionLimits {
            max_exchanges: 20,
            rate_per_min: 6000,
            burst: 1000,
            ping_pong_repeats: 4,
        },
    }
}

impl FakeHost {
    pub fn new(conns: Vec<Connection>) -> Arc<Self> {
        Arc::new(Self {
            nodes: Mutex::new(vec![agent("a_p", "planner"), agent("a_c", "coder"), agent("a_r", "revisor")]),
            conns: Mutex::new(conns),
            exited: Mutex::new(vec![]),
            written: Mutex::new(vec![]),
            bracketed: false,
            events: Mutex::new(vec![]),
            logs: None,
        })
    }

    pub fn text_for(&self, node: &str) -> String {
        self.written
            .lock()
            .unwrap()
            .iter()
            .filter(|(n, _)| n == node)
            .map(|(_, d)| String::from_utf8_lossy(d).into_owned())
            .collect()
    }
}

impl Host for FakeHost {
    fn nodes(&self) -> Vec<NodeInfo> {
        self.nodes.lock().unwrap().clone()
    }
    fn connections(&self) -> Vec<Connection> {
        self.conns.lock().unwrap().clone()
    }
    fn session(&self, node_id: &str) -> Option<SessionInfo> {
        let exited = self.exited.lock().unwrap().iter().any(|n| n == node_id);
        Some(SessionInfo {
            status: if exited { "exited" } else { "idle" },
            exited,
            bracketed_paste: self.bracketed,
            last_user_input: None,
        })
    }
    fn write(&self, node_id: &str, data: &[u8]) -> Result<(), String> {
        self.written.lock().unwrap().push((node_id.into(), data.to_vec()));
        Ok(())
    }
    fn screen_text(&self, node_id: &str, _lines: usize) -> Result<String, String> {
        Ok(format!("tela de {node_id}"))
    }
    fn set_connection_state(&self, conn_id: &str, state: ConnState, reason: Option<PauseReason>) {
        for c in self.conns.lock().unwrap().iter_mut().filter(|c| c.id == conn_id) {
            c.state = state;
            c.pause_reason = reason;
        }
    }
    fn logs_dir(&self) -> Option<PathBuf> {
        self.logs.clone()
    }
    fn emit(&self, event: LinkEvent) {
        self.events.lock().unwrap().push(event);
    }
}

pub fn broker(host: &Arc<FakeHost>) -> Arc<Broker> {
    Arc::new(Broker::with_timing(
        host.clone(),
        Duration::from_millis(1),
        Duration::from_millis(20),
    ))
}

pub fn request(node: &str, token: &str, op: &str, args: Value) -> String {
    json!({"v": 1, "op": op, "node": node, "token": token, "args": args}).to_string()
}

pub fn call(b: &Broker, node: &str, token: &str, op: &str, args: Value) -> Response {
    let line = request(node, token, op, args);
    // Garante que o formato da requisição é o do contrato.
    let _: Request = serde_json::from_str(&line).expect("requisição válida");
    b.handle(&line)
}

impl FakeHost {
    pub fn set_connection_state_for_test(&self, conn_id: &str, state: ConnState) {
        self.set_connection_state(conn_id, state, None);
    }
}
