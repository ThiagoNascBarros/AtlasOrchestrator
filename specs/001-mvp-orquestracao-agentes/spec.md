# Feature Specification: MVP do AtlasOQ — canvas para orquestrar agentes de IA

**Feature Branch**: `ATQ-001`

**Spec**: `specs/001-mvp-orquestracao-agentes`

**Created**: 2026-10-11

**Status**: Draft

**Input**: User description: "ATQ-001 Como usuário desejo um app de orquestramento de agentes de IA.
Um canvas infinito para orquestrar agentes de IA no Linux: cada agente roda num terminal que é um
nó posicionável, ao lado de notas em Markdown; dois agentes podem ser ligados por uma linha para
conversarem entre si. Os comandos `atlasoq` devem ser simples e fáceis de lembrar, e deve existir
um tutorial para os agentes aprenderem a trabalhar com o `atlasoq` pela linha de comando.
Instalação por pacote .deb."

**Referências**: [SPEC.md](../../SPEC.md) v0.3.0 · [Constituição](../../.specify/memory/constitution.md) v1.2.0

## Glossário

- **Canvas**: área de trabalho infinita, com pan e zoom, onde ficam os nós.
- **Agente**: nó do canvas que roda uma CLI de agente de IA (Claude Code, Codex, Gemini CLI,
  OpenCode, Aider) ou um shell, num terminal real. Tem um **nome** único no workspace e um
  **papel** opcional (ex.: "planeja as tarefas", "revisa código").
- **Nota**: nó com texto em Markdown.
- **Conexão**: linha desenhada pelo usuário entre dois nós, com direção (→, ← ou ↔) e estado
  (ativa ou pausada). Entre dois agentes, permite mensagens; entre agente e nota, permite que o
  agente leia e acrescente texto na nota.
- **Workspace**: o conjunto salvo de canvas, nós, notas, conexões e perfis. Fica no diretório
  `.atlasoq/` dentro de uma pasta raiz escolhida pelo usuário (normalmente a raiz do projeto).
- **Perfil de agente**: receita para criar um agente (comando, argumentos, variáveis e o hook de
  inicialização).
- **Hook de inicialização**: passo automático executado logo depois que um agente nasce, cujo
  papel no MVP é fazer o agente receber o tutorial do `atlasoq learn`.
- **`atlasoq`**: CLI que os agentes usam, de dentro do próprio terminal, para conversar com outros
  agentes e usar notas.
- **Tutorial para agentes**: texto curto, impresso por `atlasoq learn`, que ensina um agente a
  usar a CLI.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Montar o time de agentes no canvas (Priority: P1)

Como usuário, abro o AtlasOQ, crio agentes a partir de perfis prontos (por exemplo "Claude Code" e
"Codex"), dou um nome e um papel a cada um, e os posiciono no canvas. Cada agente roda num terminal
real onde posso digitar normalmente. Navego pelo canvas com pan e zoom e vejo de relance quais
agentes estão trabalhando e quais estão parados.

**Why this priority**: sem agentes rodando num canvas navegável não existe produto. Sozinha, esta
história já entrega valor: ver e operar vários agentes lado a lado num espaço organizado.

**Independent Test**: instalar o `.deb`, abrir o app, criar três agentes com perfis diferentes,
interagir com cada um, mover, redimensionar e navegar com pan e zoom.

**Acceptance Scenarios**:

1. **Given** o app aberto com um workspace vazio, **When** o usuário cria um agente com o perfil
   "Shell", nome `coder` e papel "implementa", **Then** aparece um nó no canvas com o nome, o papel
   e um terminal interativo rodando o shell do usuário no diretório escolhido.
2. **Given** um agente rodando uma TUI (ex.: `htop`, `vim` ou a CLI de um agente), **When** o
   usuário redimensiona o nó, **Then** a TUI se ajusta ao novo tamanho sem quebrar o layout.
3. **Given** vários agentes no canvas, **When** o usuário dá zoom e pan, **Then** o movimento é
   fluido e, ao parar o zoom, o texto dos terminais fica nítido e a seleção com o mouse acerta os
   caracteres.
4. **Given** um agente que está produzindo saída, **When** o usuário olha o canvas, **Then** o nó
   mostra o indicador "trabalhando"; após alguns segundos sem saída, mostra "ocioso"; se o processo
   terminar, mostra "encerrado" com o código de saída.
5. **Given** o foco num terminal, **When** o usuário aperta `Esc` duas vezes ou clica no fundo do
   canvas, **Then** o foco volta ao canvas e os atalhos do canvas voltam a funcionar.
6. **Given** um agente já chamado `coder`, **When** o usuário tenta criar outro com o mesmo nome,
   **Then** o app recusa e sugere outro nome.

---

### User Story 2 - Ligar agentes para que conversem entre si (Priority: P1)

Como usuário, desenho uma linha de um agente `planner` até um agente `coder`, escolho a direção, e
a partir daí o `planner` consegue mandar tarefas ao `coder` e ler o que o `coder` respondeu, usando
comandos `atlasoq` simples. Vejo no canvas cada mensagem passando pela linha, posso pausar ou
apagar a ligação a qualquer momento, e consulto o histórico do que foi dito.

**Why this priority**: é o que transforma um painel de terminais num orquestrador. É também o
ponto de maior risco de segurança, por isso entra junto da história 1.

**Independent Test**: com dois agentes rodando shells, ligar `planner → coder`, executar
`atlasoq send coder "..."` no `planner`, conferir a entrega no `coder`, ler a resposta com
`atlasoq read coder`, pausar a ligação e verificar a recusa.

**Acceptance Scenarios**:

1. **Given** os agentes `planner` e `coder` sem ligação, **When** o `planner` executa
   `atlasoq send coder "rode os testes"`, **Then** nada é entregue e o comando responde que não
   existe ligação de `planner` para `coder` e que o usuário precisa criá-la no canvas.
2. **Given** a ligação `planner → coder` ativa, **When** o `planner` executa
   `atlasoq send coder "rode os testes"`, **Then** o texto aparece no terminal do `coder` com a
   indicação de que veio do `planner`, é submetido como uma mensagem única, e a linha no canvas
   mostra a passagem da mensagem.
3. **Given** a ligação `planner → coder`, **When** o `coder` tenta `atlasoq send planner "..."`,
   **Then** a mensagem é recusada, porque a ligação só permite o sentido `planner → coder`.
4. **Given** a ligação `planner → coder` e o `coder` respondendo no próprio terminal, **When** o
   `planner` executa `atlasoq read coder`, **Then** recebe o texto recente da tela do `coder`, sem
   códigos de cor.
5. **Given** a ligação ativa, **When** o `planner` executa `atlasoq ask coder "qual o resultado?"`,
   **Then** a mensagem é entregue, o comando espera o `coder` ficar ocioso e devolve o que o `coder`
   produziu depois da mensagem.
6. **Given** a ligação ativa, **When** o usuário pausa ou apaga a ligação, **Then** a próxima
   mensagem é recusada imediatamente, com a explicação do motivo.
7. **Given** uma mensagem contendo sequências de controle (ESC, Ctrl-C, códigos de cor), **When** ela
   é enviada, **Then** o destino recebe apenas o texto imprimível; nenhuma tecla de controle chega
   ao terminal de destino.
8. **Given** dois agentes ligados nos dois sentidos trocando mensagens automaticamente, **When** o
   número de trocas seguidas sem intervenção humana passa do limite, **Then** a ligação é pausada,
   o usuário vê um aviso no canvas e só ele pode reativá-la.
9. **Given** mensagens trocadas numa ligação, **When** o usuário clica na linha, **Then** vê o
   histórico com horário, origem, destino, texto e resultado (entregue ou recusada) de cada uma.
10. **Given** um processo qualquer da máquina que não é um agente do canvas, **When** ele tenta usar
    a CLI `atlasoq` para enviar mensagens, **Then** a requisição é recusada.

---

### User Story 3 - Agentes nascem sabendo usar o atlasoq (Priority: P1)

Como usuário, não quero ensinar a CLI a cada agente nem apertar nenhum botão para isso. Quando crio
um agente (ou ele é reiniciado, ou reabro o workspace), um hook de inicialização roda logo depois
que ele sobe e faz o agente receber o tutorial do `atlasoq learn`. A partir daí ele sabe quem é,
com quem pode falar e como mandar, ler, perguntar e usar notas.

**Why this priority**: os usuários da CLI são agentes de IA. Qualquer passo manual para ensiná-los
seria esquecido e anularia o ganho da história 2.

**Independent Test**: criar um agente real (ex.: Claude Code) ligado a outro e, sem nenhuma outra
ação, pedir em linguagem natural "pergunte ao coder quantos testes existem"; o agente deve usar
`atlasoq ask coder ...` corretamente.

**Acceptance Scenarios**:

1. **Given** um perfil de agente com hook de inicialização, **When** o usuário cria o agente,
   **Then** logo após a inicialização o agente recebe o tutorial personalizado (nome, papel,
   ligações) sem nenhuma ação do usuário, e o nó indica que o aprendizado foi concluído.
2. **Given** um agente que já aprendeu, **When** ele é reiniciado ou o workspace é reaberto,
   **Then** o hook roda de novo e o agente recebe o tutorial outra vez.
3. **Given** um perfil cuja CLI oferece hooks próprios de início de sessão, **When** o agente nasce,
   **Then** o AtlasOQ usa esse mecanismo, com arquivos gerados dentro de `.atlasoq/`, sem alterar
   arquivos do projeto ou da configuração global da CLI.
4. **Given** um perfil cuja CLI não oferece hooks, **When** o agente nasce, **Then** o AtlasOQ usa a
   alternativa definida no perfil (prompt inicial na linha de comando ou mensagem enviada assim que
   o agente fica pronto) para que o agente rode `atlasoq learn`.
5. **Given** o hook falhou (ex.: o agente encerrou antes de ficar pronto), **When** o usuário olha o
   nó, **Then** vê o aviso "aprendizado falhou" com o motivo, e o registro fica no log.
6. **Given** o arquivo `.atlasoq/learn.md` com regras do time, **When** um agente aprende, **Then**
   recebe o tutorial padrão seguido dessas regras.
7. **Given** um perfil "Shell" (terminal para uso humano), **When** é criado, **Then** nenhum hook
   de aprendizado é executado.
8. **Given** um agente que digita um comando errado (ex.: `atlasoq sned coder oi`), **When** o
   comando falha, **Then** a resposta sugere o comando provável (`send`) e mostra o uso correto.
9. **Given** um agente que executa `atlasoq help` ou `atlasoq help send`, **When** o comando roda,
   **Then** mostra a lista de comandos ou o uso do comando pedido, com um exemplo.
10. **Given** a documentação do projeto, **When** um colaborador abre `docs/`, **Then** encontra o
    mesmo tutorial (para agentes) e um guia para usuários sobre como montar orquestrações comuns
    (ex.: um coordenador ligado a vários executores; um executor ligado a um revisor).

---

### User Story 4 - Notas como contexto compartilhado (Priority: P2)

Como usuário, crio notas em Markdown no canvas para registrar o plano, decisões e contexto. Ligo
uma nota a um ou mais agentes para que eles possam lê-la e acrescentar texto nela pela CLI, o que
dá ao time de agentes uma memória comum visível para mim.

**Why this priority**: notas organizam o trabalho do usuário (valor isolado) e, ligadas a agentes,
são um canal de contexto mais seguro e legível que mensagens soltas. O canvas funciona sem elas.

**Independent Test**: criar uma nota "plano", editá-la, ligá-la ao `planner` e ao `coder`, e
executar `atlasoq note plano` e `atlasoq note plano "feito: testes"` em cada agente.

**Acceptance Scenarios**:

1. **Given** o canvas, **When** o usuário cria uma nota e escreve Markdown, **Then** a nota alterna
   entre modo de edição e modo de leitura com o Markdown renderizado (títulos, listas, tarefas,
   tabelas, blocos de código).
2. **Given** a nota `plano` ligada ao agente `coder`, **When** o `coder` executa
   `atlasoq note plano`, **Then** recebe o conteúdo da nota em texto.
3. **Given** a nota `plano` ligada ao `coder` no sentido `coder → plano`, **When** o `coder`
   executa `atlasoq note plano "feito: testes"`, **Then** o texto é acrescentado ao fim da nota,
   com o nome do agente e o horário, e aparece na nota aberta no canvas.
4. **Given** a ligação `plano → coder` (só leitura), **When** o `coder` tenta acrescentar texto,
   **Then** a operação é recusada com a explicação.
5. **Given** uma nota sem ligação com o agente, **When** ele tenta lê-la, **Then** a operação é
   recusada.
6. **Given** uma nota com HTML ou scripts embutidos, **When** é exibida, **Then** o conteúdo
   perigoso é neutralizado e links abrem no navegador do sistema.

---

### User Story 5 - Retomar o workspace depois de fechar o app (Priority: P2)

Como usuário, fecho o app (ou ele trava) e, ao reabrir, encontro o canvas exatamente como estava:
posições, zoom, notas, ligações e agentes. Os agentes são reiniciados com o mesmo comando, pasta e
tamanho, ou, se eu preferir, ficam parados até eu mandar iniciar.

**Why this priority**: perder a organização do time de agentes destrói a confiança no app. Fica
abaixo de P1 porque no MVP o processo dos agentes não sobrevive ao fechamento do app.

**Independent Test**: montar um workspace com agentes, notas e ligações; encerrar o app à força;
reabrir e comparar o canvas e o conteúdo de `.atlasoq/`.

**Acceptance Scenarios**:

1. **Given** um workspace com 3 agentes, 2 notas e 2 ligações, **When** o usuário fecha e reabre o
   app, **Then** tudo volta nas mesmas posições, com o mesmo zoom, e as ligações com a mesma
   direção e estado.
2. **Given** um workspace salvo, **When** o app é encerrado à força durante uma alteração, **Then**
   o workspace reabre íntegro, perdendo no máximo a última alteração de layout feita no último
   segundo.
3. **Given** a opção "perguntar antes de iniciar agentes", **When** o workspace é reaberto,
   **Then** os agentes aparecem parados com um botão para iniciar todos ou cada um.
4. **Given** um agente configurado com uma variável de ambiente marcada como secreta, **When** o
   workspace é salvo, **Then** o valor dessa variável não aparece em nenhum arquivo do workspace.
5. **Given** uma nota editada fora do app com o app fechado, **When** o workspace é reaberto,
   **Then** a nota mostra o conteúdo novo.
6. **Given** uma pasta de projeto, **When** o usuário cria um workspace nela, **Then** o app cria
   `<pasta>/.atlasoq/` com os arquivos versionáveis (workspace, notas, perfis) e um `.gitignore`
   próprio que exclui logs, backups e arquivos gerados por agente.
7. **Given** um `.atlasoq/` vindo de um repositório clonado, **When** o usuário abre o workspace pela
   primeira vez, **Then** o app mostra os comandos, perfis, hooks e instruções que seriam
   executados e só inicia os agentes depois da aprovação; se esses itens mudarem depois, pede
   aprovação de novo.
8. **Given** o arquivo principal do workspace corrompido, **When** o app tenta abri-lo, **Then**
   oferece restaurar a cópia válida mais recente.

---

### User Story 6 - Instalar pelo pacote .deb (Priority: P3)

Como usuário de Ubuntu ou Debian, instalo o AtlasOQ com um pacote `.deb`, encontro o app no menu
do sistema com ícone próprio, e a CLI `atlasoq` funciona dentro dos terminais do app sem
configuração.

**Why this priority**: necessário para o release, mas não muda o valor das outras histórias, que
podem ser testadas a partir do build de desenvolvimento.

**Independent Test**: instalar o `.deb` numa máquina limpa, abrir pelo menu, criar um agente e
rodar `atlasoq me`; depois desinstalar.

**Acceptance Scenarios**:

1. **Given** um Ubuntu LTS atual limpo, **When** o usuário instala o `.deb`, **Then** as dependências
   são resolvidas pelo gerenciador de pacotes e o app aparece no menu com nome e ícone corretos.
2. **Given** o app instalado, **When** é aberto numa sessão Wayland e numa sessão X11, **Then**
   funciona nas duas.
3. **Given** um terminal de agente, **When** o agente executa `atlasoq me`, **Then** o comando é
   encontrado sem o usuário ajustar o `PATH`.
4. **Given** o app instalado, **When** o usuário remove o pacote, **Then** os binários saem do
   sistema e os workspaces do usuário permanecem intactos.

---

### Edge Cases

- Um agente envia mensagem para um nome que não existe → erro listando os nomes disponíveis para
  ele (`atlasoq list`).
- O agente de destino encerrou o processo → a mensagem é recusada com "o agente `coder` está
  encerrado; peça ao usuário para reiniciá-lo".
- O usuário renomeia um agente durante uma conversa → mensagens novas precisam usar o nome novo; o
  erro para o nome antigo informa o nome atual.
- Mensagem acima do tamanho máximo → recusada com o limite informado, sem entrega parcial.
- Duas mensagens enviadas ao mesmo agente quase ao mesmo tempo → entregues em sequência, como duas
  mensagens separadas, nunca coladas.
- O destino está com uma TUI em tela cheia ou esperando confirmação → a mensagem é entregue como
  texto colado; o comportamento depende da CLI de destino (documentado no tutorial).
- `atlasoq ask` sem resposta → retorna após o tempo-limite com o que houver e indica que o tempo
  acabou; não fica preso para sempre.
- O destino nunca fica ocioso (saída contínua) → `ask` respeita o tempo-limite.
- Um agente gera saída muito volumosa → o canvas continua responsivo e os outros agentes não são
  afetados.
- O app fecha durante um `atlasoq ask` → a CLI encerra com erro claro em vez de travar.
- Ligação de um agente para ele mesmo → não permitida.
- Zoom muito afastado com muitos agentes → os nós viram cartões resumidos e voltam ao terminal
  completo ao aproximar, sem perder conteúdo.
- O `atlasoq` é executado fora de um terminal do app → responde que só funciona dentro de um agente
  do AtlasOQ.
- Dois apps AtlasOQ abertos ao mesmo tempo → cada agente fala só com a instância que o criou.
- Mesmo workspace aberto duas vezes → a segunda abertura é bloqueada com aviso.
- A CLI do agente demora a ficar pronta (login, atualização) → o hook espera até um tempo-limite
  e, se estourar, marca "aprendizado falhou" sem travar o agente.
- O usuário digita no agente antes do hook terminar → o hook não mistura texto com o que o usuário
  está digitando; aguarda ou desiste e avisa.
- Uma nova ligação é criada depois que o agente aprendeu → o agente descobre pelo `atlasoq list`
  (o tutorial orienta consultar `list` antes de agir); não há reenvio automático do tutorial.
- `.atlasoq/learn.md` muito grande → é truncado no limite de tamanho com aviso no nó.
- A pasta raiz não permite escrita → o app recusa criar o workspace ali e explica o motivo.

## Requirements *(mandatory)*

### Functional Requirements

**Canvas**

- **FR-001**: O sistema MUST oferecer um canvas infinito com pan (arrastar com o botão do meio,
  espaço + arrastar, touchpad) e zoom (Ctrl + roda, pinça, atalhos) entre 10% e 400%, centrado no
  cursor.
- **FR-002**: Usuários MUST conseguir selecionar (clique e retângulo), mover, redimensionar e apagar
  nós, e desfazer e refazer essas operações.
- **FR-003**: O sistema MUST oferecer "enquadrar tudo", "enquadrar seleção" e um minimapa.
- **FR-004**: Abaixo de um nível de zoom definido, ou fora da área visível, os agentes MUST ser
  exibidos como cartões resumidos (nome, papel, status, últimas linhas), sem perda de conteúdo ao
  voltar.
- **FR-005**: O foco de teclado MUST estar em exatamente um lugar (canvas, um terminal ou uma nota),
  indicado visualmente, e o usuário MUST conseguir sempre sair de um terminal (`Esc` duas vezes ou
  clique no fundo).

**Agentes**

- **FR-006**: Usuários MUST conseguir criar agentes a partir de perfis (comando + argumentos),
  incluindo perfis prontos para Claude Code, Codex, Gemini CLI, OpenCode, Aider e Shell, e criar ou
  editar perfis próprios.
- **FR-007**: Cada agente MUST ter nome único no workspace (letras minúsculas, números e hífen),
  papel opcional, diretório de trabalho, comando e variáveis de ambiente.
- **FR-008**: Cada agente MUST rodar num terminal real, com suporte a cores, TUIs em tela cheia,
  mouse, colagem, Unicode e Nerd Fonts, links clicáveis e redimensionamento.
- **FR-009**: Cada agente MUST exibir um status: `trabalhando` (produziu saída recentemente),
  `ocioso` (sem saída por um período configurável), `encerrado` (com código de saída).
- **FR-010**: Usuários MUST conseguir reiniciar, duplicar e encerrar um agente pelo nó.
- **FR-011**: Clicar em `caminho:linha` ou URL num terminal MUST abrir no editor do usuário ou no
  navegador.

**Conexões**

- **FR-012**: Usuários MUST conseguir criar conexões arrastando de um nó a outro, escolher a direção
  (→, ← ou ↔), pausar, reativar e apagar. Agentes MUST NOT criar, alterar ou remover conexões.
- **FR-013**: Uma mensagem MUST ser entregue somente se existir conexão ativa que permita aquele
  sentido; numa conexão `A → B`, A pode enviar a B e ler a tela de B, e B não pode enviar a A nem ler
  A.
- **FR-014**: Mensagens MUST ser entregues no terminal de destino como texto colado e submetido,
  precedido de identificação da origem, uma por vez, em ordem de chegada.
- **FR-015**: O sistema MUST remover sequências de controle e de escape das mensagens antes da
  entrega e MUST aplicar tamanho máximo por mensagem.
- **FR-016**: O sistema MUST pausar automaticamente uma conexão quando: (a) o número de trocas
  consecutivas sem intervenção humana passar do limite, (b) a taxa de mensagens passar do limite,
  ou (c) for detectada repetição de conteúdo em vai-e-vem. Qualquer digitação do usuário num dos
  terminais da conexão MUST zerar a contagem de (a). Os limites MUST ser configuráveis por conexão.
- **FR-017**: O sistema MUST registrar localmente cada mensagem (horário, origem, destino, conexão,
  texto ou só um resumo criptográfico, se o usuário preferir, e resultado), e o usuário MUST
  conseguir ver o histórico de cada conexão no canvas.
- **FR-018**: O canvas MUST mostrar visualmente a passagem de cada mensagem pela linha e o estado
  da conexão (ativa, pausada, pausada automaticamente).
- **FR-019**: Somente processos rodando dentro de um agente do canvas MUST conseguir usar a CLI, e
  cada requisição MUST ser atribuída de forma confiável ao agente de origem.

**CLI `atlasoq`**

- **FR-020**: A CLI MUST ter exatamente estes comandos no MVP:

  | Comando | O que faz |
  |---|---|
  | `atlasoq learn` | Mostra o tutorial para agentes |
  | `atlasoq me` | Mostra o próprio nome, papel e status |
  | `atlasoq list` | Lista agentes e notas ligados a mim, com direção e status |
  | `atlasoq send <agente> <texto>` | Envia uma mensagem e retorna logo |
  | `atlasoq read <agente>` | Mostra o texto recente da tela do agente |
  | `atlasoq ask <agente> <texto>` | Envia, espera o agente ficar ocioso e mostra a resposta |
  | `atlasoq note <nota> [texto]` | Sem texto: lê a nota. Com texto: acrescenta ao fim |
  | `atlasoq help [comando]` | Mostra a ajuda geral ou de um comando |

- **FR-021**: O texto da mensagem MUST poder vir como argumento ou pela entrada padrão (ex.:
  `cat relatorio.md | atlasoq send revisor`).
- **FR-022**: Nenhum dos usos comuns da tabela do FR-020 MUST exigir flags. Flags opcionais
  permitidas: `--json` (saída estruturada, em todos os comandos), `--lines N` (`read`) e
  `--timeout S` (`ask`).
- **FR-023**: Toda falha MUST terminar com código de saída diferente de zero e uma mensagem que
  diga o motivo e o próximo passo; erros de digitação de comando MUST sugerir o comando mais
  próximo.
- **FR-024**: `atlasoq ask` MUST ter tempo-limite padrão (proposta: 10 minutos), MUST indicar
  quando a resposta veio por tempo-limite e não por ociosidade, e MUST devolver só o que o destino
  produziu depois da mensagem.
- **FR-025**: Os códigos de saída MUST ser documentados e estáveis (ex.: 0 sucesso, 2 uso
  incorreto, 3 sem conexão/permissão, 4 destino indisponível, 5 tempo esgotado, 6 fora do AtlasOQ).

**Tutorial e documentação**

- **FR-026**: `atlasoq learn` MUST imprimir um tutorial escrito para agentes de IA, com no máximo
  ~60 linhas, contendo: o que é o AtlasOQ, como descobrir quem sou e com quem falo, um exemplo de
  cada comando, as regras de etiqueta (responder no próprio terminal, mensagens curtas, não entrar
  em vai-e-vem, usar notas para contexto longo) e os erros mais comuns.
- **FR-027**: O tutorial MUST ser personalizado com o nome e as ligações do agente que o executou,
  quando executado dentro do app.
- **FR-028**: Todo agente MUST aprender a CLI automaticamente: logo após a inicialização, o hook
  de inicialização do perfil MUST fazer o agente receber o tutorial personalizado, sem ação do
  usuário. O hook MUST rodar de novo quando o agente é reiniciado e quando o workspace é reaberto.
  NÃO MUST existir passo manual (botão ou comando do usuário) necessário para o agente aprender.
- **FR-028a**: O hook MUST suportar estes modos, escolhidos por perfil: `hook` (usa o mecanismo de
  início de sessão da própria CLI, com arquivos gerados em `.atlasoq/`), `prompt` (passa à CLI um
  prompt inicial pedindo para rodar `atlasoq learn`), `type` (envia essa instrução ao agente assim
  que ele fica pronto) e `off`. Os perfis prontos MUST vir com o modo mais adequado a cada CLI, e
  "Shell" com `off`.
- **FR-028b**: O nó MUST mostrar o estado do aprendizado (aguardando, concluído, falhou com motivo)
  e o resultado MUST ir para o log do workspace.
- **FR-028c**: O conteúdo de `.atlasoq/learn.md`, se existir, MUST ser anexado ao tutorial, com
  limite de tamanho.
- **FR-028d**: O hook MUST NOT alterar arquivos fora de `.atlasoq/` (ex.: `AGENTS.md`, `CLAUDE.md`,
  `.claude/`, configuração global das CLIs).
- **FR-029**: O projeto MUST publicar o tutorial para agentes em `docs/` (mesmo conteúdo do
  `atlasoq learn`, sem personalização) e um guia para usuários com receitas de orquestração
  (coordenador e executores; executor e revisor; nota de plano compartilhada).
- **FR-030**: MUST existir um teste automatizado que executa todos os exemplos do tutorial contra
  o app e falha se algum deixar de funcionar, e outro que verifica que um agente recém-criado
  recebe o tutorial pelo hook em cada modo (`hook`, `prompt`, `type`).

**Notas**

- **FR-031**: Usuários MUST conseguir criar notas em Markdown (CommonMark com tabelas, listas de
  tarefas e blocos de código), alternando entre edição e leitura.
- **FR-032**: Cada nota MUST ser armazenada como um arquivo Markdown próprio dentro do workspace,
  editável fora do app; alterações externas MUST aparecer no app, e conflitos com edições não
  salvas MUST ser mostrados ao usuário em vez de sobrescritos.
- **FR-033**: Conexões entre agente e nota MUST controlar o acesso: `nota → agente` permite ler;
  `agente → nota` permite ler e acrescentar; `↔` permite os dois. Agentes MUST NOT apagar ou
  reescrever conteúdo existente de notas.
- **FR-034**: Conteúdo HTML e scripts em notas MUST ser neutralizados na exibição.

**Persistência**

- **FR-035**: O sistema MUST salvar automaticamente o workspace (posições, zoom, nós, notas,
  conexões, configurações dos agentes, perfis) em arquivos de texto locais, sem rede, dentro de
  `<raiz>/.atlasoq/`.
- **FR-035a**: O diretório `.atlasoq/` MUST separar arquivos versionáveis (workspace, notas, perfis,
  `learn.md`) de estado local (logs, backups, arquivos gerados por agente), e o app MUST criar um
  `.atlasoq/.gitignore` que exclui o estado local.
- **FR-035b**: Perfis MUST existir em dois níveis: globais do usuário e do workspace
  (`.atlasoq/profiles/`); o do workspace prevalece em caso de mesmo nome.
- **FR-035c**: Antes de executar comandos, perfis, hooks ou instruções de um `.atlasoq/` que o
  usuário não criou ou não aprovou nesta máquina, o app MUST mostrá-los e pedir aprovação, e MUST
  pedir de novo quando mudarem.
- **FR-036**: Gravações MUST ser atômicas, e o sistema MUST manter cópias das últimas versões
  válidas para restauração.
- **FR-037**: Ao reabrir um workspace, os agentes MUST ser reiniciados com o mesmo comando,
  diretório e tamanho, ou ficar parados se a opção "perguntar antes de iniciar" estiver ligada.
- **FR-038**: Valores de variáveis marcadas como secretas MUST NOT ser gravados no workspace.
- **FR-039**: Usuários MUST conseguir criar, abrir e alternar entre vários workspaces, com lista de
  recentes. Um mesmo workspace MUST NOT ser aberto por duas instâncias ao mesmo tempo.

**Plataforma e distribuição**

- **FR-040**: O app MUST funcionar em sessões Wayland e X11 nos ambientes GNOME e KDE.
- **FR-041**: O app MUST ser distribuído como pacote `.deb`, com entrada no menu do sistema, ícone
  próprio e a CLI `atlasoq` disponível dentro dos terminais do app sem configuração manual.
- **FR-042**: O app MUST NOT enviar dados para fora da máquina e MUST NOT ter telemetria.
- **FR-043**: Ao fechar o app, os processos dos agentes MUST ser encerrados sem deixar processos
  órfãos.

### Key Entities

- **Workspace**: nome, viewport (posição e zoom), configurações, lista de nós e de conexões.
- **Agente (nó)**: nome único, papel, perfil de origem, comando, argumentos, diretório, variáveis de
  ambiente (com marcação de secretas), posição e tamanho, tamanho do terminal, status em tempo de
  execução.
- **Perfil de agente**: nome, comando, argumentos, variáveis padrão, modo do hook de inicialização
  (`hook`, `prompt`, `type`, `off`), nível (global ou do workspace).
- **Diretório `.atlasoq/`**: raiz física do workspace, com parte versionável e parte local.
- **Aprovação de workspace**: registro local de que o usuário aprovou os comandos e hooks de um
  `.atlasoq/` em determinado estado.
- **Nota (nó)**: nome único, arquivo Markdown, posição e tamanho.
- **Conexão**: origem, destino, direção, estado (ativa, pausada, pausada automaticamente), limites
  (trocas consecutivas, taxa).
- **Mensagem**: id, horário, origem, destino, conexão, tipo (`send`, `ask`, `note`), texto,
  contagem de trocas, resultado.
- **Registro de auditoria**: sequência de mensagens e eventos de conexão (criada, pausada,
  reativada, apagada), por workspace.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Um usuário novo instala o `.deb`, cria dois agentes, liga os dois e faz um enviar
  uma mensagem ao outro em menos de 5 minutos, sem consultar documentação externa.
- **SC-002**: Recém-criado e sem nenhuma instrução do usuário, um agente de IA de mercado (testado
  com pelo menos duas CLIs diferentes) executa corretamente, sem ajuda adicional, as tarefas "pergunte ao X",
  "mande para X" e "anote na nota Y" em pelo menos 9 de 10 tentativas.
- **SC-002a**: 100% dos agentes criados com perfis prontos (exceto "Shell") recebem o tutorial em
  até 30 segundos após ficarem prontos, sem ação do usuário.
- **SC-003**: O tutorial de `atlasoq learn` tem no máximo 60 linhas e 100% dos seus exemplos passam
  no teste automatizado.
- **SC-004**: Um agente que erra o nome de um comando recebe sugestão correta em 100% dos casos de
  uma letra trocada, faltando ou sobrando.
- **SC-005**: 100% das tentativas de mensagem sem conexão ativa, no sentido proibido, ou após pausa
  são recusadas e registradas (verificado por teste automatizado).
- **SC-006**: 100% das sequências de controle de um conjunto de teste de payloads maliciosos são
  neutralizadas antes da entrega.
- **SC-007**: Um vai-e-vem automático entre dois agentes é interrompido em no máximo 20 trocas com
  os limites padrão.
- **SC-008**: Com 10 agentes ociosos no canvas, pan e zoom ficam a pelo menos 50 quadros por
  segundo num notebook com GPU integrada; com 8 agentes produzindo saída contínua, a interface
  responde a cliques e teclas em menos de 100 ms.
- **SC-009**: Em 50 encerramentos forçados durante a edição, o workspace reabre íntegro em 100% das
  vezes, com perda máxima de 1 segundo de alterações.
- **SC-010**: Ao fechar o app, 0 processos de agentes permanecem rodando.
- **SC-011**: O `.deb` instala, abre e desinstala sem erros nas versões LTS atual e anterior do
  Ubuntu e no Debian estável, em Wayland e X11.

## Assumptions

- Usuários são desenvolvedores em Linux que já usam CLIs de agentes e têm essas CLIs instaladas e
  autenticadas por conta própria. O AtlasOQ não instala nem autentica agentes.
- O status `trabalhando/ocioso` e o fim da espera de `atlasoq ask` usam uma heurística de atividade
  de saída do terminal (silêncio por alguns segundos). É suficiente para o MVP; detecção por hooks
  das CLIs fica para depois.
- Os agentes rodam com as permissões do usuário, sem sandbox. Conectar agentes amplia o alcance de
  uma eventual injeção de prompt; isso é documentado e mitigado por conexões explícitas, limites e
  auditoria, não eliminado.
- Ameaças de processos maliciosos rodando com o mesmo usuário estão fora do modelo de ameaça.
- O processo dos agentes não sobrevive ao fechamento do app no MVP; a sobrevivência (daemon de
  sessões) é o primeiro item pós-MVP.
- Desenhos à mão livre, rpm, Flatpak, AppImage, macOS e Windows estão fora deste escopo.
- Comandos da CLI são em inglês (padrão de CLIs); o tutorial e a interface do app são em português
  no MVP, com a estrutura preparada para tradução.
- O projeto é licenciado sob Apache-2.0.
- O modo `hook` depende de cada CLI oferecer um mecanismo de início de sessão configurável por
  arquivo ou flag sem tocar na configuração do projeto; quais CLIs suportam isso será confirmado no
  plano. As demais usam `prompt` ou `type`.
- Hooks no MVP só executam o aprendizado (`atlasoq learn`); hooks com scripts arbitrários do usuário
  ficam para depois.
