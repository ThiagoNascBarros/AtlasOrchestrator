# Contrato: formato do diretório `.atlasoq/`

Formato público: o usuário pode ler, editar e versionar estes arquivos. Mudanças incompatíveis
exigem novo `schema` e migração automática.

## Estrutura

```
<raiz>/.atlasoq/
├── workspace.json          versionável
├── notes/<nome>.md         versionável
├── profiles/<nome>.json    versionável (perfis do workspace)
├── learn.md                versionável, opcional (≤ 4 KiB entregues)
├── memory/<agente>.md      local por padrão; versionável com "Versionar memória"
├── agents/<agente>/        local, recriado a cada início (ex.: claude-settings.json)
├── logs/                   local (auditoria JSONL)
├── backups/                local (10 últimas versões do workspace.json)
├── .lock                   local (flock da instância que abriu o workspace)
└── .gitignore              gerado pelo app
```

`.gitignore` gerado:

```
# gerado pelo AtlasOQ — as linhas marcadas são gerenciadas pelo app
agents/
logs/
backups/
.lock
memory/   # atlasoq:memory
```

"Versionar memória" remove somente a linha marcada `# atlasoq:memory`; "Parar de versionar" a
recoloca.

## `workspace.json` (schema 1)

```json
{
  "schema": 1,
  "id": "ws_01J...",
  "name": "refactor-auth",
  "viewport": { "x": -320.5, "y": 140.0, "zoom": 0.85 },
  "settings": {
    "memory_versioned": false,
    "limits_default": { "max_exchanges": 20, "rate_per_min": 6, "burst": 3, "ping_pong_repeats": 4 }
  },
  "nodes": [
    {
      "id": "a_01J...", "kind": "agent", "name": "planner", "role": "planeja e distribui tarefas",
      "profile": "claude-code", "rect": { "x": 0, "y": 0, "w": 720, "h": 460 },
      "terminal": { "command": "claude", "args": [], "cwd": ".", "env": {},
                    "env_secret_keys": [], "cols": 100, "rows": 30 },
      "learn": { "mode": "hook" }
    },
    { "id": "n_01J...", "kind": "note", "name": "plano",
      "rect": { "x": 760, "y": 0, "w": 380, "h": 300 }, "file": "notes/plano.md" }
  ],
  "connections": [
    { "id": "c_01J...", "from": "a_01J...", "to": "a_01K...", "direction": "forward",
      "state": "active", "limits": { "max_exchanges": 20, "rate_per_min": 6, "burst": 3, "ping_pong_repeats": 4 } }
  ]
}
```

Validação na leitura: nomes únicos e válidos, ids referenciados existem, `from ≠ to`, nada de
nota↔nota, `cwd` relativo. Arquivo inválido → oferecer o backup válido mais recente.

## `profiles/<nome>.json`

```json
{ "name": "claude-code", "command": "claude", "args": [], "env": {},
  "learn": { "mode": "hook", "ready_timeout_s": 60 } }
```

Perfis prontos (globais, em `~/.config/atlasoq/profiles/`): `claude-code` (`hook`), `codex`
(`prompt`), `gemini` (`prompt`), `opencode` (`prompt`), `aider` (`type`), `shell` (`off`).

## `memory/<agente>.md`

```markdown
# Memória de coder

- 2026-10-11 14:02 feito: validação de login com testes
- 2026-10-11 14:30 decisão: tokens de sessão expiram em 15 min
- 2026-10-11 17:55 pendente: revisar cache de permissões
```

## `notes/<nome>.md`

Markdown livre. Acréscimos de agentes: `- AAAA-MM-DD HH:MM [<agente>] <texto>` no fim do arquivo.

## Escrita

Toda escrita: arquivo temporário no mesmo diretório → `fsync` → `rename` → `fsync` do diretório.
