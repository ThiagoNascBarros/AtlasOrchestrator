# Quickstart: validar o MVP do AtlasOQ

Roteiro para provar, de ponta a ponta, que o MVP atende à [spec](spec.md). Os detalhes de formato
estão nos [contratos](contracts/); aqui ficam só os passos e os resultados esperados.

## Pré-requisitos

- Ubuntu 24.04/26.04 LTS ou Debian 13, sessão Wayland ou X11.
- Desenvolvimento: Rust estável (via rustup), Node 24 + pnpm, `libwebkit2gtk-4.1-dev`,
  `libgtk-3-dev`, `build-essential`, `pkg-config`.
- Validação com agentes reais: pelo menos duas CLIs de agente instaladas e autenticadas (ex.:
  `claude` e `codex`).

## 1. Build e testes automáticos

```bash
cargo test --workspace          # PTY, broker, persistência, CLI (contratos), tutorial
pnpm -C app/ui test             # store, foco, geometria do canvas
pnpm -C app/ui e2e              # E2E via tauri-driver (Xvfb e Wayland headless)
pnpm -C app tauri build --bundles deb
```

Esperado: todos os testes passam; `.deb` gerado em `target/release/bundle/deb/`.

## 2. Instalação (história 6)

```bash
sudo apt install ./atlasoq_<versão>_amd64.deb
which atlasoq                    # /usr/bin/atlasoq
```

Esperado: "AtlasOQ" aparece no menu com ícone próprio.

## 3. Montar o time (histórias 1 e 3)

1. Abrir o AtlasOQ → "Novo workspace" → escolher uma pasta de projeto de teste.
   Esperado: `<pasta>/.atlasoq/` criado com `workspace.json` e `.gitignore`.
2. Criar o agente `planner` (perfil Claude Code, papel "planeja") e `coder` (perfil Codex, papel
   "implementa").
   Esperado: em até 30 s cada nó mostra `pronto` no aprendizado, sem nenhuma ação.
3. No `coder`, pedir em linguagem natural: "quem é você no AtlasOQ e com quem você fala?".
   Esperado: ele se identifica como `coder` e diz que ainda não tem conexões.
4. Dar zoom de 25% a 200% e pan.
   Esperado: movimento fluido; texto nítido ao parar; abaixo de 35% os nós viram cartões.

## 4. Conversa entre agentes (história 2)

1. Ligar `planner → coder`.
2. No `planner`: "pergunte ao coder quantos arquivos .rs existem no projeto".
   Esperado: `planner` usa `atlasoq ask coder ...`; `coder` recebe `[atlasoq ask de:planner ...]`,
   responde com `atlasoq reply "..."`; `planner` recebe a resposta; a linha anima.
3. No terminal do `coder`, rodar `atlasoq send planner "oi"` manualmente.
   Esperado: código 3, mensagem dizendo que a conexão só vai de `planner` para `coder`.
4. Pausar a conexão e repetir o passo 2.
   Esperado: recusa imediata com o motivo.
5. Clicar na linha.
   Esperado: histórico com as mensagens e recusas.
6. Ligar os dois em ↔ e pedir que conversem "até concordarem sobre um nome de função".
   Esperado: no máximo 20 trocas e a conexão fica `pausada automaticamente` com aviso.

## 5. Notas e memória (histórias 4 e 7)

1. Criar a nota `plano`, ligar `coder → plano`. No `coder`: "anote no plano que os testes passaram".
   Esperado: linha `- ... [coder] ...` no fim de `.atlasoq/notes/plano.md` e na nota do canvas.
2. Pedir ao `coder` uma tarefa curta e, ao fim, conferir `.atlasoq/memory/coder.md`.
   Esperado: anotações `feito:`/`decisão:`/`pendente:` com data e hora.
3. Fechar o app e reabrir.
   Esperado: agentes iniciam sozinhos; o `coder` responde "o que você fez por último?" com base na
   memória, sem perguntar se deve continuar.
4. `git status` na raiz: `memory/` não aparece. Clicar em "Versionar memória": passa a aparecer.

## 6. Robustez (história 5)

```bash
pkill -9 -f atlasoq-desktop      # durante uma movimentação de nós
pgrep -f "claude|codex"          # nenhum processo de agente restante
```

Reabrir: workspace íntegro, perda máxima de 1 s de layout.

## 7. Workspace de terceiros

Clonar um repositório com `.atlasoq/` criado em outra máquina e abri-lo.
Esperado: diálogo de aprovação listando comandos, perfis, hooks e `learn.md`; agentes só iniciam
após aprovar.

## 8. Segurança da entrega

```bash
cargo test -p atlasoq-link sanitize   # conjunto de payloads com ESC, CSI, OSC, Ctrl-C
```

Esperado: 100% neutralizados (SC-006).
