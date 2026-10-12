# AtlasOrchestrator

Canvas infinito para agentes de código no Linux. Cada terminal é um nó posicionável no canvas, ao
lado de notas e desenhos. Dois terminais podem ser conectados por uma linha para que os agentes
conversem entre si via PTY.

> **Status:** fase de especificação. Ainda não há código. O SPEC.md precisa ser aprovado antes da
> implementação (ver [constituição](.specify/memory/constitution.md), princípio VI).

## Sobre o projeto

AtlasOrchestrator é um projeto open source, implementado do zero, inspirado no conceito de um
"canvas infinito" para orquestrar vários agentes de código. Ele não copia nome, identidade visual
nem código de nenhum aplicativo existente.

### Escopo do MVP

- Canvas infinito com pan e zoom
- Nós de terminal, cada um com um PTY real
- Notas em Markdown
- Conexões entre terminais (comunicação agente-a-agente, sempre com consentimento explícito)
- Persistência local do workspace

### Princípios

Resumo da [constituição](.specify/memory/constitution.md):

1. Originalidade e licenciamento
2. Linux primeiro (X11 e Wayland)
3. Terminal como PTY real
4. Comunicação entre agentes com consentimento explícito
5. Persistência local e legível
6. Especificação antes da implementação
7. Escopo de MVP

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
├── .gitignore
└── README.md
```

### Planejada (sujeita ao SPEC.md)

A stack ainda não foi escolhida. A estrutura abaixo é provisória e será ajustada após a aprovação
do SPEC.md.

```
AtlasOrchestrator/
├── SPEC.md                # Especificação, arquitetura, riscos e milestones
├── specs/                 # Especificações por funcionalidade (Spec Kit)
├── docs/                  # Documentação e ADRs
├── src/
│   ├── pty/               # Camada PTY: criação e ciclo de vida dos terminais
│   ├── canvas/            # Camada de canvas: pan, zoom, nós e conexões
│   ├── orchestration/     # Orquestração agente-a-agente
│   └── storage/           # Armazenamento do workspace
└── tests/                 # Testes unitários e de integração
```

## Como contribuir

Antes de propor código, leia a [constituição](.specify/memory/constitution.md). Mudanças de escopo
devem passar primeiro pelo SPEC.md.

## Licença

A licença ainda será definida.
