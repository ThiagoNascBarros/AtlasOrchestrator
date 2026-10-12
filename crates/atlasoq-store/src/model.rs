//! Tipos do `workspace.json` (schema 1) e dos perfis (data-model.md).

use std::collections::BTreeMap;

use atlasoq_core::NodeName;
use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: u32 = 1;
pub const ZOOM_MIN: f64 = 0.1;
pub const ZOOM_MAX: f64 = 4.0;
pub const ROLE_MAX_CHARS: usize = 200;
pub const NODE_MIN_W: f64 = 240.0;
pub const NODE_MIN_H: f64 = 160.0;
pub const COLS_RANGE: (u16, u16) = (20, 500);
pub const ROWS_RANGE: (u16, u16) = (5, 200);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Workspace {
    pub schema: u32,
    pub id: String,
    pub name: String,
    pub viewport: Viewport,
    #[serde(default)]
    pub settings: Settings,
    #[serde(default)]
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub connections: Vec<Connection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Viewport {
    pub x: f64,
    pub y: f64,
    pub zoom: f64,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            zoom: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Settings {
    #[serde(default)]
    pub memory_versioned: bool,
    #[serde(default)]
    pub limits_default: ConnectionLimits,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Node {
    Agent(Agent),
    Note(Note),
}

impl Node {
    pub fn id(&self) -> &str {
        match self {
            Node::Agent(a) => &a.id,
            Node::Note(n) => &n.id,
        }
    }

    pub fn name(&self) -> &NodeName {
        match self {
            Node::Agent(a) => &a.name,
            Node::Note(n) => &n.name,
        }
    }

    pub fn rect(&self) -> &Rect {
        match self {
            Node::Agent(a) => &a.rect,
            Node::Note(n) => &n.rect,
        }
    }

    pub fn is_note(&self) -> bool {
        matches!(self, Node::Note(_))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: NodeName,
    #[serde(default)]
    pub role: String,
    pub profile: String,
    pub rect: Rect,
    pub terminal: Terminal,
    pub learn: LearnSettings,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Terminal {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    /// Relativo à raiz do workspace.
    pub cwd: String,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub env_secret_keys: Vec<String>,
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LearnMode {
    Hook,
    Prompt,
    Type,
    Off,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearnSettings {
    pub mode: LearnMode,
    #[serde(default = "default_ready_timeout")]
    pub ready_timeout_s: u64,
    /// Modo `prompt`: flag que antecede o texto inicial (ex.: `--prompt`). Sem flag, o texto vai
    /// como argumento posicional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_flag: Option<String>,
}

fn default_ready_timeout() -> u64 {
    60
}

impl LearnSettings {
    pub fn new(mode: LearnMode) -> Self {
        Self {
            mode,
            ready_timeout_s: default_ready_timeout(),
            prompt_flag: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub name: NodeName,
    pub rect: Rect,
    /// Relativo a `.atlasoq/` (ex.: `notes/plano.md`).
    pub file: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    /// `from → to`
    Forward,
    /// `to → from`
    Backward,
    /// ↔
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnState {
    Active,
    Paused,
    AutoPaused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PauseReason {
    MaxExchanges,
    RateLimit,
    PingPong,
}

impl PauseReason {
    pub fn as_str(self) -> &'static str {
        match self {
            PauseReason::MaxExchanges => "max_exchanges",
            PauseReason::RateLimit => "rate_limit",
            PauseReason::PingPong => "ping_pong",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionLimits {
    pub max_exchanges: u32,
    pub rate_per_min: u32,
    pub burst: u32,
    pub ping_pong_repeats: u32,
}

impl Default for ConnectionLimits {
    fn default() -> Self {
        Self {
            max_exchanges: 20,
            rate_per_min: 6,
            burst: 3,
            ping_pong_repeats: 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Connection {
    pub id: String,
    pub from: String,
    pub to: String,
    pub direction: Direction,
    pub state: ConnState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pause_reason: Option<PauseReason>,
    #[serde(default)]
    pub limits: ConnectionLimits,
}

impl Workspace {
    pub fn new(id: String, name: String) -> Self {
        Self {
            schema: SCHEMA_VERSION,
            id,
            name,
            viewport: Viewport::default(),
            settings: Settings::default(),
            nodes: Vec::new(),
            connections: Vec::new(),
        }
    }

    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id() == id)
    }

    pub fn node_by_name(&self, name: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.name().as_str() == name)
    }

    pub fn agents(&self) -> impl Iterator<Item = &Agent> {
        self.nodes.iter().filter_map(|n| match n {
            Node::Agent(a) => Some(a),
            Node::Note(_) => None,
        })
    }
}
