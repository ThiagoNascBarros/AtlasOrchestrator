//! Leitura do PTY: alimenta o parser headless e publica a saída agrupada por quadro.

use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::activity::{Activity, ActivityConfig, ActivityTracker};
use crate::host::SessionEvent;

/// Intervalo de agrupamento da saída enviada à UI.
pub(crate) const FRAME: Duration = Duration::from_millis(16);

/// Depois que o processo sai, quanto esperar pelo fim da leitura antes de encerrar a sessão.
const DRAIN_AFTER_EXIT: Duration = Duration::from_millis(300);

/// Estado compartilhado entre a sessão, a thread de leitura e a de publicação.
pub(crate) struct Shared {
    pub parser: Mutex<vt100::Parser>,
    pending: Mutex<Vec<u8>>,
    subscribers: Mutex<Vec<Sender<SessionEvent>>>,
    tracker: Mutex<ActivityTracker>,
    pub last_user: Mutex<Option<Instant>>,
    reader_done: AtomicBool,
    exit: Mutex<Option<(Option<i32>, Instant)>>,
    finished: AtomicBool,
}

impl Shared {
    pub fn new(rows: u16, cols: u16, scrollback: usize, config: ActivityConfig) -> Arc<Self> {
        Arc::new(Self {
            parser: Mutex::new(vt100::Parser::new(rows, cols, scrollback)),
            pending: Mutex::new(Vec::new()),
            subscribers: Mutex::new(Vec::new()),
            tracker: Mutex::new(ActivityTracker::new(config)),
            last_user: Mutex::new(None),
            reader_done: AtomicBool::new(false),
            exit: Mutex::new(None),
            finished: AtomicBool::new(false),
        })
    }

    pub fn subscribe(&self, tx: Sender<SessionEvent>) {
        if self.finished.load(Ordering::SeqCst) {
            let code = self.exit.lock().unwrap().map(|(c, _)| c).flatten();
            let _ = tx.send(SessionEvent::Exited(code));
            return;
        }
        self.subscribers.lock().unwrap().push(tx);
    }

    pub fn activity(&self) -> Activity {
        self.tracker.lock().unwrap().current(Instant::now())
    }

    pub fn exit_code(&self) -> Option<Option<i32>> {
        self.exit.lock().unwrap().map(|(c, _)| c)
    }

    pub fn on_exit(&self, code: Option<i32>) {
        *self.exit.lock().unwrap() = Some((code, Instant::now()));
    }

    fn broadcast(&self, event: SessionEvent) {
        self.subscribers
            .lock()
            .unwrap()
            .retain(|tx| tx.send(event.clone()).is_ok());
    }
}

/// Thread que lê o PTY até EOF.
pub(crate) fn spawn_reader(shared: Arc<Shared>, mut reader: Box<dyn Read + Send>) {
    thread::Builder::new()
        .name("atlasoq-pty-read".into())
        .spawn(move || {
            let mut buf = [0u8; 16 * 1024];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        let chunk = &buf[..n];
                        shared.parser.lock().unwrap().process(chunk);
                        shared.pending.lock().unwrap().extend_from_slice(chunk);
                        shared.tracker.lock().unwrap().on_output(Instant::now());
                    }
                }
            }
            shared.reader_done.store(true, Ordering::SeqCst);
        })
        .expect("não foi possível criar a thread de leitura do PTY");
}

/// Thread que publica a saída a cada quadro, as mudanças de status e o fim da sessão.
pub(crate) fn spawn_publisher(shared: Arc<Shared>) {
    thread::Builder::new()
        .name("atlasoq-pty-pub".into())
        .spawn(move || {
            let mut last_status = Activity::Starting;
            loop {
                thread::sleep(FRAME);
                let chunk = std::mem::take(&mut *shared.pending.lock().unwrap());
                if !chunk.is_empty() {
                    shared.broadcast(SessionEvent::Output(chunk));
                }

                let exit = *shared.exit.lock().unwrap();
                if let Some((code, at)) = exit {
                    let drained = shared.reader_done.load(Ordering::SeqCst)
                        || at.elapsed() >= DRAIN_AFTER_EXIT;
                    if drained {
                        let rest = std::mem::take(&mut *shared.pending.lock().unwrap());
                        if !rest.is_empty() {
                            shared.broadcast(SessionEvent::Output(rest));
                        }
                        shared.tracker.lock().unwrap().on_exit(code);
                        shared.finished.store(true, Ordering::SeqCst);
                        shared.broadcast(SessionEvent::Exited(code));
                        shared.subscribers.lock().unwrap().clear();
                        return;
                    }
                }

                let status = shared.activity();
                if status != last_status {
                    last_status = status;
                    shared.broadcast(SessionEvent::Status(status));
                }
            }
        })
        .expect("não foi possível criar a thread de publicação do PTY");
}
