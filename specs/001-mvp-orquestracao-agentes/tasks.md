---

description: "Lista de tarefas do MVP do AtlasOQ (spec 001)"
---

# Tasks: MVP do AtlasOQ — canvas para orquestrar agentes de IA

**Input**: Design documents from `/specs/001-mvp-orquestracao-agentes/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md),
[data-model.md](data-model.md), [contracts/](contracts/), [quickstart.md](quickstart.md)

**Tests**: incluídos onde a constituição ou a spec exigem: camada PTY e persistência (Fluxo de
Qualidade), integração entre dois PTYs conectados, contratos da CLI, teste dos exemplos do tutorial
e teste por modo de hook (FR-030), sanitização (SC-006), encerramento forçado (SC-009) e órfãos
(SC-010). Os testes de cada história são escritos antes da implementação e devem falhar primeiro.

**Organization**: tarefas agrupadas por história de usuário, em ordem de prioridade
(US1, US2, US3 = P1; US4, US5, US7 = P2; US6 = P3).

**Commits**: toda mensagem começa com `ATQ-001` (constituição, Fluxo de Desenvolvimento).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: pode rodar em paralelo (arquivos diferentes, sem dependência de tarefa incompleta)
- **[Story]**: história de usuário da spec (US1…US7)
- Caminhos seguem a estrutura do [plan.md](plan.md): `crates/`, `app/src-tauri/`, `app/ui/`,
  `docs/`, `packaging/`, `tests/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: esqueleto do repositório, ferramentas, CI e spikes do M0 que podem mudar decisões.

- [X] T001 Instalar e documentar pré-requisitos de desenvolvimento (rustup estável, Node 24, pnpm, `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `build-essential`, `pkg-config`) em docs/desenvolvimento.md
- [X] T002 Criar o workspace Cargo com os membros `crates/atlasoq-core`, `crates/atlasoq-pty`, `crates/atlasoq-store`, `crates/atlasoq-link`, `crates/atlasoq-cli`, `app/src-tauri` em Cargo.toml (edição 2021, `rust-version` fixado, `license = "Apache-2.0"`)
- [X] T003 [P] Criar os cinco crates vazios com `lib.rs`/`main.rs` e `Cargo.toml` em crates/atlasoq-core/, crates/atlasoq-pty/, crates/atlasoq-store/, crates/atlasoq-link/, crates/atlasoq-cli/ (binário `atlasoq`)
- [X] T004 Inicializar o app Tauri v2 com binário `atlasoq-desktop`, identificador e nome "AtlasOQ" em app/src-tauri/Cargo.toml e app/src-tauri/tauri.conf.json
- [X] T005 [P] Inicializar a UI com Vite + React 18 + TypeScript strict + Zustand + Vitest via pnpm em app/ui/package.json, app/ui/tsconfig.json e app/ui/vite.config.ts
- [X] T006 [P] Configurar rustfmt, clippy (`-D warnings`) e EditorConfig em rustfmt.toml, .cargo/config.toml e .editorconfig
- [X] T007 [P] Configurar ESLint e Prettier para a UI em app/ui/eslint.config.js e app/ui/.prettierrc
- [X] T008 [P] Configurar `cargo-deny` (licenças compatíveis com Apache-2.0, advisories) em deny.toml e verificação de licenças npm em app/ui/package.json (script `licenses`)
- [X] T009 Criar o pipeline de CI (fmt, clippy, `cargo test --workspace`, `cargo deny check`, `pnpm -C app/ui lint test`, build do app) em .github/workflows/ci.yml
- [X] T010 [P] Atualizar .gitignore da raiz para `target/`, `app/ui/node_modules/`, `app/ui/dist/` e manter `.specify/feature.json` ignorado em .gitignore
- [X] T011 [P] Escrever ADR-001 (stack Tauri v2 + xterm.js + portable-pty, plano B Electron) em docs/adr/001-stack.md
- [X] T012 [P] Escrever ADR-002 (conexões com direção, broker, sanitização, ask/reply, limites) em docs/adr/002-conexoes.md
- [X] T013 [P] Escrever ADR-003 (diretório `.atlasoq/`, hook de aprendizado em 4 modos, memória, confiança) em docs/adr/003-atlasoq-dir-e-hooks.md
- [ ] T014 Spike R3: 12 xterm.js ligados a PTYs reais num mundo com pan/zoom, estratégia de fonte por zoom e LOD; medir nitidez, seleção em 25/50/100/200%, fps e tela branca em GNOME/Wayland, KDE/Wayland e X11; registrar resultado e decisão (aprovar Tauri ou acionar plano B) em docs/spikes/r3-zoom.md
- [ ] T015 [P] Spike R4/R5: confirmar que `claude --settings <arquivo>` carrega hook `SessionStart` cuja saída entra no contexto, a sintaxe de `permissions.allow` para `Bash(atlasoq:*)`, e as flags de prompt inicial de `codex`, `gemini --prompt-interactive` e `opencode --prompt`; registrar a tabela final de modos por CLI em docs/spikes/r4-hooks-clis.md
- [X] T016 [P] Spike R13: gerar `.deb` mínimo com `atlasoq-desktop` e o binário `atlasoq` via `externalBin`; confirmar instalação em `/usr/bin/atlasoq` e dependências `libwebkit2gtk-4.1-0`, `libgtk-3-0`; registrar em docs/spikes/r13-deb.md

**Checkpoint**: repositório compila, CI verde, spikes registrados. Se o T014 reprovar o WebKitGTK, parar e revisar SPEC.md antes de seguir.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: tipos compartilhados, camada PTY e núcleo de armazenamento usados por todas as histórias.

**⚠️ CRITICAL**: nenhuma história começa antes desta fase.

### Tipos compartilhados (atlasoq-core)

- [ ] T017 [P] Implementar `NodeName` com validação `^[a-z][a-z0-9-]{0,31}$`, nome reservado `memory` recusado para nós, e ids `a_<ulid>`, `n_<ulid>`, `c_<ulid>`, `ws_<ulid>` em crates/atlasoq-core/src/ids.rs
- [ ] T018 [P] Definir os códigos de saída estáveis (0 sucesso, 2 uso incorreto, 3 sem conexão/permissão, 4 destino indisponível, 5 tempo esgotado, 6 fora do AtlasOQ, 7 conteúdo recusado) e os identificadores de erro do protocolo (`bad_request`, `unknown_target`, `no_connection`, `wrong_direction`, `connection_paused`, `target_exited`, `too_large`, `secret_detected`, `missing_prefix`, `no_pending_ask`, `ambiguous_reply`, `ask_expired`, `timeout`, `unauthorized`, `app_closing`, `unsupported_version`) em crates/atlasoq-core/src/errors.rs
- [ ] T019 [P] Definir requisição e resposta do protocolo `v: 1` (ops `learn`, `list`, `send`, `read`, `ask`, `reply`, `note_read`, `note_append`) conforme contracts/socket-protocol.md em crates/atlasoq-core/src/protocol.rs
- [ ] T020 [P] Criar o catálogo de textos pt-BR (mensagens de erro com "próximo passo", rótulos de status) em crates/atlasoq-core/src/i18n.rs

### Camada PTY (atlasoq-pty) — testes primeiro

- [ ] T021 [P] Teste de integração: spawn de `sh -c 'printf ok'` captura a saída e o código de saída 0 em crates/atlasoq-pty/tests/spawn.rs
- [ ] T022 [P] Teste de integração: `resize(cols, rows)` é refletido em `stty size` do processo em crates/atlasoq-pty/tests/resize.rs
- [ ] T023 [P] Teste de integração: encerrar a sessão (SIGHUP → 3 s → SIGKILL) não deixa processos do grupo vivos em crates/atlasoq-pty/tests/lifecycle.rs
- [ ] T024 [P] Teste de integração: processo que gera 50 MB de saída não bloqueia e o snapshot do parser fica consistente em crates/atlasoq-pty/tests/throughput.rs
- [ ] T025 Definir o trait `SessionHost` (`spawn`, `write`, `resize`, `kill`, `subscribe`, `snapshot`, `screen_text(lines)`) e `SessionSpec` (comando, args, cwd, env, cols, rows) em crates/atlasoq-pty/src/host.rs
- [ ] T026 Implementar `LocalSessionHost` sobre `portable-pty`, com spawn numa thread dedicada de vida longa, `setsid` e `prctl(PR_SET_PDEATHSIG, SIGHUP)` no filho em crates/atlasoq-pty/src/local.rs
- [ ] T027 Implementar a thread de leitura por sessão alimentando o parser `vt100` (scrollback padrão de 10.000 linhas) e um buffer de envio agrupado a cada ~16 ms em crates/atlasoq-pty/src/reader.rs
- [ ] T028 Implementar `screen_text(lines)` sem ANSI e a leitura do modo bracketed paste (2004) a partir do parser em crates/atlasoq-pty/src/screen.rs
- [ ] T029 Implementar o detector de atividade: `working` com saída nos últimos 2 s, `idle` após 3 s sem saída (configurável), `exited(code)`, e marcação de "última tecla do usuário" em crates/atlasoq-pty/src/activity.rs
- [ ] T030 Implementar encerramento de todas as sessões (SIGHUP ao grupo, espera de 3 s, SIGKILL) usado no fechamento do app em crates/atlasoq-pty/src/shutdown.rs

### Núcleo de armazenamento (atlasoq-store) — testes primeiro

- [ ] T031 [P] Teste: escrita atômica sobrevive a interrupção simulada (arquivo final é a versão antiga ou a nova, nunca parcial) em crates/atlasoq-store/tests/atomic.rs
- [ ] T032 [P] Teste: validação de `workspace.json` recusa nomes duplicados, nomes inválidos, ids inexistentes, `from == to`, nota↔nota e `cwd` absoluto em crates/atlasoq-store/tests/validate.rs
- [ ] T033 Implementar `write_atomic` (temporário no mesmo diretório → `fsync` → `rename` → `fsync` do diretório) em crates/atlasoq-store/src/atomic.rs
- [ ] T034 Implementar os tipos `Workspace`, `Agent`, `Note`, `Connection`, `ConnectionLimits` (padrões `max_exchanges` 20, `rate_per_min` 6, `burst` 3, `ping_pong_repeats` 4) com serde, `schema: 1`, `zoom` entre 0.1 e 4.0, `role` até 200 caracteres, `w ≥ 240`, `h ≥ 160`, `cols` 20–500, `rows` 5–200 em crates/atlasoq-store/src/model.rs
- [ ] T035 Implementar a validação descrita em contracts/workspace-format.md em crates/atlasoq-store/src/validate.rs
- [ ] T036 Implementar criar/abrir workspace: cria `<raiz>/.atlasoq/` com `workspace.json`, `notes/`, `profiles/`, `memory/`, `agents/`, `logs/`, `backups/` e o `.gitignore` gerado (linhas `agents/`, `logs/`, `backups/`, `.lock`, `memory/   # atlasoq:memory`), recusando raiz sem permissão de escrita em crates/atlasoq-store/src/workspace.rs
- [ ] T037 Implementar lock exclusivo (`flock` em `.atlasoq/.lock`) impedindo abrir o mesmo workspace duas vezes em crates/atlasoq-store/src/lock.rs
- [ ] T038 Implementar salvamento com debounce de 500 ms disparado por mutações em crates/atlasoq-store/src/autosave.rs
- [ ] T039 Implementar perfis em dois níveis (globais em `~/.config/atlasoq/profiles/`, do workspace em `.atlasoq/profiles/` com precedência) e os perfis prontos `claude-code` (`hook`), `codex` (`prompt`), `gemini` (`prompt`), `opencode` (`prompt`), `aider` (`type`), `shell` (`off`) conforme docs/spikes/r4-hooks-clis.md em crates/atlasoq-store/src/profiles.rs

### Integração Tauri base

- [ ] T040 Implementar o estado do app (workspace aberto, `LocalSessionHost`, autosave) e os comandos Tauri `workspace_create`, `workspace_open`, `workspace_recent` em app/src-tauri/src/state.rs e app/src-tauri/src/commands/workspace.rs
- [ ] T041 Implementar streaming de saída por sessão via `tauri::ipc::Channel` binário e os comandos `session_write`, `session_resize`, `session_snapshot` em app/src-tauri/src/commands/session.rs
- [ ] T042 Encerrar todas as sessões no fechamento da janela e no sinal de término do app em app/src-tauri/src/lifecycle.rs
- [ ] T043 [P] Criar o store Zustand com padrão de comandos (aplicar/desfazer) e cliente tipado dos comandos Tauri em app/ui/src/store/index.ts e app/ui/src/store/tauri.ts
- [ ] T044 [P] Carregar o catálogo de textos pt-BR da UI em app/ui/src/i18n/pt-BR.ts

**Checkpoint**: terminais reais funcionam pela camada PTY com testes verdes; workspace é criado e salvo em `.atlasoq/`.

---

## Phase 3: User Story 1 - Montar o time de agentes no canvas (Priority: P1) 🎯 MVP

**Goal**: criar agentes a partir de perfis, com nome, papel e status, num canvas com pan e zoom.

**Independent Test**: abrir o app, criar três agentes de perfis diferentes, interagir com cada um, mover, redimensionar e navegar com pan e zoom (spec, US1).

### Tests for User Story 1

- [ ] T045 [P] [US1] Testes Vitest da geometria do viewport (zoom 10%–400% centrado no cursor, conversão tela↔mundo) em app/ui/tests/canvas/viewport.test.ts
- [ ] T046 [P] [US1] Testes Vitest da máquina de foco (`Canvas` → `NóSelecionado` → `TerminalComFoco`/`NotaEditando`; `Esc Esc` e clique no fundo voltam ao canvas) em app/ui/tests/focus/focus.test.ts
- [ ] T047 [P] [US1] Teste E2E: criar agente "Shell" chamado `coder`, digitar `echo oi`, ver a saída, recusar segundo agente `coder` em app/ui/tests/e2e/us1-agents.e2e.ts

### Implementation for User Story 1

- [ ] T048 [P] [US1] Implementar o viewport com pan (botão do meio, espaço + arrastar, touchpad) e zoom (Ctrl + roda, pinça, `Ctrl+=`, `Ctrl+-`, `Ctrl+0`) entre 10% e 400% em app/ui/src/canvas/Viewport.tsx
- [ ] T049 [P] [US1] Implementar grade opcional, encaixe na grade, "enquadrar tudo" e "enquadrar seleção" em app/ui/src/canvas/framing.ts
- [ ] T050 [P] [US1] Implementar o minimapa em app/ui/src/canvas/Minimap.tsx
- [ ] T051 [US1] Implementar seleção por clique e retângulo, mover, redimensionar, apagar e desfazer/refazer como comandos do store em app/ui/src/canvas/selection.ts
- [ ] T052 [US1] Implementar a máquina de foco com indicação visual e barra de status em app/ui/src/focus/focusMachine.ts
- [ ] T053 [US1] Implementar `AgentNode` com xterm.js (addons webgl com fallback DOM, fit, search, web-links, unicode11), barra de título com nome, papel, status, cwd e menu (reiniciar, duplicar, encerrar) em app/ui/src/nodes/AgentNode.tsx
- [ ] T054 [US1] Implementar a estratégia de zoom de terminais validada no T014 (escala CSS durante o gesto; `fontSize = base × zoom` ao parar, mantendo colunas e linhas) em app/ui/src/nodes/terminalZoom.ts
- [ ] T055 [US1] Implementar o LOD: abaixo de 35% de zoom ou fora da tela, desmontar o xterm.js e mostrar cartão (nome, papel, status, últimas linhas); remontar a partir de `session_snapshot` em app/ui/src/nodes/AgentCard.tsx
- [ ] T056 [US1] Implementar o diálogo "novo agente" (perfil, nome único, papel opcional, diretório, variáveis com marcação de secretas) com validação de nome e sugestão de nome alternativo em app/ui/src/nodes/NewAgentDialog.tsx
- [ ] T057 [US1] Implementar os comandos Tauri `agent_create`, `agent_restart`, `agent_duplicate`, `agent_stop` e o evento de status (`working`/`idle`/`exited`) em app/src-tauri/src/commands/agent.rs
- [ ] T058 [US1] Implementar clique em `caminho:linha` (abre no `$EDITOR`) e em URL (abre via `xdg-open`) em app/ui/src/nodes/terminalLinks.ts e app/src-tauri/src/commands/open.rs
- [ ] T059 [US1] Implementar a tela inicial (novo workspace escolhendo pasta, abrir, recentes) em app/ui/src/workspace/StartScreen.tsx

**Checkpoint**: US1 funciona sozinha: agentes reais no canvas, navegáveis, com status.

---

## Phase 4: User Story 2 - Ligar agentes para que conversem entre si (Priority: P1)

**Goal**: conexões com direção criadas só pelo usuário; mensagens via `send`, `read`, `ask`/`reply`, com sanitização, limites, auditoria e histórico.

**Independent Test**: dois agentes com shells, ligar `planner → coder`, `atlasoq send`, `atlasoq read`, `atlasoq ask` + `atlasoq reply`, pausar e verificar recusa (spec, US2).

### Tests for User Story 2

- [ ] T060 [P] [US2] Teste de sanitização com conjunto de payloads (ESC, CSI, OSC, DCS, C0/C1, Ctrl-C, `\r\n`) verificando 100% neutralizados (SC-006) em crates/atlasoq-link/tests/sanitize.rs
- [ ] T061 [P] [US2] Teste de autenticação: uid diferente, token errado, nó inexistente e `v` não suportado são recusados em crates/atlasoq-link/tests/auth.rs
- [ ] T062 [P] [US2] Teste de integração com dois PTYs reais: criar conexão `forward`, `send` entregue com cabeçalho, sentido inverso recusado, pausa recusa imediatamente, apagar recusa em tests/integration/two_ptys_connection.rs
- [ ] T063 [P] [US2] Teste de integração do `ask`/`reply`: resposta exata devolvida; tempo esgotado devolve tela com código 5; `reply` sem pergunta (código 3), ambíguo (código 2) e expirado (código 3); cancelamento por pausa/encerramento em tests/integration/ask_reply.rs
- [ ] T064 [P] [US2] Teste da proteção contra loops: 20 trocas sem entrada humana pausam a conexão (SC-007); tecla do usuário zera; limite de taxa; ping-pong em crates/atlasoq-link/tests/loop_guard.rs
- [ ] T065 [P] [US2] Testes de contrato da CLI com `assert_cmd` + `insta` para `list`, `send`, `read`, `ask`, `reply`, `help`, códigos de saída 0/2/3/4/5/6/7, mensagens de erro, `--json`, texto por stdin e sugestão de comando com distância de edição ≤ 2 (SC-004) em crates/atlasoq-cli/tests/contract.rs

### Implementation for User Story 2

- [ ] T066 [P] [US2] Implementar a sanitização (remover C0/C1 exceto `\n` e `\t`, remover ESC/CSI/OSC/DCS, normalizar quebras) e o limite de 16 KiB em crates/atlasoq-link/src/sanitize.rs
- [ ] T067 [P] [US2] Implementar o servidor de socket Unix em `$XDG_RUNTIME_DIR/atlasoq/<instância>.sock` (diretório `0700`, socket `0600`), uma requisição por conexão, linhas de até 64 KiB em crates/atlasoq-link/src/server.rs
- [ ] T068 [US2] Implementar autenticação (`SO_PEERCRED` mesmo uid, token de 256 bits por spawn comparado em tempo constante, versão) e a geração de `ATLASOQ_SOCKET`, `ATLASOQ_NODE_ID`, `ATLASOQ_NODE_TOKEN` no ambiente de cada agente em crates/atlasoq-link/src/auth.rs
- [ ] T069 [US2] Implementar a ACL a partir das conexões ativas (A → B: A envia, pergunta e lê B; B só responde a `ask` pendente de A; ↔: ambos) em crates/atlasoq-link/src/acl.rs
- [ ] T070 [US2] Implementar a entrega: cabeçalho `[atlasoq de:<origem> msg:<id>]`, bracketed paste quando o destino ativou o modo 2004, `\r` após 150 ms, fila serial por destino em crates/atlasoq-link/src/delivery.rs
- [ ] T071 [US2] Implementar `PendingAsk` (`waiting` → `answered` | `timed_out` | `cancelled`), formato `[atlasoq ask de:<origem> id:<n>] <texto> — responda com: atlasoq reply "..."`, timeout padrão de 600 s, `reply` só para pergunta pendente e entregue ao `ask` que espera em crates/atlasoq-link/src/ask.rs
- [ ] T072 [US2] Implementar a proteção contra loops (`ExchangeCounter` com janela de 120 s, zera com tecla do usuário, limite de taxa com rajada, ping-pong por repetição) e a transição para `auto_paused` com `pause_reason` em crates/atlasoq-link/src/loop_guard.rs
- [ ] T073 [US2] Implementar o log de auditoria JSONL diário em `.atlasoq/logs/links-AAAA-MM-DD.jsonl` (campos do `AuditEvent`, texto ou `text_sha256` conforme configuração) em crates/atlasoq-link/src/audit.rs
- [ ] T074 [US2] Implementar os handlers `list`, `send`, `read`, `ask`, `reply` do broker em crates/atlasoq-link/src/handlers.rs
- [ ] T075 [US2] Implementar a CLI `atlasoq` com os comandos `list`, `send`, `read`, `ask`, `reply`, `help`, saída texto/`--json`, texto por argumento ou stdin, erros no formato "erro: … / próximo passo" conforme contracts/cli.md em crates/atlasoq-cli/src/main.rs e crates/atlasoq-cli/src/commands/
- [ ] T076 [US2] Implementar a sugestão de comando por distância de edição em crates/atlasoq-cli/src/suggest.rs
- [ ] T077 [US2] Iniciar o broker com o app, colocar o diretório da CLI no início do `PATH` dos agentes e expor eventos de mensagem/estado de conexão à UI em app/src-tauri/src/broker.rs
- [ ] T078 [US2] Implementar os comandos Tauri `connection_create` (sem auto-conexão), `connection_set_direction`, `connection_pause`, `connection_resume`, `connection_delete`, `connection_history` em app/src-tauri/src/commands/connection.rs
- [ ] T079 [P] [US2] Implementar as linhas SVG das conexões com direção, estado (ativa, pausada, pausada automaticamente) e animação de passagem de mensagem em app/ui/src/connections/ConnectionLayer.tsx
- [ ] T080 [US2] Implementar a criação de conexão arrastando de um nó a outro e o menu (direção, pausar, reativar, apagar) em app/ui/src/connections/connect.ts
- [ ] T081 [US2] Implementar o painel de histórico da conexão (horário, origem, destino, texto, resultado) e o aviso de pausa automática em app/ui/src/connections/HistoryPanel.tsx

**Checkpoint**: US1 + US2: agentes conversam por conexões seguras e auditáveis.

---

## Phase 5: User Story 3 - Agentes nascem sabendo usar o atlasoq (Priority: P1)

**Goal**: hook de inicialização automático (modos `hook`, `prompt`, `type`, `off`) entrega o tutorial; `learn` e `help` completos; documentação para agentes e usuários.

**Independent Test**: criar um agente real ligado a outro e, sem nenhuma ação, pedir "pergunte ao coder quantos testes existem"; o agente usa `atlasoq ask` (spec, US3).

### Tests for User Story 3

- [ ] T082 [P] [US3] Criar CLIs falsas que registram o que receberam (via hook `SessionStart` simulado, argumento de prompt e entrada digitada) em tests/fake-agents/
- [ ] T083 [P] [US3] Teste por modo: agente recém-criado recebe o conteúdo de aprendizado em `hook`, `prompt` e `type` em até 30 s após ficar pronto (SC-002a); `off` não recebe; falha por tempo-limite marca `failed`; a entrega do modo `type` aparece no log como mensagem do app em tests/integration/learn_modes.rs
- [ ] T140 [P] [US3] Testes de contrato de `atlasoq learn` (normal, `--hook`, fora do app com código 0, limite de 9.000 caracteres) e de `atlasoq help` com exemplo de cada comando em crates/atlasoq-cli/tests/contract_learn.rs
- [ ] T084 [P] [US3] Teste do tutorial: extrai cada bloco de exemplo de docs/agentes/tutorial.md e executa contra dois agentes conectados, verificando o efeito; falha se o tutorial passar de 60 linhas (SC-003) em tests/tutorial/run_examples.rs
- [ ] T085 [P] [US3] Teste de que o hook não escreve fora de `.atlasoq/` (snapshot do diretório raiz antes e depois) em tests/integration/learn_no_side_effects.rs

### Implementation for User Story 3

- [ ] T086 [P] [US3] Escrever o tutorial para agentes (≤ 60 linhas, afirmações factuais, `list` primeiro, um exemplo por comando, `reply` para perguntas, regras, erros comuns) conforme o esqueleto de contracts/cli.md em docs/agentes/tutorial.md
- [ ] T087 [P] [US3] Escrever o guia para usuários com receitas (coordenador → executores, executor ↔ revisor, nota de plano compartilhada, como escrever papéis, como liberar `atlasoq` nas permissões de cada CLI) em docs/guia-orquestracao.md
- [ ] T088 [US3] Montar o conteúdo de aprendizado na ordem do contrato (identidade, conexões, tutorial, `learn.md` até 4 KiB, espaço para memória), limitado a 9.000 caracteres em crates/atlasoq-link/src/learn_content.rs
- [ ] T089 [US3] Implementar o handler `learn` (normal e `--hook`, que registra `learn: done`) e o comando `atlasoq learn` com o tutorial embutido via `include_str!` e saída genérica fora do app em crates/atlasoq-link/src/handlers.rs e crates/atlasoq-cli/src/commands/learn.rs
- [ ] T090 [US3] Implementar o modo `hook`: gerar `.atlasoq/agents/<nome>/claude-settings.json` com `SessionStart` (matchers `startup`, `resume`, `clear`, `compact`) → `atlasoq learn --hook` e `permissions.allow` para `atlasoq`, e acrescentar `--settings <arquivo>` ao comando em crates/atlasoq-link/src/learn_hook.rs
- [ ] T091 [US3] Implementar o modo `prompt`: passar o conteúdo de aprendizado como argumento de prompt inicial conforme o perfil (ex.: `codex "<texto>"`, `gemini --prompt-interactive "<texto>"`, `opencode --prompt "<texto>"`) em crates/atlasoq-link/src/learn_prompt.rs
- [ ] T092 [US3] Implementar o modo `type`: esperar a primeira transição para `idle` (≤ `ready_timeout_s`, padrão 60), não entregar se o usuário digitou nos últimos 2 s, entregar pelo caminho normal de mensagens em crates/atlasoq-link/src/learn_type.rs
- [ ] T093 [US3] Implementar `LearnState` (`pending`, `done`, `failed(motivo)`, `off`), disparo ao criar, reiniciar e reabrir, e registro no log em crates/atlasoq-link/src/learn.rs
- [ ] T094 [US3] Mostrar o estado do aprendizado no nó (`aprendendo…`, `pronto`, `aprendizado falhou: <motivo>`) em app/ui/src/nodes/LearnBadge.tsx
- [ ] T095 [US3] Implementar `atlasoq help [comando]` com uso e exemplo de cada um dos 8 comandos em crates/atlasoq-cli/src/commands/help.rs

**Checkpoint**: US1–US3 entregam o núcleo do orquestrador: agentes que nascem prontos para conversar.

---

## Phase 6: User Story 4 - Notas como contexto compartilhado (Priority: P2)

**Goal**: notas Markdown no canvas, em arquivos, ligáveis a agentes com leitura e acréscimo.

**Independent Test**: criar a nota `plano`, ligar a dois agentes, `atlasoq note plano` e `atlasoq note plano "feito: testes"` (spec, US4).

### Tests for User Story 4

- [ ] T096 [P] [US4] Teste de integração de `note_read`/`note_append`: leitura com qualquer conexão, acréscimo só com agente → nota ou ↔, recusa sem conexão, formato `- AAAA-MM-DD HH:MM [<agente>] <texto>` em tests/integration/notes.rs
- [ ] T141 [P] [US4] Testes de contrato de `atlasoq note` (leitura, acréscimo, recusas com códigos 3 e 7, mensagens de erro, `--json`) em crates/atlasoq-cli/tests/contract_note.rs
- [ ] T097 [P] [US4] Teste de alteração externa: nota editada fora do app é recarregada; conflito com edição não salva mostra as duas versões em crates/atlasoq-store/tests/notes_watch.rs

### Implementation for User Story 4

- [ ] T098 [P] [US4] Implementar notas em `notes/<nome>.md` com escrita atômica e acréscimo no fim em crates/atlasoq-store/src/notes.rs
- [ ] T099 [US4] Implementar a observação de `notes/` com `notify` e a detecção de conflito em crates/atlasoq-store/src/watch.rs
- [ ] T100 [US4] Implementar os handlers `note_read` e `note_append` com a ACL agente↔nota e o comando `atlasoq note <nota> [texto]` em crates/atlasoq-link/src/handlers.rs e crates/atlasoq-cli/src/commands/note.rs
- [ ] T101 [P] [US4] Implementar `NoteNode` com CodeMirror 6 (edição) e markdown-it + DOMPurify (leitura, CommonMark + tabelas, listas de tarefas, blocos de código), links abrindo no navegador em app/ui/src/nodes/NoteNode.tsx
- [ ] T102 [US4] Implementar os comandos Tauri `note_create`, `note_save`, `note_conflict_resolve` e o evento de recarga em app/src-tauri/src/commands/note.rs
- [ ] T103 [US4] Permitir conexões agente↔nota na UI (sem nota↔nota) com direções de leitura/acréscimo em app/ui/src/connections/connect.ts

**Checkpoint**: notas funcionam sozinhas e como contexto compartilhado entre agentes.

---

## Phase 7: User Story 5 - Retomar o workspace depois de fechar o app (Priority: P2)

**Goal**: reabrir exatamente como estava, com agentes iniciando sozinhos, backups, segredos fora do disco e aprovação de workspaces de terceiros.

**Independent Test**: montar workspace com agentes, notas e conexões; encerrar à força; reabrir e comparar (spec, US5).

### Tests for User Story 5

- [ ] T104 [P] [US5] Teste de 50 encerramentos forçados durante gravação: workspace reabre íntegro em 100% dos casos (SC-009) em tests/integration/kill9_persistence.rs
- [ ] T105 [P] [US5] Teste de órfãos: após fechar e após `kill -9` do app, nenhum processo de agente permanece (SC-010) em tests/integration/no_orphans.rs
- [ ] T106 [P] [US5] Teste de confiança: workspace criado nesta máquina abre sem diálogo; workspace copiado de outra raiz ou com comando alterado exige aprovação em crates/atlasoq-store/tests/trust.rs
- [ ] T107 [P] [US5] Teste de segredos: valores de `env_secret_keys` nunca aparecem em arquivos de `.atlasoq/` em crates/atlasoq-store/tests/secrets.rs

### Implementation for User Story 5

- [ ] T108 [P] [US5] Implementar backups (10 últimas versões em `backups/`) e restauração do backup válido mais recente quando `workspace.json` estiver ilegível em crates/atlasoq-store/src/backup.rs
- [ ] T109 [P] [US5] Implementar migrações por `schema` (estrutura pronta, schema 1 como base) em crates/atlasoq-store/src/migrate.rs
- [ ] T110 [P] [US5] Implementar segredos via keyring (Secret Service) com fallback para o ambiente do app em crates/atlasoq-store/src/secrets.rs
- [ ] T111 [US5] Implementar a confiança: SHA-256 de comandos, args, env não secreto e modo de hook dos agentes, perfis do workspace e `learn.md`; registro em `~/.local/state/atlasoq/trust.json` por caminho canônico; workspace criado localmente já aprovado em crates/atlasoq-store/src/trust.rs
- [ ] T112 [US5] Implementar a reabertura: restaurar viewport, nós e conexões e iniciar todos os agentes automaticamente (após aprovação, se exigida), disparando o hook de aprendizado em app/src-tauri/src/commands/workspace.rs
- [ ] T113 [US5] Implementar o diálogo de aprovação listando comandos, perfis, hooks e `learn.md` em app/ui/src/workspace/TrustDialog.tsx
- [ ] T114 [US5] Implementar a lista de recentes em `~/.local/state/atlasoq/recent.json` e a troca entre workspaces em crates/atlasoq-store/src/recent.rs e app/ui/src/workspace/StartScreen.tsx

**Checkpoint**: fechar, travar ou reabrir não perde a organização do time.

---

## Phase 8: User Story 7 - Agentes com memória de longo prazo (Priority: P2)

**Goal**: memória por agente em `memory/<nome>.md`, escrita com `atlasoq note memory`, entregue no nascimento com limite de tamanho, fora do git por padrão.

**Independent Test**: `coder` registra duas anotações, fechar o app, reabrir; o `coder` recebe as anotações ao nascer (spec, US7).

### Tests for User Story 7

- [ ] T115 [P] [US7] Teste de memória: prefixo obrigatório (`feito:`, `decisão:`, `pendente:`), limite de 300 caracteres, data e hora acrescentadas, recusa de segredos (código 7), outro agente não acessa (código 3) em tests/integration/memory.rs
- [ ] T116 [P] [US7] Teste de entrega: só as 20 anotações mais recentes dentro de 2 KiB, aviso de anotações antigas, total de aprendizado ≤ 9.000 caracteres (SC-012) em crates/atlasoq-link/tests/learn_memory.rs
- [ ] T142 [P] [US7] Testes de contrato de `atlasoq note memory` (prefixo obrigatório, 300 caracteres, segredo recusado com código 7, acesso de outro agente recusado com código 3) em crates/atlasoq-cli/tests/contract_memory.rs
- [ ] T117 [P] [US7] Teste do botão de versionamento: remove e recoloca só a linha `# atlasoq:memory`, preservando o resto do `.gitignore` em crates/atlasoq-store/tests/gitignore.rs

### Implementation for User Story 7

- [ ] T118 [P] [US7] Implementar `MemoryEntry` e o arquivo `memory/<agente>.md` (linhas `- AAAA-MM-DD HH:MM <tipo>: <texto>`) com escrita atômica em crates/atlasoq-store/src/memory.rs
- [ ] T119 [P] [US7] Implementar a detecção de segredos (prefixos de chaves conhecidas, `password=`, blocos PEM, tokens longos de alta entropia) em crates/atlasoq-core/src/secrets_detect.rs
- [ ] T120 [US7] Tratar o nome reservado `memory` em `note_read`/`note_append` como a memória de quem executa, com validação de prefixo e tamanho, em crates/atlasoq-link/src/handlers.rs
- [ ] T121 [US7] Incluir a memória recente no conteúdo de aprendizado (até 20 anotações / 2 KiB, linha de aviso) em crates/atlasoq-link/src/learn_content.rs
- [ ] T122 [US7] Acrescentar ao tutorial a seção de memória (o que anotar, prefixos, quando; não abrir sessão perguntando se deve continuar) mantendo ≤ 60 linhas em docs/agentes/tutorial.md
- [ ] T123 [US7] Mover a memória ao renomear o agente e perguntar ao apagar (padrão: manter) em crates/atlasoq-store/src/memory.rs e app/ui/src/nodes/AgentNode.tsx
- [ ] T124 [US7] Implementar o painel de memória no nó (ver, editar, apagar anotações) em app/ui/src/nodes/MemoryPanel.tsx
- [ ] T125 [US7] Implementar "Versionar memória"/"Parar de versionar" (edita só a linha `# atlasoq:memory`, aviso sobre histórico já commitado) e `settings.memory_versioned` em crates/atlasoq-store/src/gitignore.rs e app/ui/src/workspace/SettingsPanel.tsx

**Checkpoint**: agentes lembram o que fizeram entre reinícios e dias, com custo fixo de tokens.

---

## Phase 9: User Story 6 - Instalar pelo pacote .deb (Priority: P3)

**Goal**: `.deb` com app no menu, ícone próprio e `atlasoq` no PATH.

**Independent Test**: instalar o `.deb` em máquina limpa, abrir pelo menu, criar agente, rodar `atlasoq list`, desinstalar (spec, US6).

- [ ] T126 [P] [US6] Criar o arquivo de desktop com nome "AtlasOQ", `StartupWMClass` igual ao `app_id` e categorias em packaging/atlasoq.desktop
- [ ] T127 [P] [US6] Criar o ícone próprio (SVG + PNGs 32/64/128/256/512) em packaging/icons/
- [ ] T128 [US6] Configurar o bundle `.deb` (dependências `libwebkit2gtk-4.1-0`, `libgtk-3-0`; `externalBin` `atlasoq` em `/usr/bin`; desktop e ícones) conforme docs/spikes/r13-deb.md em app/src-tauri/tauri.conf.json
- [ ] T129 [US6] Job de CI que gera o `.deb` e roda instalar → abrir → `atlasoq list` → desinstalar em containers Ubuntu 26.04, Ubuntu 24.04 e Debian 13 (SC-011) em .github/workflows/deb.yml
- [ ] T130 [US6] Verificar que remover o pacote mantém os workspaces do usuário intactos e documentar instalação/remoção em docs/instalacao.md

**Checkpoint**: release instalável.

---

## Phase 10: Polish & Cross-Cutting Concerns

**Purpose**: qualidade transversal, desempenho, segurança e validação final.

- [ ] T131 [P] Testes E2E em Xvfb (X11) e Weston headless (Wayland) cobrindo o roteiro do quickstart em app/ui/tests/e2e/quickstart.e2e.ts e .github/workflows/e2e.yml
- [ ] T132 [P] Teste de carga: 10 agentes ociosos com pan/zoom ≥ 50 fps e 8 agentes em saída contínua com resposta da UI < 100 ms (SC-008) em app/ui/tests/perf/load.e2e.ts
- [ ] T133 [P] Escrever o modelo de ameaças (conexões, injeção de prompt em cadeia, workspace malicioso, tokens, mesmo uid fora do escopo) em docs/threat-model.md
- [ ] T134 [P] Registrar a licença de cada dependência em docs/licenses.md
- [ ] T135 [P] Teste de IME (ibus e fcitx5) nos terminais e notas, documentando limitações em docs/limitacoes.md
- [ ] T143 [P] Garantir FR-042 (sem rede e sem telemetria): CSP do Tauri sem origens remotas, nenhum plugin de rede, e teste que falha se o app abrir conexão de saída (rodando em namespace de rede isolado) em app/src-tauri/tauri.conf.json e tests/integration/no_network.rs
- [ ] T144 Checklist manual de release em GNOME e KDE, cada um em Wayland e X11 (canvas, terminais, zoom, IME, clipboard, menu e ícone), executado e registrado em docs/medicoes/plataformas-mvp.md (FR-040)
- [ ] T145 Teste de usabilidade com pelo menos 3 pessoas que não conhecem o app: instalar o `.deb`, criar dois agentes, conectá-los e trocar uma mensagem em menos de 5 minutos sem documentação externa (SC-001), registrado em docs/medicoes/usabilidade-mvp.md
- [ ] T136 Revisão de segurança explícita das mudanças em conexões, consentimento e transmissão (constituição) registrada em docs/revisoes/seguranca-mvp.md
- [ ] T137 Medir SC-002 e SC-013 com pelo menos duas CLIs de agente reais (10 tentativas cada) e registrar em docs/medicoes/mvp.md
- [ ] T138 Atualizar README com GIF de demonstração, arquitetura, CLI e limitações conhecidas em README.md
- [ ] T139 Executar o roteiro completo de specs/001-mvp-orquestracao-agentes/quickstart.md e registrar o resultado em docs/medicoes/quickstart-mvp.md

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: sem dependências. O T014 (spike de zoom) é um portão: se reprovar, revisar SPEC.md antes da Fase 3.
- **Foundational (Phase 2)**: depende da Fase 1; bloqueia todas as histórias.
- **US1 (Phase 3)**: depende da Fase 2.
- **US2 (Phase 4)**: depende da Fase 2; usa nós de agente da US1 para a UI de conexões (T079–T081). O broker e a CLI (T060–T076) podem começar em paralelo à US1.
- **US3 (Phase 5)**: depende da US2 (broker, socket, entrega).
- **US4 (Phase 6)**: depende da US2 (ACL e CLI) para `atlasoq note`; o `NoteNode` (T101) pode começar após a Fase 2.
- **US5 (Phase 7)**: depende da US1; T112 usa o hook da US3.
- **US7 (Phase 8)**: depende da US3 (conteúdo de aprendizado) e da US4 (`atlasoq note`).
- **US6 (Phase 9)**: depende do T016; empacotamento final após US1–US3.
- **Polish (Phase 10)**: depois das histórias desejadas.

### Within Each User Story

- Testes primeiro, falhando; depois tipos e armazenamento; depois broker/CLI; depois UI.
- Commit ao fim de cada tarefa ou grupo lógico, com prefixo `ATQ-001`.

### Parallel Opportunities

- Fase 1: T003, T005–T008, T010–T013 em paralelo; spikes T015 e T016 em paralelo ao T014.
- Fase 2: T017–T020 em paralelo; testes T021–T024 e T031–T032 em paralelo; T043–T044 em paralelo ao backend.
- US1 e o backend da US2 em paralelo (pessoas ou agentes diferentes).
- Dentro da US2: T060–T065 em paralelo; T066, T067 e T079 em paralelo.
- US4 (T098, T101) em paralelo com US3.
- US5 (T104–T111) em paralelo com US3 e US4.

---

## Parallel Example: User Story 2

```bash
# Testes da US2 juntos:
Task: "Teste de sanitização em crates/atlasoq-link/tests/sanitize.rs"
Task: "Teste de autenticação em crates/atlasoq-link/tests/auth.rs"
Task: "Teste com dois PTYs em tests/integration/two_ptys_connection.rs"
Task: "Teste de ask/reply em tests/integration/ask_reply.rs"
Task: "Teste de loop guard em crates/atlasoq-link/tests/loop_guard.rs"
Task: "Contratos da CLI em crates/atlasoq-cli/tests/contract.rs"

# Implementação independente:
Task: "Sanitização em crates/atlasoq-link/src/sanitize.rs"
Task: "Servidor de socket em crates/atlasoq-link/src/server.rs"
Task: "Camada SVG de conexões em app/ui/src/connections/ConnectionLayer.tsx"
```

---

## Implementation Strategy

### MVP First (User Story 1)

1. Fase 1 (incluindo o portão do T014) → Fase 2 → Fase 3 (US1).
2. **Parar e validar**: agentes reais no canvas com pan/zoom e status.

### Incremental Delivery

1. US1 → canvas com agentes (demonstrável).
2. US2 → agentes conversam (núcleo do orquestrador).
3. US3 → agentes nascem sabendo usar a CLI (orquestração sem passos manuais).
4. US4 + US5 + US7 → contexto compartilhado, retomada e memória.
5. US6 → `.deb` e release v0.1.0 após a Fase 10.

### Mapeamento para os milestones do SPEC.md

| Milestone | Tarefas |
|---|---|
| M0 | T001–T016 |
| M1 | T017–T030 |
| M2 | T031–T044 (núcleo do store e Tauri base), T045–T059 |
| M3 | T096–T114 |
| M4 | T060–T095, T115–T125 |
| M4 | (também) T140–T142 |
| M5 | T126–T139, T143–T145 |

---

## Notes

- [P] = arquivos diferentes, sem dependência de tarefa incompleta.
- Cada história pode ser validada sozinha pelo seu Independent Test.
- Nenhuma tarefa escreve fora de `.atlasoq/` no projeto do usuário (princípio IX).
- T140–T145 foram acrescentadas após o `/speckit-analyze` e mantêm IDs novos para não renumerar
  as tarefas existentes; a posição no arquivo indica a ordem de execução.
- Mudança na CLI exige atualizar docs/agentes/tutorial.md no mesmo PR (princípio IX).
