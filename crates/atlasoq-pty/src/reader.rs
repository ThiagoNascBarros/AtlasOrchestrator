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

/// Saída ainda não publicada e a posição dela no fluxo total de bytes.
#[derive(Default)]
struct Pending {
    start: u64,
    bytes: Vec<u8>,
}

/// Estado compartilhado entre a sessão, a thread de leitura e a de publicação.
///
/// Ordem de locks: `parser` → `processed` → `pending` → `subscribers`.
pub(crate) struct Shared {
    pub parser: Mutex<vt100::Parser>,
    /// Bytes já entregues ao parser (posição atual do fluxo).
    processed: Mutex<u64>,
    pending: Mutex<Pending>,
    /// Assinantes e a posição do fluxo a partir da qual cada um quer receber saída.
    subscribers: Mutex<Vec<(Sender<SessionEvent>, u64)>>,
    tracker: Mutex<ActivityTracker>,
    pub last_user: Mutex<Option<Instant>>,
    reader_done: AtomicBool,
    exit: Mutex<Option<(Option<i32>, Instant)>>,
    finished: AtomicBool,
}

/// Tela no instante da assinatura: (bytes formatados, colunas, linhas).
pub(crate) type ScreenState = (Vec<u8>, u16, u16);

impl Shared {
    pub fn new(rows: u16, cols: u16, scrollback: usize, config: ActivityConfig) -> Arc<Self> {
        Arc::new(Self {
            parser: Mutex::new(vt100::Parser::new(rows, cols, scrollback)),
            processed: Mutex::new(0),
            pending: Mutex::new(Pending::default()),
            subscribers: Mutex::new(Vec::new()),
            tracker: Mutex::new(ActivityTracker::new(config)),
            last_user: Mutex::new(None),
            reader_done: AtomicBool::new(false),
            exit: Mutex::new(None),
            finished: AtomicBool::new(false),
        })
    }

    /// Registra um assinante a partir da posição atual do fluxo. Se `with_snapshot`, devolve
    /// também a tela nesse mesmo instante: o assinante recebe só a saída posterior a ela.
    pub fn subscribe(&self, tx: Sender<SessionEvent>, with_snapshot: bool) -> Option<ScreenState> {
        let parser = self.parser.lock().unwrap();
        let snapshot = with_snapshot.then(|| {
            let (rows, cols) = parser.screen().size();
            (parser.screen().contents_formatted(), cols, rows)
        });
        let from = *self.processed.lock().unwrap();
        if self.finished.load(Ordering::SeqCst) {
            let code = self.exit.lock().unwrap().and_then(|(c, _)| c);
            let _ = tx.send(SessionEvent::Exited(code));
        } else {
            self.subscribers.lock().unwrap().push((tx, from));
        }
        snapshot
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
            .retain(|(tx, _)| tx.send(event.clone()).is_ok());
    }

    /// Publica a saída pendente respeitando a posição inicial de cada assinante.
    fn flush_output(&self) {
        let (start, chunk) = {
            let mut p = self.pending.lock().unwrap();
            let start = p.start;
            p.start += p.bytes.len() as u64;
            (start, std::mem::take(&mut p.bytes))
        };
        if chunk.is_empty() {
            return;
        }
        self.subscribers.lock().unwrap().retain(|(tx, from)| {
            let skip = from.saturating_sub(start) as usize;
            if skip >= chunk.len() {
                return true;
            }
            tx.send(SessionEvent::Output(chunk[skip..].to_vec()))
                .is_ok()
        });
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
                        let mut parser = shared.parser.lock().unwrap();
                        parser.process(chunk);
                        *shared.processed.lock().unwrap() += n as u64;
                        shared
                            .pending
                            .lock()
                            .unwrap()
                            .bytes
                            .extend_from_slice(chunk);
                        drop(parser);
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
                shared.flush_output();

                let exit = *shared.exit.lock().unwrap();
                if let Some((code, at)) = exit {
                    let drained = shared.reader_done.load(Ordering::SeqCst)
                        || at.elapsed() >= DRAIN_AFTER_EXIT;
                    if drained {
                        shared.flush_output();
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
