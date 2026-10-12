//! Servidor de socket Unix: uma requisição JSON por conexão, linhas de até 64 KiB.

use std::fs;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use atlasoq_core::i18n;
use atlasoq_core::protocol::MAX_LINE_BYTES;
use atlasoq_core::{ErrorKind, Response};

use crate::auth;
use crate::broker::Broker;

/// `$XDG_RUNTIME_DIR/atlasoq/<instância>.sock` (ou `/tmp/atlasoq-<uid>/` sem `XDG_RUNTIME_DIR`).
pub fn socket_path(instance: &str) -> PathBuf {
    let dir = std::env::var_os("XDG_RUNTIME_DIR")
        .map(|d| PathBuf::from(d).join("atlasoq"))
        .unwrap_or_else(|| PathBuf::from(format!("/tmp/atlasoq-{}", auth::own_uid())));
    dir.join(format!("{instance}.sock"))
}

pub struct Server {
    path: PathBuf,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Server {
    /// Cria o diretório (`0700`) e o socket (`0600`) e começa a atender.
    pub fn start(broker: Arc<Broker>, path: PathBuf) -> io::Result<Self> {
        if let Some(dir) = path.parent() {
            fs::DirBuilder::new()
                .recursive(true)
                .mode(0o700)
                .create(dir)?;
            fs::set_permissions(dir, fs::Permissions::from_mode(0o700))?;
        }
        let _ = fs::remove_file(&path);
        let listener = UnixListener::bind(&path)?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
        listener.set_nonblocking(true)?;
        let stop = Arc::new(AtomicBool::new(false));
        let thread = {
            let stop = stop.clone();
            std::thread::Builder::new()
                .name("atlasoq-broker".into())
                .spawn(move || accept_loop(&listener, &broker, &stop))?
        };
        Ok(Self {
            path,
            stop,
            thread: Some(thread),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
        let _ = fs::remove_file(&self.path);
    }
}

fn accept_loop(listener: &UnixListener, broker: &Arc<Broker>, stop: &AtomicBool) {
    while !stop.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((stream, _)) => {
                let broker = broker.clone();
                let _ = std::thread::Builder::new()
                    .name("atlasoq-request".into())
                    .spawn(move || serve(&broker, stream));
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(_) => std::thread::sleep(Duration::from_millis(200)),
        }
    }
}

fn serve(broker: &Broker, stream: UnixStream) {
    // Outro usuário: fecha sem responder.
    if !auth::peer_allowed(auth::peer_uid(&stream), auth::own_uid()) {
        return;
    }
    let _ = stream.set_nonblocking(false);
    let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
    let mut line = Vec::new();
    let mut reader = BufReader::new(&stream).take(MAX_LINE_BYTES as u64 + 1);
    let read = reader.read_until(b'\n', &mut line);
    let response = match read {
        Ok(0) | Err(_) => return,
        Ok(_) if line.len() > MAX_LINE_BYTES => Response::err(
            ErrorKind::TooLarge,
            "a requisição passa do limite de 64 KiB.",
            "Envie um texto menor.",
        ),
        Ok(_) => match std::str::from_utf8(&line) {
            Ok(text) => broker.handle(text.trim_end()),
            Err(_) => Response::err(
                ErrorKind::BadRequest,
                i18n::bad_request("a requisição não é UTF-8").message,
                "",
            ),
        },
    };
    if let Ok(mut out) = serde_json::to_vec(&response) {
        out.push(b'\n');
        let _ = (&stream).write_all(&out);
    }
}
