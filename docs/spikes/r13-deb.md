# Spike R13: pacote `.deb` com a CLI no PATH

Data: 2026-10-11 · Tarefa: T016

- `scripts/prepare-cli.sh` compila `atlasoq` e o copia para
  `app/src-tauri/binaries/atlasoq-<target-triple>`; o Tauri o declara em `bundle.externalBin`.
- `pnpm -C app tauri build --bundles deb` gerou `AtlasOQ_0.1.0_amd64.deb` (2,6 MiB) com
  `/usr/bin/atlasoq-desktop`, `/usr/bin/atlasoq`, ícones em `hicolor` e `AtlasOQ.desktop`.
- `Depends`: `libwebkit2gtk-4.1-0, libgtk-3-0` (o Tauri já os inclui; a lista do
  `tauri.conf.json` duplica as entradas).

Ajustes para o T128:
- Remover `depends` duplicados do `tauri.conf.json`.
- O nome do pacote saiu `atlas-oq` (derivado de `productName`); fixar `atlasoq`.
- Conferir `StartupWMClass` do `.desktop` com o `app_id` em Wayland.
