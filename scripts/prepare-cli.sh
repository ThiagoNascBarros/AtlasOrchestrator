#!/usr/bin/env bash
# Compila a CLI `atlasoq` e a coloca onde o bundler do Tauri espera o externalBin
# (app/src-tauri/binaries/atlasoq-<target-triple>).
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
profile="${1:-release}"
triple="$(rustc -vV | sed -n 's/^host: //p')"
if [ "$profile" = "release" ]; then
  cargo build --manifest-path "$root/Cargo.toml" -p atlasoq-cli --release
else
  cargo build --manifest-path "$root/Cargo.toml" -p atlasoq-cli
fi
mkdir -p "$root/app/src-tauri/binaries"
cp "$root/target/$profile/atlasoq" "$root/app/src-tauri/binaries/atlasoq-$triple"
