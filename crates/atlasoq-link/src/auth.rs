//! Autenticação: mesmo usuário (`SO_PEERCRED`) e token de 256 bits por sessão de agente.

use std::collections::HashMap;
use std::io::Read;
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;
use std::sync::Mutex;

/// `uid` do processo do outro lado do socket.
pub fn peer_uid(stream: &UnixStream) -> Option<u32> {
    let mut cred = libc::ucred {
        pid: 0,
        uid: 0,
        gid: 0,
    };
    let mut len = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    // SAFETY: `cred` e `len` são válidos e o tamanho informado é o da estrutura.
    let rc = unsafe {
        libc::getsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            (&mut cred as *mut libc::ucred).cast(),
            &mut len,
        )
    };
    (rc == 0).then_some(cred.uid)
}

pub fn own_uid() -> u32 {
    // SAFETY: `geteuid` não tem pré-condições.
    unsafe { libc::geteuid() }
}

/// O par só é aceito se o `uid` for conhecido e igual ao do app.
pub fn peer_allowed(peer: Option<u32>, own: u32) -> bool {
    peer == Some(own)
}

/// Comparação em tempo constante (não vaza o prefixo comum).
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Gera 32 bytes aleatórios em base64url (sem preenchimento).
pub fn new_token() -> String {
    let mut bytes = [0u8; 32];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut bytes))
        .expect("sem fonte de aleatoriedade em /dev/urandom");
    base64url(&bytes)
}

fn base64url(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |n, (i, b)| n | (*b as u32) << (16 - 8 * i));
        let chars = chunk.len() + 1;
        for i in 0..chars {
            out.push(ALPHABET[((n >> (18 - 6 * i)) & 63) as usize] as char);
        }
    }
    out
}

/// Tokens vigentes: id do agente → token da sessão atual.
#[derive(Default)]
pub struct TokenStore {
    tokens: Mutex<HashMap<String, String>>,
}

impl TokenStore {
    /// Emite um novo token para o agente (invalida o anterior).
    pub fn issue(&self, node_id: &str) -> String {
        let token = new_token();
        self.tokens
            .lock()
            .unwrap()
            .insert(node_id.to_owned(), token.clone());
        token
    }

    pub fn revoke(&self, node_id: &str) {
        self.tokens.lock().unwrap().remove(node_id);
    }

    pub fn verify(&self, node_id: &str, token: &str) -> bool {
        self.tokens
            .lock()
            .unwrap()
            .get(node_id)
            .is_some_and(|t| constant_time_eq(t.as_bytes(), token.as_bytes()))
    }
}
