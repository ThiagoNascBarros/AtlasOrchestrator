//! Implementação local de [`SessionHost`] sobre `portable-pty`.

use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize, SlavePty};

use crate::activity::{Activity, ActivityConfig};
use crate::host::{PtyError, SessionEvent, SessionHost, SessionId, SessionSpec, Snapshot};
use crate::reader::{spawn_publisher, spawn_reader, Shared};
use crate::screen::last_lines;
use crate::shutdown::terminate_group;

type SpawnResult = Result<Box<dyn Child + Send + Sync>, String>;

struct SpawnJob {
    slave: Box<dyn SlavePty + Send>,
    cmd: CommandBuilder,
    reply: Sender<SpawnResult>,
}

struct Session {
    master: Mutex<Box<dyn MasterPty + Send>>,
    writer: Mutex<Box<dyn Write + Send>>,
    shared: Arc<Shared>,
    pid: Option<u32>,
}

/// Host de sessões no próprio processo do app.
///
/// Os processos são criados por uma thread dedicada e de vida longa: `PR_SET_PDEATHSIG` dispara
/// na morte da thread que criou o filho, então ela precisa viver tanto quanto o app.
pub struct LocalSessionHost {
    sessions: Mutex<HashMap<SessionId, Arc<Session>>>,
    next_id: AtomicU64,
    spawner: Mutex<Sender<SpawnJob>>,
    activity: ActivityConfig,
    pdeathsig_wrapper: Option<&'static str>,
}

impl Default for LocalSessionHost {
    fn default() -> Self {
        Self::new(ActivityConfig::default())
    }
}

impl LocalSessionHost {
    pub fn new(activity: ActivityConfig) -> Self {
        let (tx, rx) = mpsc::channel::<SpawnJob>();
        thread::Builder::new()
            .name("atlasoq-pty-spawner".into())
            .spawn(move || {
                for job in rx {
                    let result = job.slave.spawn_command(job.cmd).map_err(|e| e.to_string());
                    drop(job.slave);
                    let _ = job.reply.send(result);
                }
            })
            .expect("não foi possível criar a thread de spawn");
        let pdeathsig_wrapper = ["/usr/bin/setpriv", "/bin/setpriv"]
            .into_iter()
            .find(|p| Path::new(p).exists());
        Self {
            sessions: Mutex::new(HashMap::new()),
            next_id: AtomicU64::new(1),
            spawner: Mutex::new(tx),
            activity,
            pdeathsig_wrapper,
        }
    }

    fn get(&self, id: SessionId) -> Result<Arc<Session>, PtyError> {
        self.sessions
            .lock()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or(PtyError::UnknownSession(id))
    }

    fn build_command(&self, spec: &SessionSpec) -> CommandBuilder {
        let mut cmd = match self.pdeathsig_wrapper {
            // O filho recebe SIGHUP se o app morrer, mesmo com kill -9 (FR-043).
            Some(setpriv) => {
                let mut c = CommandBuilder::new(setpriv);
                c.args(["--pdeathsig", "HUP", "--"]);
                c.arg(&spec.command);
                c
            }
            None => CommandBuilder::new(&spec.command),
        };
        cmd.args(&spec.args);
        cmd.cwd(&spec.cwd);
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");
        for (k, v) in &spec.env {
            cmd.env(k, v);
        }
        cmd
    }

    /// Número de sessões registradas (vivas ou recém-encerradas).
    pub fn len(&self) -> usize {
        self.sessions.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

fn exit_code(status: &portable_pty::ExitStatus) -> Option<i32> {
    if status.signal().is_some() {
        None
    } else {
        Some(status.exit_code() as i32)
    }
}

impl SessionHost for LocalSessionHost {
    fn spawn(&self, spec: SessionSpec) -> Result<SessionId, PtyError> {
        let size = PtySize {
            rows: spec.rows,
            cols: spec.cols,
            pixel_width: 0,
            pixel_height: 0,
        };
        let pair = native_pty_system()
            .openpty(size)
            .map_err(|e| PtyError::Spawn(e.to_string()))?;
        let cmd = self.build_command(&spec);

        let (reply_tx, reply_rx) = mpsc::channel();
        self.spawner
            .lock()
            .unwrap()
            .send(SpawnJob {
                slave: pair.slave,
                cmd,
                reply: reply_tx,
            })
            .map_err(|_| PtyError::Spawn("thread de spawn encerrada".into()))?;
        let mut child = reply_rx
            .recv()
            .map_err(|_| PtyError::Spawn("thread de spawn encerrada".into()))?
            .map_err(PtyError::Spawn)?;

        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| PtyError::Io(e.to_string()))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|e| PtyError::Io(e.to_string()))?;
        let shared = Shared::new(spec.rows, spec.cols, spec.scrollback, self.activity);
        let pid = child.process_id();

        spawn_reader(shared.clone(), reader);
        spawn_publisher(shared.clone());
        {
            let shared = shared.clone();
            thread::Builder::new()
                .name("atlasoq-pty-wait".into())
                .spawn(move || {
                    let code = child.wait().ok().and_then(|s| exit_code(&s));
                    shared.on_exit(code);
                })
                .map_err(|e| PtyError::Spawn(e.to_string()))?;
        }

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let session = Session {
            master: Mutex::new(pair.master),
            writer: Mutex::new(writer),
            shared,
            pid,
        };
        self.sessions.lock().unwrap().insert(id, Arc::new(session));
        Ok(id)
    }

    fn write(&self, id: SessionId, data: &[u8]) -> Result<(), PtyError> {
        let s = self.get(id)?;
        let mut w = s.writer.lock().unwrap();
        w.write_all(data)
            .and_then(|_| w.flush())
            .map_err(|e| PtyError::Io(e.to_string()))
    }

    fn write_user(&self, id: SessionId, data: &[u8]) -> Result<(), PtyError> {
        let s = self.get(id)?;
        *s.shared.last_user.lock().unwrap() = Some(Instant::now());
        self.write(id, data)
    }

    fn resize(&self, id: SessionId, cols: u16, rows: u16) -> Result<(), PtyError> {
        let s = self.get(id)?;
        s.master
            .lock()
            .unwrap()
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| PtyError::Io(e.to_string()))?;
        s.shared
            .parser
            .lock()
            .unwrap()
            .screen_mut()
            .set_size(rows, cols);
        Ok(())
    }

    fn kill(&self, id: SessionId, grace: Duration) -> Result<(), PtyError> {
        let s = self.get(id)?;
        if let Some(pid) = s.pid {
            let shared = s.shared.clone();
            terminate_group(pid, grace, move || shared.exit_code().is_some());
        }
        Ok(())
    }

    fn subscribe(&self, id: SessionId) -> Result<Receiver<SessionEvent>, PtyError> {
        let s = self.get(id)?;
        let (tx, rx) = mpsc::channel();
        s.shared.subscribe(tx);
        Ok(rx)
    }

    fn snapshot(&self, id: SessionId) -> Result<Snapshot, PtyError> {
        let s = self.get(id)?;
        let parser = s.shared.parser.lock().unwrap();
        let (rows, cols) = parser.screen().size();
        Ok(Snapshot {
            cols,
            rows,
            formatted: parser.screen().contents_formatted(),
        })
    }

    fn screen_text(&self, id: SessionId, lines: usize) -> Result<String, PtyError> {
        let s = self.get(id)?;
        let mut parser = s.shared.parser.lock().unwrap();
        Ok(last_lines(&mut parser, lines))
    }

    fn bracketed_paste(&self, id: SessionId) -> Result<bool, PtyError> {
        let s = self.get(id)?;
        let parser = s.shared.parser.lock().unwrap();
        Ok(parser.screen().bracketed_paste())
    }

    fn activity(&self, id: SessionId) -> Result<Activity, PtyError> {
        Ok(self.get(id)?.shared.activity())
    }

    fn last_user_input(&self, id: SessionId) -> Result<Option<Instant>, PtyError> {
        Ok(*self.get(id)?.shared.last_user.lock().unwrap())
    }

    fn pid(&self, id: SessionId) -> Result<Option<u32>, PtyError> {
        Ok(self.get(id)?.pid)
    }

    fn shutdown_all(&self, grace: Duration) {
        let sessions: Vec<Arc<Session>> = self.sessions.lock().unwrap().values().cloned().collect();
        let handles: Vec<_> = sessions
            .into_iter()
            .filter_map(|s| {
                let pid = s.pid?;
                Some(thread::spawn(move || {
                    let shared = s.shared.clone();
                    terminate_group(pid, grace, move || shared.exit_code().is_some());
                }))
            })
            .collect();
        for h in handles {
            let _ = h.join();
        }
    }
}
