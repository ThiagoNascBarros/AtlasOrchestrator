# Contrato: CLI `atlasoq`

Interface pública usada pelos agentes. Mudar nome ou semântica de um comando publicado exige emenda
MAJOR da constituição (princípio VIII). Testes de contrato cobrem cada linha deste documento.

## Regras gerais

- Formato: `atlasoq <comando> [alvo] [texto]`. Exatamente 8 comandos.
- `<agente>` e `<nota>` são os **nomes visíveis no canvas**. `memory` é reservado e sempre significa
  "a minha memória".
- `<texto>` pode ser um ou mais argumentos (unidos por espaço) ou vir da entrada padrão quando
  omitido e a entrada não é um terminal.
- Saída padrão: texto em português, curto. `--json` em qualquer comando: um objeto JSON por
  execução, em stdout.
- Erros vão para stderr no formato:
  ```
  erro: <o que aconteceu>
        <próximo passo>
  ```
- Ambiente obrigatório (injetado pelo app): `ATLASOQ_SOCKET`, `ATLASOQ_NODE_ID`,
  `ATLASOQ_NODE_TOKEN`. Ausente → código 6.

## Códigos de saída

| Código | Significado |
|---|---|
| 0 | Sucesso |
| 2 | Uso incorreto (comando, argumento ou flag) |
| 3 | Sem conexão ou sem permissão para a operação |
| 4 | Destino indisponível (não existe, encerrado, app fechou) |
| 5 | Tempo esgotado (`ask`) |
| 6 | Executado fora de um agente do AtlasOQ |
| 7 | Conteúdo recusado (tamanho, segredo, anotação sem prefixo) |

## Comandos

### `atlasoq learn`

Imprime o tutorial personalizado (≤ 60 linhas) seguido de `.atlasoq/learn.md` (se houver) e das
últimas anotações da memória. Flag interna `--hook` (usada pelo hook do Claude Code): mesma
saída, sem cores, e registra `learn: done`. Fora do app, imprime o tutorial genérico com código 0.

### `atlasoq list`

```
você: coder — implementa as tarefas (trabalhando)
agentes:
  planner   ← pode te enviar e perguntar      ocioso
  revisor   → você pode enviar e perguntar    trabalhando
notas:
  plano     ↔ ler e acrescentar
perguntas pendentes para você: 1 (de planner) — responda com: atlasoq reply "..."
```

### `atlasoq send <agente> <texto>`

Entrega e retorna. Saída: `enviado para revisor (msg 42)`.
Erros: 3 sem ligação no sentido; 4 destino inexistente/encerrado; 7 texto > 16 KiB ou vazio;
3 conexão pausada (com motivo).

### `atlasoq read <agente> [--lines N]`

Texto recente da tela e do scrollback do agente, sem ANSI. `N` padrão 60, máximo 2000. Requer
ligação no sentido eu → agente.

### `atlasoq ask <agente> <texto> [--timeout S]`

Entrega a pergunta e bloqueia até a resposta.
- Resposta via `reply`: imprime exatamente o texto respondido, código 0.
- Tempo esgotado (padrão 600 s): imprime `sem resposta de <agente> em <S>s; tela recente:` e o
  texto da tela, código 5.
- Conexão pausada/apagada ou destino encerrado durante a espera: código 3 ou 4 com o motivo.

Formato entregue ao destino:
```
[atlasoq ask de:planner id:17] quantos testes falharam? — responda com: atlasoq reply "..."
```

### `atlasoq reply [agente] <texto>`

Responde à pergunta pendente. Primeiro argumento é tratado como `agente` se coincidir com o nome
de um perguntador pendente; caso contrário, tudo é texto.
- Sem pergunta pendente → código 3: `não há pergunta para responder. Para avisar alguém, use
  atlasoq send <agente> "..."`.
- Mais de um perguntador → código 2 listando as perguntas pendentes.
- Pergunta expirada → código 3 com sugestão de `send`.

### `atlasoq note <nota> [texto]`

- Sem texto: imprime a nota. Requer qualquer ligação ativa entre a nota e o agente.
- Com texto: acrescenta ao fim como `- AAAA-MM-DD HH:MM [<agente>] <texto>`. Requer ligação
  agente → nota ou ↔.
- `atlasoq note memory` lê a própria memória; `atlasoq note memory "feito: ..."` acrescenta.
  Prefixo obrigatório `feito:`, `decisão:` ou `pendente:`; até 300 caracteres; segredos recusados
  (código 7).

### `atlasoq help [comando]`

Lista os 8 comandos com uma linha cada, ou o uso detalhado com exemplo.

## Erros de digitação

Comando desconhecido com distância de edição ≤ 2 de um comando válido:
```
erro: comando "sned" não existe. Você quis dizer "send"?
      uso: atlasoq send <agente> <texto>
```

## Esqueleto do tutorial (`docs/agentes/tutorial.md`)

1. Quem você é e o que é o AtlasOQ (2 linhas, factual).
2. `atlasoq list` primeiro, sempre que for falar com alguém.
3. Um exemplo por comando (`send`, `ask`, `reply`, `read`, `note`).
4. Memória: o que anotar, prefixos, quando; não abrir sessão perguntando se deve continuar.
5. Regras: responda perguntas com `reply`; mensagens curtas; contexto longo em notas; evite
   vai-e-vem; leia o erro, ele diz o próximo passo.
6. Erros comuns e o que fazer.
