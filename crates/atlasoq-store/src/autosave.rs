//! Salvamento automático com debounce (padrão: 500 ms após a última alteração).

use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::model::Workspace;

type SaveFn = dyn Fn(&Workspace) + Send + Sync;

struct State {
    pending: Option<Workspace>,
    last_change: Instant,
    stop: bool,
}

/// Agenda gravações e grava a versão mais recente depois de `delay` sem novas alterações.
pub struct Autosaver {
    inner: Arc<(Mutex<State>, Condvar)>,
    save: Arc<SaveFn>,
    worker: Option<thread::JoinHandle<()>>,
}

impl Autosaver {
    pub fn new(delay: Duration, save: impl Fn(&Workspace) + Send + Sync + 'static) -> Self {
        let inner = Arc::new((
            Mutex::new(State {
                pending: None,
                last_change: Instant::now(),
                stop: false,
            }),
            Condvar::new(),
        ));
        let save: Arc<SaveFn> = Arc::new(save);
        let worker = {
            let inner = inner.clone();
            let save = save.clone();
            thread::Builder::new()
                .name("atlasoq-autosave".into())
                .spawn(move || run(inner, save, delay))
                .expect("não foi possível criar a thread de salvamento")
        };
        Self {
            inner,
            save,
            worker: Some(worker),
        }
    }

    pub fn schedule(&self, ws: Workspace) {
        let (lock, cv) = &*self.inner;
        let mut st = lock.lock().unwrap();
        st.pending = Some(ws);
        st.last_change = Instant::now();
        cv.notify_all();
    }

    /// Grava agora o que estiver pendente.
    pub fn flush(&self) {
        let pending = self.inner.0.lock().unwrap().pending.take();
        if let Some(ws) = pending {
            (self.save)(&ws);
        }
    }
}

fn run(inner: Arc<(Mutex<State>, Condvar)>, save: Arc<SaveFn>, delay: Duration) {
    let (lock, cv) = &*inner;
    let mut st = lock.lock().unwrap();
    loop {
        if st.stop {
            return;
        }
        if st.pending.is_none() {
            st = cv.wait(st).unwrap();
            continue;
        }
        let elapsed = st.last_change.elapsed();
        if elapsed < delay {
            st = cv.wait_timeout(st, delay - elapsed).unwrap().0;
            continue;
        }
        let ws = st.pending.take();
        drop(st);
        if let Some(ws) = ws {
            save(&ws);
        }
        st = lock.lock().unwrap();
    }
}

impl Drop for Autosaver {
    fn drop(&mut self) {
        self.flush();
        {
            let (lock, cv) = &*self.inner;
            lock.lock().unwrap().stop = true;
            cv.notify_all();
        }
        if let Some(w) = self.worker.take() {
            let _ = w.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn agrupa_alteracoes_e_grava_a_ultima() {
        let saves = Arc::new(Mutex::new(Vec::new()));
        let count = Arc::new(AtomicUsize::new(0));
        let a = {
            let saves = saves.clone();
            let count = count.clone();
            Autosaver::new(Duration::from_millis(100), move |ws| {
                count.fetch_add(1, Ordering::SeqCst);
                saves.lock().unwrap().push(ws.name.clone());
            })
        };
        for i in 0..5 {
            a.schedule(Workspace::new("ws_1".into(), format!("v{i}")));
            thread::sleep(Duration::from_millis(10));
        }
        thread::sleep(Duration::from_millis(400));
        assert_eq!(count.load(Ordering::SeqCst), 1);
        assert_eq!(saves.lock().unwrap().as_slice(), ["v4"]);
    }

    #[test]
    fn drop_grava_pendente() {
        let saves = Arc::new(Mutex::new(Vec::new()));
        {
            let saves = saves.clone();
            let a = Autosaver::new(Duration::from_secs(60), move |ws| {
                saves.lock().unwrap().push(ws.name.clone())
            });
            a.schedule(Workspace::new("ws_1".into(), "final".into()));
        }
        assert_eq!(saves.lock().unwrap().as_slice(), ["final"]);
    }
}
