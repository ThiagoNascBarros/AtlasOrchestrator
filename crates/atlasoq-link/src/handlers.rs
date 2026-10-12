//! Operações do protocolo: `list`, `send`, `read`, `ask`, `reply`.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use atlasoq_core::i18n::{self, Msg};
use atlasoq_core::protocol::{MAX_READ_LINES, PROTOCOL_VERSION};
use atlasoq_core::{ErrorKind, Op, Request, Response};
use atlasoq_store::{ConnState, Connection, PauseReason};
use serde_json::{json, Value};

use crate::acl::{self, Denied};
use crate::ask::Outcome;
use crate::audit::AuditEvent;
use crate::broker::{Broker, LinkEvent, NodeInfo};
use crate::delivery;
use crate::loop_guard::Verdict;
use crate::sanitize::{sanitize, MAX_TEXT_BYTES};

fn fail(kind: ErrorKind, m: Msg) -> Response {
    Response::err(kind, m.message, m.hint)
}

impl Broker {
    /// Trata uma linha do protocolo: versão, autenticação, autorização e operação.
    pub fn handle(&self, raw: &str) -> Response {
        let value: Value = match serde_json::from_str(raw) {
            Ok(v) => v,
            Err(e) => return fail(ErrorKind::BadRequest, i18n::bad_request(&e.to_string())),
        };
        let v = value.get("v").and_then(Value::as_u64).unwrap_or(0) as u32;
        if v != PROTOCOL_VERSION {
            return fail(ErrorKind::UnsupportedVersion, i18n::unsupported_version(v));
        }
        let req: Request = match serde_json::from_value(value) {
            Ok(r) => r,
            Err(e) => return fail(ErrorKind::BadRequest, i18n::bad_request(&e.to_string())),
        };
        let me = self
            .host
            .nodes()
            .into_iter()
            .find(|n| n.id == req.node && !n.is_note);
        let Some(me) = me.filter(|_| self.tokens.verify(&req.node, &req.token)) else {
            return fail(ErrorKind::Unauthorized, i18n::unauthorized());
        };
        if self.closing.load(Ordering::SeqCst) {
            return Response::err(ErrorKind::AppClosing, "o AtlasOQ está fechando.", "");
        }
        match req.op {
            Op::List {} => self.op_list(&me),
            Op::Send { to, text } => self.op_send(&me, &to, &text),
            Op::Read { from, lines } => self.op_read(&me, &from, lines),
            Op::Ask {
                to,
                text,
                timeout_s,
            } => self.op_ask(&me, &to, &text, timeout_s),
            Op::Reply { to, text } => self.op_reply(&me, to, &text),
            Op::Learn { .. } | Op::NoteRead { .. } | Op::NoteAppend { .. } => fail(
                ErrorKind::BadRequest,
                i18n::bad_request("este comando ainda não está disponível nesta versão"),
            ),
        }
    }

    /// Nomes dos agentes sobre os quais `me` pode agir (para as dicas de erro).
    fn reachable(&self, me: &NodeInfo) -> Vec<String> {
        let conns = self.host.connections();
        self.host
            .nodes()
            .into_iter()
            .filter(|n| !n.is_note && n.id != me.id)
            .filter(|n| {
                conns
                    .iter()
                    .any(|c| acl::links(c, &me.id, &n.id) && acl::direction_allows(c, &me.id, &n.id))
            })
            .map(|n| n.name)
            .collect()
    }

    fn resolve(&self, me: &NodeInfo, name: &str) -> Result<NodeInfo, Response> {
        if name == me.name {
            return Err(fail(
                ErrorKind::BadRequest,
                i18n::bad_request("você não pode falar consigo mesmo"),
            ));
        }
        self.host
            .nodes()
            .into_iter()
            .find(|n| !n.is_note && n.name == name)
            .ok_or_else(|| {
                fail(
                    ErrorKind::UnknownTarget,
                    i18n::unknown_target(name, &self.reachable(me)),
                )
            })
    }

    fn authorize(&self, me: &NodeInfo, target: &NodeInfo) -> Result<Connection, Response> {
        let conns = self.host.connections();
        match acl::authorize(&conns, &me.id, &target.id) {
            Ok(c) => Ok(c.clone()),
            Err(Denied::NoConnection) => Err(fail(
                ErrorKind::NoConnection,
                i18n::no_connection(&me.name, &target.name, &self.reachable(me)),
            )),
            Err(Denied::WrongDirection) => Err(fail(
                ErrorKind::WrongDirection,
                i18n::wrong_direction(&me.name, &target.name),
            )),
            Err(Denied::Paused(reason)) => Err(fail(
                ErrorKind::ConnectionPaused,
                i18n::connection_paused(&me.name, &target.name, reason.map(PauseReason::as_str)),
            )),
        }
    }

    fn check_text(&self, text: &str) -> Result<String, Response> {
        let clean = sanitize(text);
        if clean.trim().is_empty() {
            return Err(fail(ErrorKind::TooLarge, i18n::empty_text()));
        }
        if clean.len() > MAX_TEXT_BYTES {
            return Err(fail(ErrorKind::TooLarge, i18n::too_large(MAX_TEXT_BYTES)));
        }
        Ok(clean)
    }

    fn session_ready(&self, target: &NodeInfo) -> Result<bool, Response> {
        match self.host.session(&target.id) {
            Some(s) if !s.exited => Ok(s.bracketed_paste),
            _ => Err(fail(ErrorKind::TargetExited, i18n::target_exited(&target.name))),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn log_message(
        &self,
        kind: &'static str,
        me: &NodeInfo,
        target: &NodeInfo,
        conn_id: Option<&str>,
        msg_id: Option<u64>,
        text: Option<&str>,
        result: &str,
    ) {
        let mut ev = AuditEvent::new("message");
        ev.kind = Some(kind);
        ev.msg_id = msg_id;
        ev.from = Some(me.name.clone());
        ev.to = Some(target.name.clone());
        ev.conn_id = conn_id.map(str::to_owned);
        ev.bytes = text.map(str::len);
        ev.text = text.map(str::to_owned);
        ev.exchanges = conn_id.and_then(|c| {
            self.guards
                .lock()
                .unwrap()
                .get(c)
                .map(crate::loop_guard::LoopGuard::exchanges)
        });
        ev.result = Some(result.to_owned());
        self.audit(&ev);
    }

    /// Aplica a proteção contra loops. Se pausar, a conexão muda de estado e as perguntas
    /// pendentes nela terminam.
    fn admit(
        &self,
        conn: &Connection,
        me: &NodeInfo,
        target: &NodeInfo,
        text: &str,
    ) -> Result<(), Response> {
        let mut h = DefaultHasher::new();
        text.hash(&mut h);
        let user_input = [&me.id, &target.id]
            .iter()
            .filter_map(|id| self.host.session(id).and_then(|s| s.last_user_input))
            .max();
        let verdict = self
            .guards
            .lock()
            .unwrap()
            .entry(conn.id.clone())
            .or_insert_with(|| crate::loop_guard::LoopGuard::new(Instant::now(), &conn.limits))
            .record(Instant::now(), h.finish(), user_input, &conn.limits);
        let Verdict::Pause(reason) = verdict else {
            return Ok(());
        };
        self.host
            .set_connection_state(&conn.id, ConnState::AutoPaused, Some(reason));
        let m = i18n::connection_paused(&me.name, &target.name, Some(reason.as_str()));
        self.connection_stopped(&conn.id, ErrorKind::ConnectionPaused, &m.message);
        let mut ev = AuditEvent::new("connection_paused");
        ev.conn_id = Some(conn.id.clone());
        ev.from = Some(me.name.clone());
        ev.to = Some(target.name.clone());
        ev.result = Some(reason.as_str().to_owned());
        self.audit(&ev);
        Err(fail(ErrorKind::ConnectionPaused, m))
    }

    fn flow(&self, conn_id: &str, from: &NodeInfo, to: &NodeInfo, kind: &'static str) {
        self.host.emit(LinkEvent::Message {
            conn_id: conn_id.to_owned(),
            from_id: from.id.clone(),
            to_id: to.id.clone(),
            kind,
        });
    }

    fn op_send(&self, me: &NodeInfo, to: &str, text: &str) -> Response {
        let run = || -> Result<Response, Response> {
            let target = self.resolve(me, to)?;
            let text = self.check_text(text)?;
            let conn = self.authorize(me, &target)?;
            let bracketed = self.session_ready(&target)?;
            self.admit(&conn, me, &target, &text)?;
            let id = self.next_msg.fetch_add(1, Ordering::SeqCst);
            let payload = delivery::format_message(&me.name, id, &text);
            if let Err(e) = delivery::deliver(
                &*self.host,
                &self.queues,
                &target.id,
                &payload,
                bracketed,
                self.submit_delay,
            ) {
                self.log_message("send", me, &target, Some(&conn.id), Some(id), Some(&text), &format!("failed:{e}"));
                return Err(fail(ErrorKind::TargetExited, i18n::target_exited(&target.name)));
            }
            self.log_message("send", me, &target, Some(&conn.id), Some(id), Some(&text), "delivered");
            self.flow(&conn.id, me, &target, "send");
            Ok(Response::ok(json!({ "msg_id": id, "to": target.name })))
        };
        run().unwrap_or_else(|r| {
            self.log_refusal("send", me, to, &r);
            r
        })
    }

    fn log_refusal(&self, kind: &'static str, me: &NodeInfo, to: &str, r: &Response) {
        let target = NodeInfo {
            id: String::new(),
            name: to.to_owned(),
            role: String::new(),
            is_note: false,
        };
        let why = r
            .error
            .and_then(|e| serde_json::to_value(e).ok())
            .and_then(|v| v.as_str().map(str::to_owned))
            .unwrap_or_default();
        self.log_message(kind, me, &target, None, None, None, &format!("refused:{why}"));
    }

    fn op_read(&self, me: &NodeInfo, from: &str, lines: u32) -> Response {
        let run = || -> Result<Response, Response> {
            let target = self.resolve(me, from)?;
            let conn = self.authorize(me, &target)?;
            if self.host.session(&target.id).is_none() {
                return Err(fail(ErrorKind::TargetExited, i18n::target_exited(&target.name)));
            }
            let lines = lines.clamp(1, MAX_READ_LINES) as usize;
            let text = self
                .host
                .screen_text(&target.id, lines)
                .map_err(|_| fail(ErrorKind::TargetExited, i18n::target_exited(&target.name)))?;
            self.log_message("read", me, &target, Some(&conn.id), None, None, "ok");
            Ok(Response::ok(json!({ "text": text })))
        };
        run().unwrap_or_else(|r| {
            self.log_refusal("read", me, from, &r);
            r
        })
    }

    fn op_ask(&self, me: &NodeInfo, to: &str, text: &str, timeout_s: u64) -> Response {
        let prepared = || -> Result<(NodeInfo, Connection, String, bool), Response> {
            let target = self.resolve(me, to)?;
            let text = self.check_text(text)?;
            let conn = self.authorize(me, &target)?;
            let bracketed = self.session_ready(&target)?;
            self.admit(&conn, me, &target, &text)?;
            Ok((target, conn, text, bracketed))
        };
        let (target, conn, text, bracketed) = match prepared() {
            Ok(p) => p,
            Err(r) => {
                self.log_refusal("ask", me, to, &r);
                return r;
            }
        };
        let id = self.asks.open(&me.id, &me.name, &target.id, &conn.id);
        let payload = delivery::format_ask(&me.name, id, &text);
        if let Err(e) = delivery::deliver(
            &*self.host,
            &self.queues,
            &target.id,
            &payload,
            bracketed,
            self.submit_delay,
        ) {
            self.asks.remove(id);
            self.log_message("ask", me, &target, Some(&conn.id), Some(id), Some(&text), &format!("failed:{e}"));
            return fail(ErrorKind::TargetExited, i18n::target_exited(&target.name));
        }
        self.log_message("ask", me, &target, Some(&conn.id), Some(id), Some(&text), "delivered");
        self.flow(&conn.id, me, &target, "ask");

        let timeout = timeout_s.clamp(1, 3600);
        let outcome = self.asks.wait(
            id,
            Instant::now() + Duration::from_secs(timeout),
            self.tick,
            || self.ask_cancelled(&conn.id, &target),
        );
        match outcome {
            Outcome::Answered(answer) => Response::ok(json!({ "text": answer, "from": target.name })),
            Outcome::TimedOut => {
                self.log_message("ask", me, &target, Some(&conn.id), Some(id), None, "timeout");
                let m = i18n::ask_timeout(&target.name, timeout);
                let screen = self.host.screen_text(&target.id, 60).unwrap_or_default();
                Response::err_with_data(ErrorKind::Timeout, m.message, m.hint, json!({ "screen": screen }))
            }
            Outcome::Cancelled(kind, message) => Response::err(kind, message, ""),
        }
    }

    /// Condição que interrompe um `ask` em espera: conexão pausada/apagada ou destino encerrado.
    fn ask_cancelled(&self, conn_id: &str, target: &NodeInfo) -> Option<(ErrorKind, String)> {
        if self.closing.load(Ordering::SeqCst) {
            return Some((ErrorKind::AppClosing, "o AtlasOQ está fechando.".into()));
        }
        let conns = self.host.connections();
        match conns.iter().find(|c| c.id == conn_id) {
            None => {
                return Some((
                    ErrorKind::NoConnection,
                    format!("a conexão com \"{}\" foi apagada.", target.name),
                ))
            }
            Some(c) if c.state != ConnState::Active => {
                return Some((
                    ErrorKind::ConnectionPaused,
                    format!("a conexão com \"{}\" foi pausada.", target.name),
                ))
            }
            Some(_) => {}
        }
        match self.host.session(&target.id) {
            Some(s) if !s.exited => None,
            _ => Some((ErrorKind::TargetExited, i18n::target_exited(&target.name).message)),
        }
    }

    fn op_reply(&self, me: &NodeInfo, to: Option<String>, text: &str) -> Response {
        let pending = self.asks.pending_for(&me.id);
        // `to` só vale como destinatário se for o nome de um perguntador pendente; senão faz
        // parte do texto.
        let (wanted, text) = match to {
            Some(t) if pending.iter().any(|p| p.from_name == t) => (Some(t), text.to_owned()),
            Some(t) => (None, format!("{t} {text}").trim().to_owned()),
            None => (None, text.to_owned()),
        };
        let chosen = match (&wanted, pending.len()) {
            (_, 0) => {
                return match self.asks.last_expired_for(&me.id) {
                    Some(e) => fail(ErrorKind::AskExpired, i18n::ask_expired(&e.from_name)),
                    None => fail(ErrorKind::NoPendingAsk, i18n::no_pending_ask()),
                }
            }
            (Some(w), _) => pending.iter().find(|p| &p.from_name == w),
            (None, 1) => pending.first(),
            (None, _) => {
                let names: Vec<String> = pending.iter().map(|p| p.from_name.clone()).collect();
                return fail(ErrorKind::AmbiguousReply, i18n::ambiguous_reply(&names));
            }
        };
        let Some(chosen) = chosen else {
            return fail(ErrorKind::NoPendingAsk, i18n::no_pending_ask());
        };
        let text = match self.check_text(&text) {
            Ok(t) => t,
            Err(r) => return r,
        };
        let Some(conn_id) = self.asks.answer(chosen.id, &text) else {
            return fail(ErrorKind::AskExpired, i18n::ask_expired(&chosen.from_name));
        };
        let asker = NodeInfo {
            id: chosen.from_id.clone(),
            name: chosen.from_name.clone(),
            role: String::new(),
            is_note: false,
        };
        let user_input = self.host.session(&me.id).and_then(|s| s.last_user_input);
        if let Some(g) = self.guards.lock().unwrap().get_mut(&conn_id) {
            g.note_exchange(Instant::now(), user_input);
        }
        self.log_message("reply", me, &asker, Some(&conn_id), Some(chosen.id), Some(&text), "delivered");
        self.flow(&conn_id, me, &asker, "reply");
        Response::ok(json!({ "to": asker.name }))
    }

    fn op_list(&self, me: &NodeInfo) -> Response {
        let conns = self.host.connections();
        let nodes = self.host.nodes();
        let status_of = |id: &str| {
            self.host
                .session(id)
                .map_or("encerrado", |s| status_label(s.status))
        };
        let mut agents = Vec::new();
        let mut notes = Vec::new();
        for n in nodes.iter().filter(|n| n.id != me.id) {
            let linked: Vec<&Connection> =
                conns.iter().filter(|c| acl::links(c, &me.id, &n.id)).collect();
            let Some(conn) = linked.first() else { continue };
            let i_can = acl::direction_allows(conn, &me.id, &n.id);
            let they_can = acl::direction_allows(conn, &n.id, &me.id);
            let relation = match (i_can, they_can) {
                (true, true) => "both",
                (true, false) => "outbound",
                _ => "inbound",
            };
            let state = match conn.state {
                ConnState::Active => "active",
                ConnState::Paused => "paused",
                ConnState::AutoPaused => "auto_paused",
            };
            let item = json!({ "name": n.name, "role": n.role, "relation": relation, "state": state, "status": status_of(&n.id) });
            if n.is_note {
                notes.push(item);
            } else {
                agents.push(item);
            }
        }
        let pending: Vec<Value> = self
            .asks
            .pending_for(&me.id)
            .into_iter()
            .map(|p| json!({ "from": p.from_name, "id": p.id }))
            .collect();
        Response::ok(json!({
            "you": { "name": me.name, "role": me.role, "status": status_of(&me.id) },
            "agents": agents,
            "notes": notes,
            "pending": pending,
        }))
    }
}

fn status_label(status: &str) -> &'static str {
    match status {
        "working" => "trabalhando",
        "idle" => "ocioso",
        "starting" => "iniciando",
        _ => "encerrado",
    }
}
