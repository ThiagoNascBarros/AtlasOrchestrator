# Contrato: hook de inicialização (aprendizado + memória)

Define o que cada agente recebe ao nascer e como, por modo. Ver decisões em
[research.md](../research.md) R4–R6.

## Conteúdo entregue (todos os modos)

Texto único, em afirmações factuais, nesta ordem:

1. Identidade: `Você é o agente "<nome>" no AtlasOQ. Papel: <papel>.`
2. Ligações atuais (mesma informação do `atlasoq list`).
3. Tutorial (`docs/agentes/tutorial.md`, ≤ 60 linhas).
4. Regras do time (`.atlasoq/learn.md`, ≤ 4 KiB), se existir.
5. Memória: `Suas últimas anotações:` + até 20 anotações / 2 KiB; se houver mais, uma linha
   `(há N anotações mais antigas; leia com atlasoq note memory)`.

Tamanho máximo total: 9.000 caracteres (abaixo do limite de 10.000 do SessionStart do Claude Code).

## Por modo

| Modo | Disparo | Entrega | Concluído quando |
|---|---|---|---|
| `hook` | A CLI do agente executa seu hook de início de sessão | O app gera `.atlasoq/agents/<nome>/claude-settings.json` com hook `SessionStart` → `atlasoq learn --hook` e regra de permissão para `atlasoq`; inicia `claude --settings <arquivo>` | `atlasoq learn --hook` é chamado pelo agente |
| `prompt` | Início do processo | Texto passado como argumento de prompt inicial do perfil (ex.: `codex "<texto>"`) | O processo iniciou sem erro |
| `type` | Primeira transição para `idle` após o início (≤ `ready_timeout_s`) | Texto entregue pelo caminho normal de mensagens (sanitizado, bracketed paste, `\r`) | Entrega concluída |
| `off` | — | — | — |

## Regras

- Roda ao criar, ao reiniciar e ao reabrir o workspace. No modo `hook`, também em `resume`, `clear`
  e `compact` da própria CLI (reinjeta tutorial e memória após compactação).
- Modo `type`: se o usuário digitou no agente nos últimos 2 s, espera; se continuar até o
  tempo-limite, falha com `usuário digitando`.
- Falha → `LearnState.failed(motivo)` visível no nó e registrada no log; o agente continua rodando.
- Nenhum arquivo é escrito fora de `.atlasoq/`.
- Itens do workspace que entram no texto (`learn.md`, papéis) passam pela aprovação de workspace.
