use std::fs;

use atlasoq_store::{StoreError, WorkspaceDir};

#[test]
fn cria_estrutura_e_gitignore() {
    let root = tempfile::tempdir().unwrap();
    let (dir, ws) = WorkspaceDir::create(root.path(), "projeto").unwrap();
    for sub in ["notes", "profiles", "memory", "agents", "logs", "backups"] {
        assert!(dir.dir().join(sub).is_dir(), "{sub}");
    }
    let gi = fs::read_to_string(dir.gitignore()).unwrap();
    for line in [
        "agents/",
        "logs/",
        "backups/",
        ".lock",
        "memory/   # atlasoq:memory",
    ] {
        assert!(gi.lines().any(|l| l == line), "{line}");
    }
    assert!(ws.id.starts_with("ws_"));
    drop(dir);
    let (_, reopened) = WorkspaceDir::open(root.path()).unwrap();
    assert_eq!(reopened, ws);
}

#[test]
fn nao_abre_duas_vezes() {
    let root = tempfile::tempdir().unwrap();
    let (_dir, _) = WorkspaceDir::create(root.path(), "p").unwrap();
    assert!(matches!(
        WorkspaceDir::open(root.path()),
        Err(StoreError::AlreadyOpen)
    ));
}

#[test]
fn pasta_sem_workspace() {
    let root = tempfile::tempdir().unwrap();
    assert!(matches!(
        WorkspaceDir::open(root.path()),
        Err(StoreError::NotAWorkspace(_))
    ));
}

#[test]
fn nao_recria_existente() {
    let root = tempfile::tempdir().unwrap();
    let (dir, _) = WorkspaceDir::create(root.path(), "p").unwrap();
    drop(dir);
    assert!(matches!(
        WorkspaceDir::create(root.path(), "p"),
        Err(StoreError::AlreadyExists(_))
    ));
}

#[test]
fn raiz_sem_permissao_de_escrita() {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir().unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o555)).unwrap();
    let r = WorkspaceDir::create(root.path(), "p");
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o755)).unwrap();
    assert!(matches!(r, Err(StoreError::NotWritable(_))), "{r:?}");
}

#[test]
fn salva_e_recarrega_alteracoes() {
    let root = tempfile::tempdir().unwrap();
    let (dir, mut ws) = WorkspaceDir::create(root.path(), "p").unwrap();
    ws.viewport.zoom = 0.85;
    ws.name = "renomeado".into();
    dir.save(&ws).unwrap();
    assert_eq!(dir.load().unwrap(), ws);
}
