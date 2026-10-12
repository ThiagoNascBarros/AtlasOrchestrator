# AtlasOQ

Canvas infinito para orquestrar agentes de IA no Linux. Cada agente (Claude Code, Codex, Gemini
CLI, OpenCode, Aider ou um shell) roda num terminal real, representado por um nó no canvas, ao lado
de notas em Markdown. Você liga dois agentes com uma linha, e eles passam a conversar pela CLI
`atlasoq`. Todo agente já nasce sabendo usá-la: logo após iniciar, um hook roda o `atlasoq learn`.

> **Status:** fase de especificação, sem código ainda. SPEC.md v0.3.0 aprovado (nome, stack e
> empacotamento); a spec do MVP está em [specs/001](specs/001-mvp-orquestracao-agentes/spec.md).

## Sobre o projeto

AtlasOQ é um projeto open source, implementado do zero, inspirado no conceito de um "canvas
infinito" para orquestrar vários agentes de código. Ele não copia nome, identidade visual nem
código de nenhum aplicativo existente.

### Escopo do MVP

- Canvas infinito com pan e zoom
- Agentes em nós de terminal, cada um com um PTY real, nome, papel e status
- Notas em Markdown, que podem ser ligadas a agentes como contexto compartilhado
- Conexões entre agentes, sempre criadas pelo usuário, com direção, limites e auditoria
- CLI `atlasoq` com 8 comandos e tutorial que cada agente recebe automaticamente ao iniciar
- Persistência local do workspace em `<projeto>/.atlasoq/`
- Instalação por pacote `.deb`

### A CLI dos agentes

```
atlasoq learn                    # tutorial para o agente
atlasoq me                       # quem sou eu
atlasoq list                     # com quem estou ligado
atlasoq send coder "rode os testes"
atlasoq read coder               # o que está na tela do coder
atlasoq ask coder "quantos testes falharam?"
atlasoq note plano               # lê a nota "plano"
atlasoq note plano "feito: testes"
atlasoq help
```

### Stack

Tauri v2 (UI em TypeScript), xterm.js para os terminais e núcleo em Rust com `portable-pty`. Ver
[SPEC.md](SPEC.md), seção 4.

### O diretório `.atlasoq/`

```
<projeto>/.atlasoq/
├── workspace.json    # canvas, agentes e conexões
├── notes/            # notas em Markdown
├── profiles/         # perfis de agente do workspace
├── learn.md          # regras do time, anexadas ao tutorial
├── agents/           # gerado pelo app (hooks), ignorado pelo git
├── logs/             # auditoria, ignorado pelo git
└── backups/          # ignorado pelo git
```

### Princípios

Resumo da [constituição](.specify/memory/constitution.md) v1.2.0:

1. Originalidade e licenciamento
2. Linux primeiro (X11 e Wayland)
3. Terminal como PTY real
4. Comunicação entre agentes com consentimento explícito
5. Persistência local e legível
6. Especificação antes da implementação
7. Escopo de MVP
8. CLI simples e memorável
9. Agentes aprendem pela própria CLI

## Estrutura de pastas

### Atual

```
AtlasOrchestrator/
├── .claude/skills/        # Skills do Spec Kit usadas pelo Claude Code
├── .specify/
│   ├── memory/
│   │   └── constitution.md   # Constituição do projeto
│   ├── templates/            # Templates de spec, plano, tarefas e checklist
│   ├── scripts/              # Scripts do Spec Kit
│   ├── workflows/            # Workflows do Spec Kit
│   └── integrations/         # Manifestos de integração
├── specs/
│   └── 001-mvp-orquestracao-agentes/
│       └── spec.md           # Spec do MVP (ATQ-001)
├── SPEC.md                   # Especificação geral, arquitetura, riscos e milestones
├── LICENSE                   # Apache-2.0
├── .gitignore
└── README.md
```

### Planejada

```
AtlasOrchestrator/
├── SPEC.md
├── specs/                   # Specs por funcionalidade (Spec Kit)
├── docs/                    # ADRs, threat model, tutorial para agentes, guia de orquestração
├── crates/
│   ├── atlasoq-pty/         # Camada PTY
│   ├── atlasoq-link/        # Orquestração agente-a-agente
│   ├── atlasoq-store/       # Persistência do workspace
│   └── atlasoq-cli/         # CLI `atlasoq`
├── app/
│   ├── src-tauri/           # Integração Tauri
│   └── ui/                  # Canvas, nós, conexões e notas
├── packaging/               # .deb, .desktop, ícones
└── tests/                   # Integração e E2E
```

## Como contribuir

Antes de propor código, leia a [constituição](.specify/memory/constitution.md). O trabalho acontece
em branches `ATQ-NNN`, e cada commit começa com o identificador da branch (ex.: `ATQ-001 ...`).
Mudanças de escopo passam primeiro pelo SPEC.md.

## Licença

Distribuído sob a [Apache License 2.0](LICENSE).
