//! Escrita atômica: temporário no mesmo diretório → fsync → rename → fsync do diretório
//! (constituição, princípio V).

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::{Result, StoreError};

static COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    write_atomic_with(path, bytes, || Ok(()))
}

/// Variante com um gancho chamado depois de gravar o temporário e antes do `rename`.
/// Usada nos testes para simular uma interrupção nesse ponto.
#[doc(hidden)]
pub fn write_atomic_with(
    path: &Path,
    bytes: &[u8],
    before_rename: impl FnOnce() -> std::io::Result<()>,
) -> Result<()> {
    let dir = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let tmp = dir.join(format!(
        ".{name}.tmp-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));

    let result = (|| {
        let mut f = File::create(&tmp).map_err(|e| StoreError::io(&tmp, e))?;
        f.write_all(bytes).map_err(|e| StoreError::io(&tmp, e))?;
        f.sync_all().map_err(|e| StoreError::io(&tmp, e))?;
        drop(f);
        before_rename().map_err(|e| StoreError::io(&tmp, e))?;
        fs::rename(&tmp, path).map_err(|e| StoreError::io(path, e))?;
        File::open(dir)
            .and_then(|d| d.sync_all())
            .map_err(|e| StoreError::io(dir, e))
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}
