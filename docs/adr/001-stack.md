# ADR-001: Stack da aplicação

- **Status:** aceito (2026-10-11), sujeito ao spike R3 (docs/spikes/r3-zoom.md)
- **Contexto:** o AtlasOQ precisa de terminais reais num canvas com zoom, notas em Markdown,
  controle fino de processos e empacotamento `.deb` para Linux (Wayland e X11).
- **Decisão:** Tauri v2 com UI em TypeScript + React + Zustand (Vite) sobre WebKitGTK 4.1;
  terminais com xterm.js; núcleo em Rust com `portable-pty` e parser `vt100`, dividido em crates
  sem dependência do Tauri (`atlasoq-core`, `atlasoq-pty`, `atlasoq-store`, `atlasoq-link`,
  `atlasoq-cli`).
- **Consequências:**
  - Consumo de memória e tamanho do pacote menores que com Electron.
  - Risco no WebKitGTK (WebGL, DMA-BUF, Wayland) e no zoom do xterm.js; atacado no spike R3.
  - **Plano B:** se o spike reprovar, a UI migra para Electron mantendo os crates Rust como
    processo separado.
- **Alternativas:** Electron + node-pty (mais memória, pacote grande); GTK4 + VTE (zoom de
  terminais e Markdown difíceis).
