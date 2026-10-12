# Implementation Plan: MVP do AtlasOQ — canvas para orquestrar agentes de IA

**Branch**: `ATQ-001` | **Date**: 2026-10-11 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/001-mvp-orquestracao-agentes/spec.md`

## Summary

App desktop Linux em que cada agente de IA (Claude Code, Codex, Gemini CLI, OpenCode, Aider ou
shell) roda num PTY real, exibido como nó de um canvas infinito, ao lado de notas em Markdown. O
usuário liga agentes por conexões com direção; os agentes conversam pela CLI `atlasoq` (8 comandos:
`learn`, `list`, `send`, `read`, `ask`, `reply`, `note`, `help`), mediada por um broker que aplica
ACL, sanitização, limites contra loops e auditoria. Cada agente nasce sabendo usar a CLI: um hook
de inicialização entrega tutorial e memória recente (modos `hook`, `prompt`, `type`, `off`). Tudo é
salvo em `<raiz>/.atlasoq/`. Distribuição em `.deb`.

Abordagem técnica: Tauri v2 com UI TypeScript/React e núcleo Rust em crates independentes do Tauri
(PTY, broker, armazenamento, CLI). Decisões detalhadas em [research.md](research.md).

## Technical Context

**Language/Version**: Rust estável (edição 2021, versão mínima fixada no M0); TypeScript 5 (strict)

**Primary Dependencies**: Tauri v2; `portable-pty`; `vt100`; `tokio`; `serde`/`serde_json`;
`notify`; `nix` (setsid, prctl, flock, SO_PEERCRED); `keyring` (Secret Service); `ulid`; `sha2`;
React 18; Zustand; `@xterm/xterm` + addons; CodeMirror 6; `markdown-it`; DOMPurify; Vite

**Storage**: arquivos locais em `<raiz>/.atlasoq/` (JSON + Markdown + JSONL); estado do usuário em
`~/.config/atlasoq/` e `~/.local/state/atlasoq/`

**Testing**: `cargo test` (unidade, integração com PTYs reais), `assert_cmd` + `insta` (contratos
da CLI), Vitest (UI), WebdriverIO + `tauri-driver` (E2E em Xvfb e Weston headless), CLIs falsas
para os modos de hook, teste de exemplos do tutorial

**Target Platform**: Linux x86_64; Ubuntu 26.04 LTS, Ubuntu 24.04 LTS, Debian 13; GNOME e KDE;
Wayland e X11

**Project Type**: desktop-app (Tauri) + CLI + biblioteca Rust interna

**Performance Goals**: ≥ 50 fps em pan/zoom com 10 agentes ociosos; resposta da UI < 100 ms com 8
agentes em saída contínua; CLI `atlasoq` responde em < 50 ms (exceto `ask`); hook entrega o
tutorial em ≤ 30 s após o agente ficar pronto

**Constraints**: offline, sem telemetria; nenhuma escrita fora de `.atlasoq/` (exceto estado do
próprio app em `~/.config` e `~/.local/state`); memória entregue ao agente ≤ 2 KiB; conteúdo de
aprendizado ≤ 9.000 caracteres; sem processos órfãos; escrita atômica

**Scale/Scope**: até ~20 agentes e ~50 notas por workspace como alvo de projeto; 1 usuário; 8
comandos de CLI; 7 histórias de usuário; 52 requisitos funcionais

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Princípio | Verificação | Pré-pesquisa | Pós-design |
|---|---|---|---|
| I. Originalidade e licenciamento | Implementação do zero; vocabulário próprio; dependências MIT/Apache/MPL compatíveis com Apache-2.0; `cargo-deny` e checagem npm no CI | ✅ | ✅ |
| II. Linux primeiro | Alvos Ubuntu/Debian, GNOME/KDE, Wayland/X11; E2E nos dois servidores de display | ✅ | ✅ |
| III. Terminal como PTY real; app não chama IA | `portable-pty`; memória escrita pelo próprio agente, sem LLM no app | ✅ | ✅ |
| IV. Comunicação com consentimento | Conexões só pelo usuário; ACL no broker; sanitização; limites contra loops; auditoria; `reply` restrito a pergunta pendente | ✅ | ✅ |
| V. Persistência local e legível | `.atlasoq/` em JSON/Markdown; escrita atômica; backups; segredos fora | ✅ | ✅ |
| VI. Especificação antes da implementação | SPEC.md v0.3.0 aprovado; spec 001 com clarify | ✅ | ✅ |
| VII. Escopo de MVP | Itens da spec ⊂ escopo da constituição v1.3.0 (inclui memória); sem desenhos, daemon, sandbox | ✅ | ✅ |
| VIII. CLI simples e memorável | 8 comandos, uma palavra cada, nomes visíveis, sem flags no caso comum, erros com próximo passo, `--json` | ✅ | ✅ |
| IX. Agentes aprendem pela própria CLI | Hook automático (4 modos), conteúdo embutido para evitar pedidos de permissão, teste do tutorial e teste por modo, nada fora de `.atlasoq/` | ✅ | ✅ |
| Restrições: socket local | `0600`, `SO_PEERCRED`, token por nó | ✅ | ✅ |
| Restrições: confiança no workspace | Hash + `trust.json` + diálogo | ✅ | ✅ |
| Restrições: stack e `.deb` | Tauri v2 + xterm.js + portable-pty; apenas `.deb` | ✅ | ✅ |
| Fluxo: branch e commits `ATQ-NNN` | Branch `ATQ-001` | ✅ | ✅ |

Nenhuma violação. Pontos de atenção (não violações):
- A regra de permissão `Bash(atlasoq:*)` gerada para o Claude Code fica dentro de
  `.atlasoq/agents/<nome>/` e vale só para aquele processo (R5).
- O nome da branch (`ATQ-001`) difere do nome do diretório da spec (`001-mvp-orquestracao-agentes`);
  o Spec Kit resolve a spec por `.specify/feature.json`.

## Project Structure

### Documentation (this feature)

```text
specs/001-mvp-orquestracao-agentes/
├── spec.md
├── plan.md              # este arquivo
├── research.md          # Fase 0
├── data-model.md        # Fase 1
├── quickstart.md        # Fase 1
├── contracts/           # Fase 1
│   ├── cli.md
│   ├── socket-protocol.md
│   ├── workspace-format.md
│   └── learn-hook.md
└── tasks.md             # Fase 2 (/speckit-tasks)
```

### Source Code (repository root)

```text
Cargo.toml                    # workspace Cargo
crates/
├── atlasoq-core/             # tipos compartilhados: nomes, ids, mensagens, erros, códigos de saída
├── atlasoq-pty/              # SessionHost, LocalSessionHost, parser vt100, status, ciclo de vida
├── atlasoq-store/            # .atlasoq/: formato, validação, escrita atômica, backups, migrações,
│                             #   perfis, notas, memória, .gitignore, trust
├── atlasoq-link/             # broker: socket, autenticação, ACL, sanitização, entrega, ask/reply,
│                             #   limites contra loops, auditoria, hook de aprendizado
└── atlasoq-cli/              # binário `atlasoq`: 8 comandos, sugestões, --json, tutorial embutido
app/
├── src-tauri/                # binário `atlasoq-desktop`: comandos Tauri, canais de saída, bundle .deb
└── ui/
    ├── src/
    │   ├── canvas/           # viewport, pan/zoom, seleção, minimapa, undo/redo
    │   ├── nodes/            # AgentNode (xterm.js, LOD, status), NoteNode (CodeMirror, Markdown)
    │   ├── connections/      # linhas SVG, criação, estado, histórico
    │   ├── focus/            # máquina de estados de foco
    │   ├── workspace/        # abrir/criar, aprovação, configurações (Versionar memória)
    │   ├── store/            # Zustand + comandos
    │   └── i18n/             # catálogo de textos (pt-BR)
    └── tests/                # Vitest + E2E (WebdriverIO)
docs/
├── agentes/tutorial.md       # fonte do `atlasoq learn`
├── guia-orquestracao.md      # receitas para usuários
├── threat-model.md
├── licenses.md
└── adr/                      # ADR-001 stack, ADR-002 conexões, ADR-003 .atlasoq e hooks
packaging/
├── atlasoq.desktop
└── icons/
tests/
├── integration/              # dois PTYs conectados, ask/reply, loop guard, kill -9, órfãos
├── fake-agents/              # CLIs falsas para os modos hook/prompt/type
└── tutorial/                 # executa os exemplos de docs/agentes/tutorial.md
```

**Structure Decision**: workspace Cargo com cinco crates sem dependência do Tauri (permite o plano B
em Electron e o daemon pós-MVP reutilizando `atlasoq-pty` e `atlasoq-link`), mais o app Tauri em
`app/`. `atlasoq-core` evita dependência circular entre CLI e broker no formato das mensagens.

## Ordem de implementação (insumo para /speckit-tasks)

Segue os milestones do SPEC.md, mapeados às histórias da spec:

| Fase | Conteúdo | Histórias |
|---|---|---|
| M0 | Esqueleto do repositório, CI, ADRs, spikes R3 (zoom), R4/R5 (`--settings`, flags das CLIs), R13 (`externalBin`) | — |
| M1 | `atlasoq-pty` completo + testes | 1 |
| M2 | Canvas, nó de agente, foco, LOD, perfis prontos | 1 |
| M3 | `atlasoq-store`, notas, reabertura automática, aprovação de workspace | 4, 5 |
| M4 | `atlasoq-link`, `atlasoq-cli`, conexões na UI, hook de aprendizado, memória, tutorial e guia | 2, 3, 7 |
| M5 | `.deb`, E2E em Wayland/X11, carga, threat model, medições SC-002/SC-013 com CLIs reais | 6 |

## Complexity Tracking

Sem violações da constituição a justificar.
