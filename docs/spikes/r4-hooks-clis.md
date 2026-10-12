# Spike R4/R5: hooks e permissões das CLIs de agente

Data: 2026-10-11 · Tarefa: T015

## Claude Code 2.1.296 — confirmado

Teste com `claude -p ... --settings <arquivo> --model haiku`, com um `atlasoq` falso no `PATH`:

```json
{
  "hooks": { "SessionStart": [ { "matcher": "startup|resume|clear|compact",
    "hooks": [ { "type": "command", "command": "atlasoq learn --hook" } ] } ] },
  "permissions": { "allow": ["Bash(atlasoq:*)", "Bash(atlasoq *)"] }
}
```

- A saída de `atlasoq learn --hook` entrou no contexto do agente: ele respondeu o código de
  verificação presente só na saída do hook.
- Com a regra `permissions.allow`, o agente executou `atlasoq list` em modo não interativo sem
  pedir aprovação.
- `--settings` aceita caminho de arquivo; nenhum arquivo fora do diretório temporário foi tocado.

**Decisão:** perfil `claude-code` usa o modo `hook`, com o arquivo gerado em
`.atlasoq/agents/<nome>/claude-settings.json` contendo o hook e as duas formas da regra de
permissão.

## Codex, Gemini CLI, OpenCode, Aider — não verificados localmente

Não estão instalados nesta máquina. Os perfis prontos seguem a documentação (research.md R4):

| CLI | Modo | Comando |
|---|---|---|
| Codex | `prompt` | `codex "<conteúdo>"` |
| Gemini CLI | `prompt` | `gemini --prompt-interactive "<conteúdo>"` |
| OpenCode | `prompt` | `opencode --prompt "<conteúdo>"` |
| Aider | `type` | entrega após a primeira ociosidade |

**Pendente:** validar com as CLIs instaladas antes do release (T137). Se uma flag não funcionar, o
perfil muda para `type`, que não depende da CLI.
