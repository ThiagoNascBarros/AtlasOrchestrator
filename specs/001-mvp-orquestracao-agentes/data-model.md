# Data Model: MVP do AtlasOQ

**Feature**: [spec.md](spec.md) · **Formato em disco**: [contracts/workspace-format.md](contracts/workspace-format.md)

Entidades persistidas ficam em `<raiz>/.atlasoq/` (JSON e Markdown). Entidades de execução vivem só
em memória no processo do app.

## Persistidas

### Workspace

| Campo | Tipo | Regras |
|---|---|---|
| `schema` | inteiro | Versão do formato; começa em 1; migrações puras por versão |
| `id` | string (ULID) | Gerado na criação |
| `name` | string | 1–80 caracteres |
| `viewport` | `{x: f64, y: f64, zoom: f64}` | `zoom` entre 0.1 e 4.0 |
| `settings.memory_versioned` | bool | Padrão `false`; espelha a linha `memory/` do `.atlasoq/.gitignore` |
| `settings.limits_default` | `ConnectionLimits` | Padrões para novas conexões |
| `nodes` | lista de `Agent` ou `Note` | Nomes únicos entre todos os nós |
| `connections` | lista de `Connection` | Ver regras abaixo |

### Agent (nó)

| Campo | Tipo | Regras |
|---|---|---|
| `id` | string | `a_<ulid>` |
| `kind` | `"agent"` | |
| `name` | string | `^[a-z][a-z0-9-]{0,31}$`; único no workspace; `memory` é reservado |
| `role` | string | Opcional, até 200 caracteres |
| `profile` | string | Nome do perfil de origem |
| `rect` | `{x, y, w, h}` | `w ≥ 240`, `h ≥ 160` |
| `terminal.command` | string | Não vazio |
| `terminal.args` | lista de string | |
| `terminal.cwd` | string | Relativo à raiz; MUST resolver dentro do sistema de arquivos acessível |
| `terminal.env` | mapa string→string | Sem valores de chaves secretas |
| `terminal.env_secret_keys` | lista de string | Valores vêm do keyring ou do ambiente do app |
| `terminal.cols`, `terminal.rows` | inteiro | 20–500 / 5–200 |
| `learn.mode` | `hook` \| `prompt` \| `type` \| `off` | Herdado do perfil, pode ser sobrescrito |

### Note (nó)

| Campo | Tipo | Regras |
|---|---|---|
| `id` | string | `n_<ulid>` |
| `kind` | `"note"` | |
| `name` | string | Mesmas regras de nome dos agentes; `memory` é reservado |
| `rect` | `{x, y, w, h}` | |
| `file` | string | `notes/<name>.md` |

### Connection

| Campo | Tipo | Regras |
|---|---|---|
| `id` | string | `c_<ulid>` |
| `from`, `to` | id de nó | `from ≠ to`; agente↔agente ou agente↔nota; nota↔nota não permitido |
| `direction` | `forward` \| `backward` \| `both` | `forward` = `from → to` |
| `state` | `active` \| `paused` \| `auto_paused` | `auto_paused` guarda `pause_reason` |
| `pause_reason` | string? | `max_exchanges`, `rate_limit`, `ping_pong` |
| `limits` | `ConnectionLimits` | |

`ConnectionLimits`: `max_exchanges` (padrão 20), `rate_per_min` (6), `burst` (3),
`ping_pong_repeats` (4).

Permissões derivadas de uma conexão ativa:

| Ligação | Quem envia / pergunta / lê a tela | Nota |
|---|---|---|
| agente A → agente B | A sobre B; B só responde a `ask` pendente de A | — |
| agente A ↔ agente B | Os dois | — |
| nota N → agente A | — | A lê N |
| agente A → nota N | — | A lê e acrescenta em N |

### AgentProfile

| Campo | Tipo | Regras |
|---|---|---|
| `name` | string | Único por nível; o do workspace sobrepõe o global |
| `command`, `args`, `env` | | |
| `learn.mode` | enum | Ver R4 da pesquisa |
| `learn.ready_timeout_s` | inteiro | Padrão 60 |
| `scope` | `global` \| `workspace` | Derivado do local do arquivo |

### MemoryEntry (em `memory/<agente>.md`)

| Campo | Tipo | Regras |
|---|---|---|
| `at` | data e hora local (minuto) | Preenchido pelo app |
| `kind` | `feito` \| `decisão` \| `pendente` | Obrigatório; sem prefixo → recusado com dica |
| `text` | string | 1–300 caracteres, sanitizado, sem segredos detectáveis |

### AuditEvent (em `logs/links-AAAA-MM-DD.jsonl`, local)

| Campo | Tipo |
|---|---|
| `ts` | RFC 3339 |
| `event` | `message`, `connection_created`, `connection_paused`, `connection_resumed`, `connection_deleted`, `learn`, `trust` |
| `msg_id`, `kind` | para mensagens: `send`, `ask`, `reply`, `note`, `memory` |
| `from`, `to`, `conn_id` | nomes e id |
| `exchanges` | contador no momento |
| `bytes` | tamanho |
| `text` ou `text_sha256` | conforme configuração |
| `result` | `delivered`, `refused:<motivo>`, `timeout`, `ok`, `failed:<motivo>` |

### TrustRecord (em `~/.local/state/atlasoq/trust.json`)

`{ root: caminho canônico, sha256: hash do conjunto executável, approved_at }`.

## Em execução (memória do app)

### AgentSession

Estados e transições:

```
created ──spawn──▶ starting ──primeira saída──▶ working ◀──saída──┐
                       │                          │ 3 s sem saída │
                       │                          ▼               │
                       │                        idle ─────────────┘
                       └──falha no spawn──▶ exited(code)
working|idle ──processo termina──▶ exited(code) ──reiniciar──▶ starting
```

### LearnState (por sessão)

```
pending ──modo hook: `atlasoq learn --hook` chamado──▶ done
pending ──modo prompt: processo iniciado com o texto──▶ done
pending ──modo type: primeiro idle → entrega──▶ done
pending ──tempo-limite / processo saiu / usuário digitando──▶ failed(motivo)
off (modo off)
```

### PendingAsk

| Campo | Regras |
|---|---|
| `id` | sequencial por instância |
| `asker`, `target` | nomes |
| `created_at`, `deadline` | `deadline = created_at + timeout` (padrão 600 s) |
| estado | `waiting` → `answered` \| `timed_out` \| `cancelled(motivo)` |

Regras: `reply` sem nome responde à pergunta `waiting` mais antiga endereçada ao agente, se houver
só um perguntador; com mais de um, exige o nome. Pausar/apagar a conexão ou encerrar o destino
cancela as perguntas pendentes.

### ExchangeCounter (por conexão)

Incrementa quando um agente envia (send/ask/reply) a quem lhe enviou algo nos últimos 120 s.
Zera com qualquer tecla do usuário num dos dois terminais. Ao passar de `max_exchanges` →
`auto_paused`.
