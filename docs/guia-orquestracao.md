# Guia de orquestração

Receitas para montar times de agentes no canvas do AtlasOQ. Os agentes aprendem sozinhos a usar o
`atlasoq` ao nascer; você só precisa desenhar as ligações e dar um papel claro a cada um.

## Como escrever papéis

O papel aparece no tutorial de cada agente e na saída de `atlasoq list`. Escreva o que o agente
faz e o que não faz, em uma frase:

- Bom: `implementa as tarefas do plano; não faz revisão`
- Ruim: `ajudante`

## Coordenador → executores

Um agente `planner` liga-se a `coder-1` e `coder-2` no sentido de envio (`planner → coder-N`).

1. Crie os três agentes e ligue `planner` a cada executor pelo botão `⟶`.
2. Peça ao `planner` a tarefa. Ele usa `atlasoq send coder-1 "..."` e recebe respostas dos executores
   com `atlasoq reply` quando pergunta.

Os executores só respondem ao planner; não podem mandar mensagens por conta própria a ele.

## Executor ↔ revisor

Um `coder` e um `revisor` com ligação de duas vias (↔).

- O `coder` envia o diff para revisão com `atlasoq send revisor "..."`.
- O `revisor` pergunta detalhes com `atlasoq ask coder "..."` e o `coder` responde com `reply`.

Use ligação ↔ só entre pares que realmente precisam conversar nos dois sentidos.

## Regras do time

Crie `.atlasoq/learn.md` na pasta do workspace com as regras que todos devem seguir (até 4 KiB).
Ele é entregue a cada agente junto com o tutorial, logo após a identidade e as ligações.

## Limites e pausas

Cada conexão tem limites: 20 trocas sem intervenção humana, 6 mensagens por minuto e pausa quando
há vai-e-vem repetitivo. Quando uma conexão pausa sozinha, o nó e o histórico avisam. Abra o
histórico, confirme que está tudo certo e reative.
