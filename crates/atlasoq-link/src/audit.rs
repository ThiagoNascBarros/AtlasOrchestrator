//! Log de auditoria JSONL diário (`.atlasoq/logs/links-AAAA-MM-DD.jsonl`).

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Default, Serialize)]
pub struct AuditEvent {
    pub ts: String,
    pub event: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conn_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchanges: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
}

impl AuditEvent {
    pub fn new(event: &'static str) -> Self {
        Self {
            ts: now_rfc3339(),
            event,
            ..Self::default()
        }
    }
}

/// Anexa o evento ao arquivo do dia. Falhas de escrita não derrubam a operação.
pub fn append(logs_dir: &Path, ev: &AuditEvent) {
    let _ = fs::create_dir_all(logs_dir);
    let file = logs_dir.join(format!("links-{}.jsonl", &ev.ts[..10]));
    let Ok(line) = serde_json::to_string(ev) else {
        return;
    };
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(file) {
        let _ = writeln!(f, "{line}");
    }
}

/// Eventos mais recentes da conexão (mais novo primeiro), lendo os arquivos do mais novo ao mais
/// antigo.
pub fn history(logs_dir: &Path, conn_id: &str, limit: usize) -> Vec<Value> {
    let Ok(rd) = fs::read_dir(logs_dir) else {
        return Vec::new();
    };
    let mut files: Vec<_> = rd
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("links-") && n.ends_with(".jsonl"))
        })
        .collect();
    files.sort();
    let mut out = Vec::new();
    for path in files.into_iter().rev() {
        let Ok(text) = fs::read_to_string(path) else {
            continue;
        };
        for line in text.lines().rev() {
            if let Ok(v) = serde_json::from_str::<Value>(line) {
                if v.get("conn_id").and_then(Value::as_str) == Some(conn_id) {
                    out.push(v);
                    if out.len() >= limit {
                        return out;
                    }
                }
            }
        }
    }
    out
}

/// Data e hora UTC em RFC 3339 (`2026-10-12T14:03:09Z`).
pub fn now_rfc3339() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0) as i64;
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    // Dias desde 1970-01-01 → data civil (algoritmo de Howard Hinnant).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}
