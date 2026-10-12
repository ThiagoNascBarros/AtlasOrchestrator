use std::fs;

use atlasoq_store::atomic::{write_atomic, write_atomic_with};

#[test]
fn grava_e_nao_deixa_temporarios() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("workspace.json");
    write_atomic(&path, b"v1").unwrap();
    write_atomic(&path, b"v2").unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"v2");
    let others: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(others.len(), 1, "não deve sobrar arquivo temporário");
}

#[test]
fn interrupcao_antes_do_rename_mantem_versao_antiga() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("workspace.json");
    write_atomic(&path, b"versao antiga").unwrap();
    let r = write_atomic_with(&path, b"versao nova incompleta", || {
        Err(std::io::Error::other("processo interrompido"))
    });
    assert!(r.is_err());
    assert_eq!(fs::read(&path).unwrap(), b"versao antiga");
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn escritas_concorrentes_nunca_deixam_arquivo_parcial() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("w.json");
    let a = vec![b'a'; 256 * 1024];
    let b = vec![b'b'; 256 * 1024];
    write_atomic(&path, &a).unwrap();
    std::thread::scope(|s| {
        for i in 0..8 {
            let (path, a, b) = (&path, &a, &b);
            s.spawn(move || {
                for _ in 0..20 {
                    write_atomic(path, if i % 2 == 0 { a } else { b }).unwrap();
                }
            });
        }
        for _ in 0..200 {
            let content = fs::read(&path).unwrap();
            assert!(
                content == a || content == b,
                "conteúdo parcial: {} bytes",
                content.len()
            );
        }
    });
}
