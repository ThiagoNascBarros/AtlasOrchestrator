# Contrato: protocolo CLI ↔ app (socket local)

Interno entre o binário `atlasoq` e o app, mas versionado e documentado porque é a fronteira de
segurança das conexões (princípio IV).

## Transporte

- Socket Unix `SOCK_STREAM` em `$ATLASOQ_SOCKET` (`$XDG_RUNTIME_DIR/atlasoq/<instância>.sock`).
- Diretório `0700`, socket `0600`. O app recusa conexões cujo `SO_PEERCRED.uid` difere do seu.
- Mensagens: um objeto JSON UTF-8 por linha (`\n`), máximo de 64 KiB por linha.
- Uma requisição por conexão. O app responde com uma linha e fecha, exceto `ask`, que mantém a
  conexão até o resultado final.

## Requisição

```json
{ "v": 1, "op": "send", "node": "a_01J...", "token": "<base64url 32 bytes>", "args": { ... } }
```

| `op` | `args` |
|---|---|
| `learn` | `{ "hook": bool }` |
| `list` | `{}` |
| `send` | `{ "to": "revisor", "text": "..." }` |
| `read` | `{ "from": "coder", "lines": 60 }` |
| `ask` | `{ "to": "coder", "text": "...", "timeout_s": 600 }` |
| `reply` | `{ "to": "planner" \| null, "text": "..." }` |
| `note_read` | `{ "note": "plano" \| "memory" }` |
| `note_append` | `{ "note": "plano" \| "memory", "text": "..." }` |

`help` e a sugestão de comandos são resolvidos na CLI, sem socket.

## Resposta

```json
{ "v": 1, "ok": true, "data": { ... } }
{ "v": 1, "ok": false, "code": 3, "error": "no_connection", "message": "...", "hint": "..." }
```

`code` é o código de saída que a CLI usa. `error` é um identificador estável:
`bad_request`, `unknown_target`, `no_connection`, `wrong_direction`, `connection_paused`,
`target_exited`, `too_large`, `secret_detected`, `missing_prefix`, `no_pending_ask`,
`ambiguous_reply`, `ask_expired`, `timeout`, `unauthorized`, `app_closing`, `unsupported_version`.

## Autenticação e autorização

1. `uid` do par == `uid` do app, senão fecha sem resposta.
2. `v` suportado, senão `unsupported_version`.
3. `node` existe e `token` confere (comparação em tempo constante), senão `unauthorized`.
4. Autorização pela tabela de permissões do [data-model.md](../data-model.md) e pelos limites da
   conexão. Toda decisão (permitida ou recusada) gera um `AuditEvent`.

## Compatibilidade

Campos novos podem ser adicionados sem mudar `v`; remover ou mudar o significado de um campo exige
`v: 2`, e o app MUST aceitar `v: 1` enquanto a CLI publicada o usar.
