# Spike R3: zoom de terminais no WebKitGTK (portão da Fase 3)

Tarefa: T014. Decide se seguimos com Tauri ou acionamos o plano B (Electron), conforme
`docs/adr/001-stack.md`.

## Como rodar

```bash
source "$HOME/.cargo/env"
pnpm -C app tauri dev
```

Crie um workspace numa pasta de teste e 12 agentes com o perfil `shell` (tecla `N` no canvas).
Em 8 deles, rode um gerador de saída contínua: `yes "$(date)" | pv -qL 200k` ou
`while :; do ls -la /usr/bin; done`.

## Critérios (marcar em cada ambiente)

| Critério | GNOME/Wayland | KDE/Wayland | X11 |
|---|---|---|---|
| Texto nítido depois que o zoom para (25%, 50%, 100%, 200%) | | | |
| Seleção com o mouse acerta os caracteres em 25%, 50%, 100% e 200% | | | |
| Pan fluido (≥ 50 fps) com 12 terminais | | | |
| Cliques e teclas respondem (< 100 ms) com 8 terminais em saída contínua | | | |
| Sem tela branca nem janela invisível | | | |
| Abaixo de 35% os agentes viram cartões e voltam ao terminal ao aproximar | | | |

Resultado: ☐ aprovado (seguir com Tauri) ☐ reprovado (plano B)

Observações:
