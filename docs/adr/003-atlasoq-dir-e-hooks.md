# ADR-003: Diretório `.atlasoq/`, hook de aprendizado e memória

- **Status:** aceito (2026-10-11)
- **Contexto:** agentes devem nascer sabendo usar a CLI, sem passo manual (princípio IX), lembrar
  o que fizeram entre reinícios e o app não pode alterar arquivos fora do seu diretório.
- **Decisão:**
  - Workspace em `<raiz>/.atlasoq/`: `workspace.json`, `notes/`, `profiles/` e `learn.md`
    versionáveis; `agents/`, `logs/`, `backups/`, `.lock` e, por padrão, `memory/` ignorados pelo
    git via `.atlasoq/.gitignore` gerado.
  - Hook de inicialização por perfil, em quatro modos: `hook` (hook de sessão da própria CLI, com
    configuração gerada em `.atlasoq/agents/<nome>/`), `prompt` (tutorial e memória como prompt
    inicial), `type` (mensagem do app ao agente após ficar pronto) e `off`.
  - Conteúdo entregue: identidade, conexões, tutorial (≤ 60 linhas), `learn.md` (≤ 4 KiB) e as
    últimas 20 anotações da memória (≤ 2 KiB), no máximo 9.000 caracteres.
  - Memória por agente em `memory/<nome>.md`, escrita pelo próprio agente com
    `atlasoq note memory "feito|decisão|pendente: ..."`.
  - Workspaces vindos de outra máquina exigem aprovação antes de executar comandos e hooks.
- **Consequências:** custo de tokens fixo e baixo; nenhuma alteração em `AGENTS.md`, `CLAUDE.md` ou
  configuração global das CLIs; suporte a hooks nativos limitado a CLIs que aceitam configuração
  por flag (Claude Code), as demais usam `prompt` ou `type`.
- **Alternativas:** escrever instruções no repositório (proibido); botão "Ensinar" (passo manual,
  rejeitado pelo usuário); resumos gerados pelo app (o app não chama IA, princípio III).
