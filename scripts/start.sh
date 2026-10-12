#!/usr/bin/env bash
# Inicia o AtlasOQ em modo dev com um único comando (ou um clique no atalho do menu).
# Carrega Rust e Node mesmo fora de um terminal interativo e instala as dependências da UI
# na primeira execução.
set -euo pipefail
root="$(cd "$(dirname "$(readlink -f "$0")")/.." && pwd)"

[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"
if [ -s "${NVM_DIR:-$HOME/.nvm}/nvm.sh" ]; then
  . "${NVM_DIR:-$HOME/.nvm}/nvm.sh"
fi

for cmd in cargo node pnpm; do
  if ! command -v "$cmd" >/dev/null; then
    echo "atlasoq: \"$cmd\" não encontrado; veja docs/desenvolvimento.md" >&2
    read -rp "Enter para fechar" _ || true
    exit 1
  fi
done

cd "$root"
if [ ! -d app/node_modules ] || [ ! -d app/ui/node_modules ]; then
  echo "atlasoq: instalando dependências da UI..."
  pnpm -C app install
fi

echo "atlasoq: iniciando (a primeira compilação demora alguns minutos)..."
# O app põe o atlasoq no PATH dos agentes a partir da pasta do próprio executável.
cargo build -p atlasoq-cli
pnpm -C app tauri dev || {
  echo "atlasoq: o app terminou com erro."
  read -rp "Enter para fechar" _ || true
}
