# Ambiente de desenvolvimento

O AtlasOQ é desenvolvido e testado em Linux (Ubuntu 24.04/26.04 LTS ou Debian 13).

## Dependências do sistema

```bash
sudo apt install build-essential pkg-config curl \
  libwebkit2gtk-4.1-dev libgtk-3-dev libssl-dev librsvg2-dev libayatana-appindicator3-dev
```

## Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
rustup component add rustfmt clippy
cargo install --locked cargo-deny
```

## Node e pnpm

Node 24 (por exemplo via nvm) e pnpm via corepack:

```bash
corepack enable pnpm
pnpm -C app install
```

## Comandos

| Ação | Comando |
|---|---|
| Testes Rust | `cargo test --workspace` |
| Lint Rust | `cargo lint` (alias de clippy com `-D warnings`) |
| Licenças e avisos | `cargo deny check` |
| Testes da UI | `pnpm -C app/ui test` |
| Lint da UI | `pnpm -C app/ui lint` |
| App em modo dev | `pnpm -C app tauri dev` |
| Pacote `.deb` | `pnpm -C app tauri build --bundles deb` |

Commits começam com o identificador da branch, por exemplo `ATQ-001 ...`.
