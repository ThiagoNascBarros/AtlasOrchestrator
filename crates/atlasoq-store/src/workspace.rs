//! Criação, abertura e gravação do diretório `<raiz>/.atlasoq/`.

use std::fs;
use std::path::{Path, PathBuf};

use atlasoq_core::{new_id, IdKind};

use crate::atomic::write_atomic;
use crate::error::{Result, StoreError};
use crate::lock::WorkspaceLock;
use crate::model::Workspace;
use crate::validate::validate;

pub const DIR_NAME: &str = ".atlasoq";
pub const WORKSPACE_FILE: &str = "workspace.json";
pub const MEMORY_GITIGNORE_LINE: &str = "memory/   # atlasoq:memory";

/// `.gitignore` gerado (contracts/workspace-format.md).
pub fn default_gitignore() -> String {
    format!(
        "# gerado pelo AtlasOQ — as linhas marcadas são gerenciadas pelo app\n\
         agents/\nlogs/\nbackups/\n.lock\n{MEMORY_GITIGNORE_LINE}\n"
    )
}

/// Um workspace aberto: caminhos e o lock exclusivo.
#[derive(Debug)]
pub struct WorkspaceDir {
    root: PathBuf,
    dir: PathBuf,
    _lock: WorkspaceLock,
}

impl WorkspaceDir {
    /// Cria `<raiz>/.atlasoq/` e um workspace vazio.
    pub fn create(root: &Path, name: &str) -> Result<(Self, Workspace)> {
        let root = root.canonicalize().map_err(|e| StoreError::io(root, e))?;
        let dir = root.join(DIR_NAME);
        if dir.join(WORKSPACE_FILE).exists() {
            return Err(StoreError::AlreadyExists(dir));
        }
        fs::create_dir_all(&dir).map_err(|e| match e.kind() {
            std::io::ErrorKind::PermissionDenied => StoreError::NotWritable(root.clone()),
            _ => StoreError::io(&dir, e),
        })?;
        for sub in ["notes", "profiles", "memory", "agents", "logs", "backups"] {
            let p = dir.join(sub);
            fs::create_dir_all(&p).map_err(|e| StoreError::io(&p, e))?;
        }
        write_atomic(&dir.join(".gitignore"), default_gitignore().as_bytes())?;

        let lock = WorkspaceLock::acquire(&dir.join(".lock"))?;
        let ws = Workspace::new(new_id(IdKind::Workspace), name.to_owned());
        let this = Self {
            root,
            dir,
            _lock: lock,
        };
        this.save(&ws)?;
        Ok((this, ws))
    }

    /// Abre e valida um workspace existente.
    pub fn open(root: &Path) -> Result<(Self, Workspace)> {
        let root = root.canonicalize().map_err(|e| StoreError::io(root, e))?;
        let dir = root.join(DIR_NAME);
        let file = dir.join(WORKSPACE_FILE);
        if !file.exists() {
            return Err(StoreError::NotAWorkspace(root));
        }
        let lock = WorkspaceLock::acquire(&dir.join(".lock"))?;
        let this = Self {
            root,
            dir,
            _lock: lock,
        };
        let ws = this.load()?;
        Ok((this, ws))
    }

    pub fn load(&self) -> Result<Workspace> {
        let file = self.workspace_file();
        let bytes = fs::read(&file).map_err(|e| StoreError::io(&file, e))?;
        let ws: Workspace = serde_json::from_slice(&bytes).map_err(|e| StoreError::Json {
            path: file.clone(),
            source: e,
        })?;
        let problems = validate(&ws);
        if problems.is_empty() {
            Ok(ws)
        } else {
            Err(StoreError::Invalid(problems.join("; ")))
        }
    }

    pub fn save(&self, ws: &Workspace) -> Result<()> {
        let problems = validate(ws);
        if !problems.is_empty() {
            return Err(StoreError::Invalid(problems.join("; ")));
        }
        let mut json = serde_json::to_vec_pretty(ws).map_err(|e| StoreError::Json {
            path: self.workspace_file(),
            source: e,
        })?;
        json.push(b'\n');
        write_atomic(&self.workspace_file(), &json)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn dir(&self) -> &Path {
        &self.dir
    }
    pub fn workspace_file(&self) -> PathBuf {
        self.dir.join(WORKSPACE_FILE)
    }
    pub fn notes_dir(&self) -> PathBuf {
        self.dir.join("notes")
    }
    pub fn memory_dir(&self) -> PathBuf {
        self.dir.join("memory")
    }
    pub fn profiles_dir(&self) -> PathBuf {
        self.dir.join("profiles")
    }
    pub fn agents_dir(&self) -> PathBuf {
        self.dir.join("agents")
    }
    pub fn logs_dir(&self) -> PathBuf {
        self.dir.join("logs")
    }
    pub fn backups_dir(&self) -> PathBuf {
        self.dir.join("backups")
    }
    pub fn learn_md(&self) -> PathBuf {
        self.dir.join("learn.md")
    }
    pub fn gitignore(&self) -> PathBuf {
        self.dir.join(".gitignore")
    }
}
