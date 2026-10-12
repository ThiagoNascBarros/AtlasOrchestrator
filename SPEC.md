# AtlasOQ — Especificação (SPEC.md)

| Campo | Valor |
|---|---|
| Status | **Aprovado com alterações** (nome, stack, `.deb`); detalhes do MVP em [specs/001](specs/001-mvp-orquestracao-agentes/spec.md) |
| Versão | 0.3.0 |
| Data | 2026-10-11 |
| Constituição | v1.3.1 |
| Fontes de pesquisa | `relatorio-orquestracao-agentes-ia-desktop-claude.md`, `relatorio-orquestracao-agentes-ia-desktop-gemini.md` |

AtlasOQ é um canvas infinito para **orquestrar agentes de IA** no Linux. Cada agente roda num
terminal real, representado por um nó posicionável, ao lado de notas em Markdown. O usuário liga
agentes com linhas visíveis, e só por elas um agente consegue mandar mensagens a outro, usando
uma CLI curta (`atlasoq`) que os próprios agentes aprendem com `atlasoq learn`.

O projeto é open source e implementado do zero. Ele se inspira num conceito público ("canvas de
terminais conectados") e não reaproveita nome, identidade visual, vocabulário de produto, textos
ou código de nenhum app existente.

---

## 1. Objetivos

1. **Orquestrar agentes num canvas.** Montar um time de agentes (com nome e papel), ver de uma
   vez o que cada um está fazendo, ligar quem fala com quem e acompanhar as mensagens, num canvas
   que se navega com pan e zoom.
2. **Terminais reais.** Cada nó roda um PTY real, então Claude Code, Codex, OpenCode, Aider, um
   shell ou uma TUI qualquer funcionam sem adaptação (princípio III).
3. **Comunicação entre agentes explícita, visível e auditável.** Um agente só fala com outro se o
   usuário desenhou a conexão. Toda mensagem fica registrada localmente e pode ser revisada
   (princípio IV).
4. **Nunca perder o workspace.** Layout, notas e conexões ficam em arquivos de texto locais,
   gravados de forma atômica, legíveis e versionáveis com git (princípio V).
5. **Linux de primeira classe.** Funcionar em sessões Wayland e X11, nos ambientes GNOME e KDE,
   com pacotes nativos (princípio II).
6. **CLI que agentes aprendem sozinhos.** Poucos comandos, curtos e previsíveis, com um tutorial
   embutido e testado (princípios VIII e IX).
7. **Base de engenharia sólida.** Camadas com fronteiras claras e testes automatizados na camada
   PTY, na persistência e na troca de dados entre terminais.

## 2. Não-objetivos

Fora do escopo do projeto (pelo menos até nova revisão deste documento):

- **macOS e Windows.** Nenhuma decisão é tomada para mantê-los viáveis.
- **Ser um agente de IA.** O AtlasOQ não chama LLMs, não tem chave de API e não interpreta o que
  os agentes dizem. Ele hospeda CLIs de terceiros.
- **Sincronização em nuvem, contas ou login.** Nada sai da máquina por padrão. Não há telemetria.
- **Colaboração em tempo real** entre várias pessoas no mesmo canvas.
- **Pipe bruto entre terminais** (saída de A copiada byte a byte para a entrada de B). Ele mistura
  sequências ANSI, gera loops e abre espaço para injeção de teclas. A comunicação é sempre por
  mensagens (seção 6.4).
- **Sandbox de agentes** (bubblewrap, Landlock, containers). É desejável, mas fica para depois do
  MVP. No MVP os processos rodam com o UID do usuário, sem elevação (ver Riscos).
- **Substituir o emulador de terminal do usuário** ou o tmux.
- **Plugins** e API pública estável.

## 3. Funcionalidades do MVP

O MVP segue o princípio VII: canvas, terminais, notas, conexões e persistência. Cada item tem
critério de aceite verificável.

### 3.1 Canvas infinito com pan e zoom

- Pan com arrastar do botão do meio, espaço + arrastar, ou dois dedos no touchpad.
- Zoom com Ctrl + roda, pinça no touchpad e atalhos (`Ctrl+=`, `Ctrl+-`, `Ctrl+0`). Faixa de 10%
  a 400%, centrado no cursor.
- Grade de fundo opcional e "encaixar na grade" ao mover nós.
- "Enquadrar tudo" e "enquadrar seleção".
- Minimapa simples (retângulos) para orientação.
- Seleção por clique e por retângulo; mover, redimensionar e apagar múltiplos nós.
- Desfazer e refazer para operações de layout (mover, redimensionar, criar, apagar, conectar).

**Aceite:** com 10 terminais abertos (ociosos), pan e zoom mantêm 60 fps num notebook de
referência com GPU integrada.

### 3.2 Agentes (nós de terminal)

- Criar agente com: nome único, papel opcional, diretório de trabalho, comando (padrão: `$SHELL` de login) e
  variáveis de ambiente extras.
- Perfis prontos e editáveis: Claude Code, Codex, Gemini CLI, OpenCode, Aider e Shell. Um perfil é
  só um comando com argumentos; não há integração específica com nenhuma CLI no MVP.
- Status visível no nó: `trabalhando`, `ocioso` (heurística de silêncio na saída) ou `encerrado`.
- Hook de inicialização: logo após subir, o agente recebe o tutorial do `atlasoq learn`
  automaticamente (seção 6.4.7). O nó mostra o estado do aprendizado.
- Terminal completo: cores de 24 bits, TUIs em tela alternativa, mouse, bracketed paste, OSC 8
  (hyperlinks), Unicode e Nerd Fonts, redimensionamento com SIGWINCH.
- Scrollback configurável (padrão de 10 mil linhas), busca no scrollback, copiar e colar.
- Clicar em `caminho:linha` ou em URL abre no `$EDITOR` ou via `xdg-open`.
- Barra de título do nó com: nome, papel, status, estado do aprendizado, cwd e menu (reiniciar,
  duplicar, encerrar).
- Foco determinístico: só um nó recebe teclado por vez, com indicação visual clara. `Esc Esc`
  devolve o foco ao canvas. A política de foco é uma máquina de estados explícita e testada.

**Aceite:** `htop`, `vim`, `claude`, `codex` e `less` funcionam igual a um terminal comum; o
redimensionamento do nó atualiza `stty size` no processo.

### 3.3 Notas em Markdown

- Nó de nota com modo leitura (Markdown renderizado) e modo edição (editor de texto com realce).
- Suporte a CommonMark + tabelas, listas de tarefas e blocos de código com realce.
- HTML embutido é sanitizado; links abrem no navegador do sistema.
- Cada nota é um arquivo `.md` no diretório do workspace, editável fora do app.

- Uma nota pode ser ligada a agentes: `nota → agente` dá leitura; `agente → nota` dá leitura e
  acréscimo (`atlasoq note`). Agentes nunca apagam nem reescrevem o conteúdo existente.

**Aceite:** uma nota editada fora do app (com o app fechado) aparece atualizada ao reabrir.

### 3.4 Conexões entre agentes

- O usuário arrasta de um conector de um terminal até outro terminal para criar a conexão.
- Direção: A→B, B→A ou bidirecional. Estado: ativa ou pausada. Ambos visíveis na linha.
- Um agente rodando em A envia mensagem a B com `atlasoq send b "..."` ou pergunta e espera a
  resposta com `atlasoq ask b "..."` (seção 6.4). A mensagem só é entregue se a conexão existir,
  estiver ativa e permitir essa direção.
- A mensagem chega no PTY de B como texto colado, com um cabeçalho de origem, e é submetida.
- Um agente pode **ler** a tela e o scrollback recente de um terminal conectado (respeitando a
  direção), para obter a resposta do outro lado.
- A linha anima quando uma mensagem passa. Clicar na linha mostra o histórico daquela conexão.
- Apagar ou pausar a conexão bloqueia novas entregas imediatamente.
- Proteção contra loops: limite de trocas automáticas consecutivas e limite de taxa por conexão
  (seção 6.4.4). Ao estourar, a conexão é pausada e o usuário vê um aviso.
- Log de auditoria local de todas as mensagens, entregues ou recusadas.

**Aceite:** dois agentes (ou dois scripts de teste) trocam 10 mensagens por uma conexão
bidirecional; ao pausar a conexão, a 11ª é recusada com erro claro e registrada no log.

### 3.5 Persistência local

- Um workspace mora em `<raiz>/.atlasoq/`, onde `<raiz>` é a pasta escolhida pelo usuário
  (normalmente a raiz do projeto). Estrutura na seção 6.5.
- Ao abrir um `.atlasoq/` que o usuário não criou ou não aprovou nesta máquina (ex.: repositório
  clonado), o app mostra comandos, perfis, hooks e `learn.md` e só inicia agentes após aprovação.
- Salvamento automático com debounce (cerca de 500 ms após a última alteração) e escrita atômica.
- Ao reabrir, o canvas, as notas e as conexões voltam como estavam. Cada terminal é **recriado**
  com o mesmo comando, cwd e tamanho (o processo anterior não sobrevive ao fechamento do app no
  MVP; ver seção 6.1 e milestone pós-MVP "daemon").
- Ao reabrir um workspace aprovado, todos os agentes iniciam sozinhos, sem pergunta (clarify
  2026-10-11).
- Múltiplos workspaces, com lista de recentes.
- Nenhum segredo é serializado: variáveis de ambiente marcadas como secretas ficam fora do JSON.

**Aceite:** `kill -9` no app durante uma edição não corrompe o `workspace.json`; o pior caso é
perder os últimos ~500 ms de alterações de layout.

### 3.6 CLI `atlasoq` e tutorial para agentes

- Oito comandos curtos, sem flags no uso comum (seção 6.4.2).
- `atlasoq learn` imprime um tutorial de até ~60 linhas, escrito para agentes, personalizado com o
  nome e as conexões de quem o executou.
- Mesmo tutorial em `docs/agentes/tutorial.md` e um guia de receitas de orquestração para usuários
  em `docs/guia-orquestracao.md`.
- Teste automatizado que executa todos os exemplos do tutorial.

- Todo agente aprende sozinho ao nascer, por hook de inicialização (seção 6.4.7). Não há botão nem
  passo manual.

**Aceite:** um agente de mercado recém-criado, sem nenhuma instrução do usuário, consegue
perguntar algo a outro agente e anotar numa nota ligada.

### 3.6a Memória de longo prazo por agente

- Cada agente tem um arquivo `.atlasoq/memory/<nome>.md` com anotações curtas e datadas.
- O agente escreve com `atlasoq note memory "feito: ..."` (`memory` é nome reservado para a própria
  memória; a CLI continua com 8 comandos). Prefixos: `feito:`, `decisão:`, `pendente:`; até 300
  caracteres por anotação.
- Ao nascer, o hook entrega o tutorial **e** as últimas 20 anotações (no máximo ~2 KB, cerca de 600
  tokens). O resto fica no arquivo e pode ser lido com `atlasoq note memory`.
- Só o próprio agente lê e escreve a sua memória pela CLI; o usuário vê e edita pelo nó.
- O app não resume nada (não chama IA); a economia de tokens vem das anotações curtas e do limite
  de entrega. Compactação automática fica para depois do MVP.
- Por padrão a memória não é versionada (`memory/` no `.atlasoq/.gitignore`); um botão nas
  configurações do workspace passa a versioná-la, e outro volta atrás (clarify 2026-10-11).

### 3.7 Instalação

- Pacote `.deb` (único formato oficial por enquanto), com `.desktop`, ícone e a CLI `atlasoq`
  instalada em `/usr/bin`.

### 3.8 Fora do MVP (mas previsto na arquitetura)

Desenhos à mão livre e formas; daemon de sessões que sobrevive à UI; detecção de estado por hooks
das CLIs (incluindo "pedindo permissão") com notificações D-Bus; worktree git por nó; servidor MCP
expondo o canvas; rpm e Flatpak; sandbox por nó. Ver seção 8.

---

## 4. Stack: comparação das opções

As três opções avaliadas atendem ao requisito de PTY real. A diferença está no terminal dentro de
um canvas com zoom, no custo de memória, no empacotamento e no risco em Wayland.

### 4.1 Opção A — Tauri v2 + xterm.js + portable-pty (Rust)

Backend em Rust (PTY, persistência, broker de mensagens) e UI web (TypeScript) dentro do
WebKitGTK do sistema.

**Prós**
- `portable-pty` (MIT, do projeto WezTerm) é maduro; Rust dá controle fino de processos, sinais e
  sockets Unix, e o mesmo código serve depois para um daemon separado.
- xterm.js (MIT) é o emulador web mais usado (VS Code e outros); suporta Unicode, OSC 8, mouse,
  tela alternativa e renderizador WebGL.
- Canvas em DOM/CSS deixa compor terminais, notas e linhas com pouco código, e Markdown sai quase
  de graça (markdown-it, CodeMirror).
- Binário pequeno e consumo de memória base bem menor que o Electron.
- Bom alinhamento com o que os relatórios apontam como tendência (backend Rust + Tauri).

**Contras**
- WebKitGTK varia entre distros: desempenho de WebGL, renderização DMA-BUF (tela em branco em
  algumas combinações de GPU/driver), bugs de compositing em Wayland. Exige testes em várias
  distros desde o início.
- xterm.js dentro de elementos com `transform: scale()` tem problemas conhecidos de nitidez e de
  coordenadas do mouse/seleção. O zoom de terminais precisa de estratégia própria (seção 6.2.2).
- IPC entre Rust e WebView precisa de cuidado com vazão: saída de vários terminais em alta
  frequência tem que ser agrupada por quadro.
- Duas linguagens (Rust e TypeScript) no projeto.

### 4.2 Opção B — Electron + xterm.js + node-pty

**Prós**
- Chromium embutido: renderização idêntica em todas as distros, WebGL previsível, DevTools
  completos.
- node-pty + xterm.js é a combinação mais testada do mercado (VS Code).
- Uma linguagem só (TypeScript) no projeto inteiro.

**Contras**
- Memória e disco: centenas de MB de RAM antes do primeiro agente, pacote acima de 100 MB. Os
  relatórios citam esse custo como reclamação recorrente de apps do gênero.
- Atualizações de segurança do Chromium viram responsabilidade do projeto, com releases
  frequentes.
- Problemas próprios de Wayland (flags de Ozone, APIs que travam a inicialização, citados no
  relatório).
- node-pty é módulo nativo: rebuild por versão do Electron.
- Diferenciação menor: é a stack de vários concorrentes.

### 4.3 Opção C — GTK4 + Rust (VTE para terminais)

**Prós**
- O mais nativo possível: acessibilidade, temas, IME, Wayland, portais XDG e D-Bus sem camada
  intermediária.
- VTE é o motor de terminal do GNOME, robusto e bem integrado.
- Memória mínima e uma linguagem só (Rust).
- Melhor caminho para Flatpak/Flathub.

**Contras**
- Canvas com zoom de widgets VTE é difícil: escalar um `VteTerminal` exige truques com
  `GtkSnapshot` ou trocar o tamanho da fonte por nó, e o comportamento em zoom contínuo é pouco
  explorado.
- Canvas infinito, linhas, seleção e minimapa precisam ser desenhados à mão.
- Markdown renderizado em GTK não tem solução pronta de qualidade (GtkTextView manual ou um
  WebView, o que traz de volta parte da Opção A).
- VTE é LGPL; compatível com MIT/Apache por conexão dinâmica, mas exige atenção.
- Já existe um projeto embrionário nesse nicho (GTK4 + VTE), segundo o relatório.
- Curva de gtk4-rs mais íngreme e menos gente para contribuir.

### 4.4 Comparação resumida

| Critério | A: Tauri | B: Electron | C: GTK4/Rust |
|---|---|---|---|
| Terminal com zoom no canvas | Bom, com estratégia própria | Bom, com estratégia própria | Difícil |
| Notas Markdown | Fácil | Fácil | Difícil |
| Memória base | Baixa | Alta | Muito baixa |
| Consistência entre distros | Média (WebKitGTK) | Alta | Alta |
| Wayland | Risco médio | Risco médio | Baixo |
| Empacotamento (deb/rpm/Flatpak) | Bom | Bom, pacote grande | Muito bom |
| Robustez de PTY e processos | Alta (Rust) | Média (node-pty) | Alta (Rust) |
| Velocidade para o MVP | Alta | Muito alta | Baixa |

### 4.5 Decisão (aprovada em 2026-10-11)

**Opção A: Tauri v2 + xterm.js + portable-pty.** Ela entrega o canvas com terminais e Markdown
rápido como o Electron, mas com um núcleo em Rust que é o ponto mais crítico (PTY, processos,
broker de mensagens, persistência) e que depois vira o daemon sem reescrita. O maior risco
(WebKitGTK e zoom de xterm.js) é atacado num spike de uma semana no M0, com critério objetivo.

**Plano B:** se o spike do M0 reprovar o WebKitGTK (seção 8, critérios), a UI migra para Electron
mantendo o núcleo Rust como processo separado. Por isso o núcleo não depende de nada do Tauri
(seção 6).

### 4.6 Componentes concretos propostos

| Camada | Escolha | Licença | Observação |
|---|---|---|---|
| Shell da aplicação | Tauri v2 | MIT/Apache-2.0 | Canais IPC (`tauri::ipc::Channel`) para streaming |
| PTY | `portable-pty` | MIT | Encapsulado num trait próprio |
| Parser VT headless (backend) | `vt100` ou `alacritty_terminal` | MIT / Apache-2.0 | Para `atlasoq read` e para saber se o alvo ativou bracketed paste |
| Async | `tokio` | MIT | |
| Serialização | `serde` + `serde_json` | MIT/Apache-2.0 | |
| UI | TypeScript + React + Zustand | MIT | Canvas próprio em DOM/CSS, sem biblioteca de grafos no MVP |
| Terminal | `@xterm/xterm` + addons `webgl`, `fit`, `search`, `web-links`, `unicode11` | MIT | Fallback para renderizador DOM |
| Editor de notas | CodeMirror 6 | MIT | |
| Markdown | `markdown-it` + DOMPurify | MIT / Apache-2.0 ou MPL-2.0 | |
| Testes | `cargo test`, Vitest, WebdriverIO + `tauri-driver` | MIT | E2E no Linux via WebKitWebDriver |

As versões serão fixadas em `Cargo.lock` e `package-lock.json`/`pnpm-lock.yaml`, e a licença de
cada dependência será registrada em `docs/licenses.md` (princípio I, verificação automatizada com
`cargo-deny` e `license-checker`).

**Licença do projeto:** **Apache-2.0** (aprovada em 2026-10-11; arquivo `LICENSE` na raiz).

---

## 5. Visão geral da arquitetura

```
┌──────────────────────────── Processo da UI (WebView) ────────────────────────────┐
│  Camada de Canvas                                                                │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────┐  ┌──────────────────────────┐ │
│  │ Viewport     │  │ Nós          │  │ Conexões  │  │ Estado do workspace      │ │
│  │ pan/zoom/LOD │  │ term / nota  │  │ (SVG)     │  │ (store + undo/redo)      │ │
│  └──────────────┘  └──────┬───────┘  └─────┬─────┘  └────────────┬─────────────┘ │
│                    xterm.js por nó         │                     │               │
└────────────────────────────┬───────────────┼─────────────────────┼───────────────┘
                 IPC Tauri (comandos + canais de bytes, agrupados por quadro)
┌────────────────────────────┴───────────────┴─────────────────────┴───────────────┐
│  Núcleo Rust (crates sem dependência de Tauri)                                   │
│  ┌───────────────────┐  ┌──────────────────────────┐  ┌───────────────────────┐  │
│  │ atlasoq-pty       │  │ atlasoq-link (broker)    │  │ atlasoq-store         │  │
│  │ SessionHost       │◄─┤ grafo de conexões, ACL,  │  │ workspace.json,       │  │
│  │ spawn/resize/kill │  │ entrega, loop guard,     │  │ notas, escrita        │  │
│  │ parser VT headless│  │ log de auditoria         │  │ atômica, migrações    │  │
│  └─────────┬─────────┘  └────────────▲─────────────┘  └───────────────────────┘  │
└────────────┼─────────────────────────┼───────────────────────────────────────────┘
             │ PTY master              │ socket Unix (0600, SO_PEERCRED, token por nó)
     ┌───────▼───────┐          ┌──────┴────────┐
     │ shell/agente  │─────────►│ CLI `atlasoq` │  (invocada pelo agente dentro do PTY)
     │ (PTY slave)   │          └───────────────┘
     └───────────────┘
```

Organização do repositório (substitui a estrutura provisória do README):

```
AtlasOrchestrator/
├── SPEC.md
├── docs/                    # ADRs, threat model, licenças, tutorial para agentes, guia
├── specs/                   # Specs por funcionalidade (Spec Kit), ex.: 001-mvp-orquestracao-agentes
├── crates/
│   ├── atlasoq-pty/         # Camada PTY
│   ├── atlasoq-link/        # Orquestração agente-a-agente (broker)
│   ├── atlasoq-store/       # Persistência do workspace
│   └── atlasoq-cli/         # Binário `atlasoq` usado pelos agentes
├── app/
│   ├── src-tauri/           # Cola Tauri: comandos, canais, ciclo de vida
│   └── ui/                  # TypeScript: canvas, nós, conexões, notas
├── packaging/               # .deb, .desktop, ícones
└── tests/                   # Integração e E2E
```

---

## 6. Arquitetura por camada

### 6.1 Camada PTY (`atlasoq-pty`)

**Responsabilidade:** criar, alimentar, redimensionar e encerrar processos em PTYs, e expor a saída
como fluxo de bytes.

- Trait `SessionHost` com `spawn(spec) -> SessionId`, `write(id, bytes)`, `resize(id, cols, rows)`,
  `kill(id, signal)`, `subscribe(id) -> stream de bytes`, `snapshot(id) -> texto da tela`.
- Implementação `LocalSessionHost` sobre `portable-pty`. No MVP ela roda dentro do processo do app.
  Depois do MVP, a mesma interface ganha a implementação `DaemonSessionHost`, que conversa com um
  processo `atlasoqd` dono dos PTYs (sessões sobrevivem a crash e reinício da UI). Essa é a
  principal evolução de robustez apontada pela pesquisa e fica preparada desde já.
- Cada sessão tem uma thread de leitura que entrega bytes a: (1) um **parser VT headless** que
  mantém a tela e o scrollback no backend e (2) um buffer de envio para a UI.
- **Vazão:** a saída é agrupada e enviada à UI no máximo a cada ~16 ms por sessão, por um
  `Channel` binário. Se a UI estiver atrasada (nó fora da tela ou minimizado), o backend mantém só
  o estado do parser e manda um snapshot quando o nó volta a ficar visível. Isso impede que um
  agente verboso trave a interface.
- **Ambiente injetado** em cada PTY: `ATLASOQ_SOCKET`, `ATLASOQ_NODE_ID`, `ATLASOQ_NODE_TOKEN`,
  `TERM=xterm-256color`, `COLORTERM=truecolor`. O diretório da CLI `atlasoq` entra no `PATH`.
- **Ciclo de vida:** o processo filho é líder de sessão (`setsid`) e tem `PR_SET_PDEATHSIG`
  configurado, para não sobrar órfão se o app morrer. Ao fechar o app, envia SIGHUP ao grupo,
  espera um tempo e manda SIGKILL.
- Sem elevação de privilégio; nenhum comando com `sudo` por conta própria (constituição).

**Testes:** spawn de `sh -c 'printf ...'` com verificação da saída; resize refletido em
`stty size`; kill e código de saída; ausência de processos órfãos após encerramento; vazão com
um processo gerando 50 MB de saída.

### 6.2 Camada de Canvas (`app/ui`)

#### 6.2.1 Modelo

- Coordenadas de mundo (float) independentes da tela. Viewport = `{x, y, zoom}`.
- Um container "mundo" recebe `transform: translate(...) scale(...)` para notas, linhas e
  placeholders. As linhas de conexão são SVG numa camada própria, recalculadas a partir das
  bordas dos nós.
- Store único (Zustand) com histórico de comandos para desfazer e refazer. Toda mutação passa por
  comandos, o que também alimenta o salvamento automático.

#### 6.2.2 Terminais e zoom (ponto mais delicado)

Escalar um xterm.js com CSS borra o texto e desalinha mouse e seleção. Estratégia proposta, a
validar no spike do M0:

1. **Durante o gesto de zoom:** os terminais são escalados por CSS junto com o mundo (fluido,
   provisoriamente borrado).
2. **Quando o zoom para** (debounce de ~120 ms): cada terminal visível sai do container escalado e
   é re-renderizado em tamanho real com `fontSize = base × zoom`, mantendo **o mesmo número de
   colunas e linhas**. O PTY não recebe SIGWINCH por causa de zoom, só quando o usuário
   redimensiona o nó.
3. **Nível de detalhe (LOD):** abaixo de ~35% de zoom, ou fora da área visível, o xterm.js é
   desmontado e o nó vira um cartão leve (título, status, últimas linhas como texto). O parser do
   backend continua com o estado completo, então nada se perde; ao voltar, o terminal é remontado
   a partir do snapshot.
4. Renderizador WebGL quando disponível, com fallback automático para o renderizador DOM. Há um
   limite de contextos WebGL simultâneos no navegador, então só os terminais visíveis em zoom
   legível ficam com WebGL.

#### 6.2.3 Foco e entrada

Máquina de estados explícita: `Canvas` → `NóSelecionado` → `TerminalComFoco` / `NotaEditando`.
Atalhos do canvas só valem fora de `TerminalComFoco`. Sair do terminal é sempre possível com
`Esc Esc` ou clique no fundo do canvas. O estado é exibido na barra de status. A pesquisa aponta
travamento de foco como a reclamação mais grave em apps do gênero, então essa máquina tem testes
unitários próprios e testes E2E.

### 6.3 Notas

Componente de nó com CodeMirror 6 (edição) e markdown-it + DOMPurify (leitura). O conteúdo vive
em `notes/<id>.md`. O app observa o diretório (inotify) e recarrega notas alteradas fora dele; se
houver conflito com uma edição não salva, mostra as duas versões em vez de sobrescrever.

### 6.4 Orquestração agente-a-agente (`atlasoq-link`)

#### 6.4.1 Princípio

Os agentes não leem nem escrevem no PTY um do outro diretamente. Eles chamam a CLI `atlasoq`, que
fala com o broker no app. O broker decide, a partir do grafo de conexões que **só o usuário edita**,
se a operação é permitida, executa e registra. Nenhum agente consegue criar, alterar ou remover
conexões (constituição: "Nenhum terminal MUST ter permissão para criar conexões sozinho").

#### 6.4.2 Comandos da CLI (MVP)

Regras de desenho (princípio VIII): uma palavra curta em inglês por comando, no máximo 8 comandos,
agentes e notas chamados pelo **nome visível no canvas**, nenhuma flag no uso comum, erros que
dizem o próximo passo, saída em texto com `--json` opcional.

| Comando | Efeito | Requer |
|---|---|---|
| `atlasoq learn` | Imprime o tutorial para agentes, personalizado | — |
| `atlasoq list` | Quem sou (nome, papel) e os agentes e notas ligados a mim, com direção e status | — |
| `atlasoq send <agente> <texto>` | Entrega a mensagem e retorna logo | conexão ativa eu→agente |
| `atlasoq read <agente>` | Texto recente da tela do agente, sem ANSI | conexão ativa eu→agente |
| `atlasoq ask <agente> <texto>` | Envia uma pergunta e espera a resposta dada com `reply`; no tempo-limite devolve a leitura da tela, com aviso | conexão ativa eu→agente |
| `atlasoq reply [agente] <texto>` | Responde a uma pergunta pendente; a resposta vai só para o `ask` que espera | pergunta pendente endereçada a mim |
| `atlasoq note <nota> [texto]` | Sem texto: lê a nota. Com texto: acrescenta ao fim, com autor e hora | leitura: qualquer conexão; acréscimo: agente→nota |
| `atlasoq help [comando]` | Ajuda geral ou de um comando, com exemplo | — |

Detalhes:
- O texto pode vir como argumento ou pela entrada padrão: `cat plano.md | atlasoq send coder`.
- Flags opcionais: `--json` (todos), `--lines N` (`read`), `--timeout S` (`ask`, padrão de 10 min).
- A pergunta chega ao destino como `[atlasoq ask de:<origem> id:<n>] <texto> — responda com:
  atlasoq reply "..."`. Com mais de uma pergunta pendente, `reply` exige o nome de quem perguntou.
- `reply` vale mesmo numa conexão só de ida (quem perguntou → quem responde), apenas enquanto a
  pergunta está pendente; conta como troca na proteção contra loops.
- Comando digitado errado sugere o mais próximo (`atlasoq sned` → "você quis dizer `send`?").
- Códigos de saída estáveis: 0 sucesso, 2 uso incorreto, 3 sem conexão ou permissão, 4 destino
  indisponível, 5 tempo esgotado, 6 executado fora do AtlasOQ.

Exemplo de erro:

```
$ atlasoq send planner "pronto"
erro: não existe conexão de "coder" para "planner".
      Você só pode enviar para: revisor.
      Para falar com "planner", peça ao usuário para ligar coder → planner no canvas.
```

A direção vale para os dois lados: numa conexão A→B, A pode enviar para B e ler a tela de B ("A
comanda B e vê o resultado"). B não envia para A nem lê A. Numa conexão bidirecional, os dois
podem tudo.

#### 6.4.3 Entrega no PTY de destino

1. Sanitização: remove caracteres de controle e sequências de escape (mantém texto imprimível,
   `\n` e `\t`), aplica limite de tamanho (padrão de 16 KiB) e normaliza quebras de linha.
2. Cabeçalho de origem configurável, por padrão `[atlasoq de:<nome-origem> msg:<id>]`, seguido do
   texto.
3. Se o parser headless indicar que o alvo ativou bracketed paste (modo 2004), o texto vai entre
   `ESC[200~` e `ESC[201~`; senão, vai como texto simples.
4. Após um intervalo curto (configurável, ~150 ms), envia `\r` para submeter. Duas mensagens são
   sempre entregues em sequência, nunca coladas na mesma submissão.
5. Uma fila por destino serializa as entregas.

O passo 1 é a defesa principal contra um agente comprometido mandar sequências de controle ou
teclas especiais (Ctrl-C, ESC, setas) para outro. Enviar teclas brutas é um não-objetivo do MVP.

#### 6.4.4 Proteção contra loops e abuso

- **Trocas consecutivas:** se um agente envia para quem lhe mandou mensagem nos últimos 120 s,
  isso conta como resposta e incrementa o contador de trocas da conexão. Acima de `max_hops`
  (padrão de 20), a conexão é pausada. (Sem flag `--reply-to`, para manter a CLI simples.)
- **Entrada humana zera o contador:** qualquer tecla digitada pelo usuário em um dos dois
  terminais zera a contagem da conexão.
- **Limite de taxa** por conexão (padrão de 6 mensagens por minuto, com rajada de 3).
- **Detector de ping-pong:** N trocas seguidas com conteúdo quase idêntico pausam a conexão.
- Ao pausar, a linha muda de estado, aparece um aviso no canvas e o evento vai para o log. Só o
  usuário reativa.

#### 6.4.5 Autenticação e transporte

- Socket Unix em `$XDG_RUNTIME_DIR/atlasoq/<instância>.sock`, diretório `0700`, socket `0600`.
- O broker verifica `SO_PEERCRED` (mesmo UID) e exige `ATLASOQ_NODE_TOKEN`, um token aleatório de
  256 bits por nó, gerado a cada spawn, guardado só em memória e nunca persistido. O token
  identifica o nó de origem; sem ele, a requisição é recusada.
- Protocolo: JSON por linha, versionado (`{"v":1,"op":"send",...}`), documentado em
  `docs/protocol.md`.

#### 6.4.6 Auditoria

`logs/links-AAAA-MM-DD.jsonl` no workspace, uma linha por evento: `ts`, `msg_id`, `from`, `to`,
`conn_id`, `hop`, `bytes`, `resultado` (`entregue`, `recusado:<motivo>`, `pausado:<motivo>`) e o
texto da mensagem. O texto pode ser omitido nas configurações, guardando só o hash. Rotação
diária e retenção configurável. O painel da conexão lê esse log.

#### 6.4.7 Tutorial para agentes (princípio IX)

- **Fonte única:** `docs/agentes/tutorial.md`, embutido no binário da CLI na compilação. `atlasoq
  learn` imprime esse texto e, dentro do app, acrescenta um cabeçalho com "você é `<nome>`, papel
  `<papel>`, ligado a: ...".
- **Formato:** no máximo ~60 linhas, frases curtas, um bloco de exemplo por comando, seção "regras"
  (pergunta recebida se responde com `reply`; use `ask` quando precisar da resposta e `send` quando não;
  mensagens curtas; contexto longo vai numa nota; não fique em vai-e-vem; se um comando falhar, leia
  o erro, ele diz o que fazer) e seção "erros comuns".
- **Personalização:** `.atlasoq/learn.md` (opcional, versionável) é anexado ao tutorial com as
  regras do time (ex.: "o revisor nunca edita código"), limitado a ~4 KiB.
- **Entrega automática por hook de inicialização** (sem botão, sem passo manual). O perfil de cada
  agente define o modo:

  | Modo | Como funciona | Uso |
  |---|---|---|
  | `hook` | O app gera em `.atlasoq/agents/<nome>/` a configuração do hook de início de sessão da própria CLI, executando `atlasoq learn`, e passa esse arquivo à CLI por flag ou variável de ambiente. A saída entra no contexto do agente sem gastar um turno | CLIs com hooks de sessão configuráveis sem tocar no projeto (Claude Code via `--settings`; demais a confirmar no plano) |
  | `prompt` | O app inicia a CLI com o tutorial e a memória embutidos como prompt inicial (não pede para rodar comando, o que dispararia pedido de permissão) | Codex, Gemini CLI, OpenCode (ver research.md R4) |
  | `type` | O app espera o agente ficar pronto (primeira ociosidade após o início) e envia o mesmo conteúdo pelo caminho normal de entrega | Aider; fallback universal |
  | `off` | Nada | Perfil "Shell" |

  O hook roda ao criar o agente, ao reiniciá-lo e ao reabrir o workspace. O nó mostra
  `aprendendo…`, `pronto` ou `aprendizado falhou: <motivo>`; o resultado vai para o log.
  Se o usuário estiver digitando no agente, o modo `type` espera ou desiste e avisa, sem misturar
  texto. Tempo-limite padrão para o agente ficar pronto: 60 s.
- **Sem efeitos fora de `.atlasoq/`:** o app não grava `AGENTS.md`, `CLAUDE.md`, `.claude/` nem a
  configuração global das CLIs.
- **Guia para usuários:** `docs/guia-orquestracao.md` com receitas: coordenador → executores,
  executor ↔ revisor, nota de plano compartilhada, e como escrever o papel de cada agente.
- **Teste:** um teste de integração sobe dois agentes com shells, extrai cada bloco de exemplo do
  tutorial e o executa, verificando o efeito esperado. Mudou a CLI e não atualizou o tutorial, o
  teste falha. Outro teste usa uma CLI falsa para cada modo de hook e verifica que o agente
  recém-criado recebe o tutorial.

### 6.5 Armazenamento do workspace (`atlasoq-store`)

```
<raiz>/                          # pasta escolhida pelo usuário, normalmente a raiz do projeto
└── .atlasoq/
    ├── workspace.json           # canvas, nós, conexões                       (versionável)
    ├── notes/plano.md           # uma nota por arquivo                        (versionável)
    ├── profiles/revisor.json    # perfis do workspace, sobrepõem os globais   (versionável)
    ├── learn.md                 # regras do time anexadas ao tutorial         (versionável, opcional)
    ├── memory/<nome>.md         # memória de longo prazo de cada agente       (local por padrão; botão "Versionar memória")
    ├── agents/<nome>/           # gerado a cada início (config de hook etc.)  (local)
    ├── logs/                    # auditoria de mensagens e hooks              (local)
    ├── backups/                 # últimas N versões do workspace.json         (local)
    └── .gitignore               # criado pelo app: agents/ logs/ backups/ memory/

~/.config/atlasoq/               # nível do usuário
├── profiles/*.json              # perfis globais (os prontos vêm com o app)
└── settings.json
~/.local/state/atlasoq/
├── recent.json                  # workspaces recentes
└── trust.json                   # aprovações de workspace (hash dos comandos/hooks aprovados)
```

Exemplo de perfil (`profiles/claude-code.json`):

```json
{ "name": "claude-code", "command": "claude", "args": [],
  "learn": { "mode": "hook" }, "env": {} }
```

Exemplo de `workspace.json`:

```json
{
  "schema": 1,
  "id": "ws_01J...",
  "name": "refactor-auth",
  "viewport": { "x": -320.5, "y": 140.0, "zoom": 0.85 },
  "settings": {},
  "nodes": [
    {
      "id": "t_a1", "kind": "agent", "name": "planner", "role": "planeja e distribui tarefas",
      "profile": "claude-code",
      "rect": { "x": 0, "y": 0, "w": 720, "h": 460 },
      "terminal": {
        "command": "claude", "args": [], "cwd": ".",
        "env": { "FOO": "bar" }, "env_secret_keys": ["ANTHROPIC_API_KEY"],
        "cols": 100, "rows": 30
      }
    },
    { "id": "n_7f3a", "kind": "note", "name": "plano", "rect": { "x": 760, "y": 0, "w": 380, "h": 300 },
      "note": { "file": "notes/plano.md" } }
  ],
  "connections": [
    { "id": "c_1", "from": "t_a1", "to": "t_b2", "direction": "both", "state": "active",
      "limits": { "max_hops": 20, "rate_per_min": 6 } }
  ]
}
```

- **Escrita atômica:** grava em `workspace.json.tmp` no mesmo diretório, `fsync`, `rename`,
  `fsync` do diretório (princípio V). O mesmo vale para notas.
- **Caminhos relativos:** `cwd` é relativo à `<raiz>`, para o workspace funcionar em outra máquina
  ou outro clone.
- **Confiança:** ao abrir, o app calcula um hash de comandos, perfis, hooks e `learn.md`; se não
  bater com `trust.json`, mostra o que será executado e pede aprovação antes de iniciar agentes.
- **Backups:** mantém as últimas 10 versões em `backups/`. Se o `workspace.json` estiver
  ilegível na abertura, o app oferece restaurar o backup mais recente válido.
- **Migrações:** campo `schema`; migrações puras e testadas para cada versão.
- **Segredos:** chaves listadas em `env_secret_keys` têm o valor lido do keyring do sistema (Secret
  Service via D-Bus) ou do ambiente do app, e nunca vão para o JSON.
- **Fora do MVP:** scrollback dos terminais não é persistido (com o daemon, a sessão inteira
  sobrevive).

---

## 7. Riscos

| # | Risco | Prob. | Impacto | Mitigação |
|---|---|---|---|---|
| R1 | **WebKitGTK em Wayland**: tela branca com renderizador DMA-BUF em certas GPUs/drivers, falhas de compositing, WebGL lento ou indisponível | Média | Alto | Spike no M0 em GNOME e KDE (Wayland e X11), Intel/AMD/NVIDIA quando possível. Fallbacks documentados (variáveis do WebKit, renderizador DOM do xterm.js). Plano B: UI em Electron com o mesmo núcleo Rust |
| R2 | **Zoom de terminais** borrado ou com seleção desalinhada | Média | Alto | Estratégia da seção 6.2.2, validada no M0 com critério objetivo |
| R3 | **Desempenho com muitos terminais** (IPC, contextos WebGL, layout) | Média | Alto | Saída agrupada por quadro, LOD, parser headless no backend, limite de contextos WebGL; teste de carga no CI |
| R4 | **X11 vs Wayland** em atalhos globais, posição de janela, clipboard primário, IME | Média | Médio | Nada de atalhos globais nem posicionamento absoluto de janela no MVP; clipboard via APIs do Tauri; teste de IME (ibus/fcitx5) no M5 |
| R5 | **Empacotamento**: só `.deb` deixa de fora usuários de Fedora/Arch; dependências do WebKitGTK variam entre versões do Ubuntu/Debian | Alta | Médio | `.deb` é o único formato oficial (constituição). Testar instalação nas versões LTS atual e anterior do Ubuntu e no Debian estável. rpm, Flatpak (que exige `flatpak-spawn --host` para PTYs) e AppImage (problemas de FUSE 2 e libs Wayland embutidas) só por emenda |
| R6 | **Injeção de prompt em cadeia**: um agente lê conteúdo malicioso (repo, página, dependência) e passa instruções a outro agente pela conexão | Alta | Alto | Conexões só pelo usuário, direção explícita, cabeçalho de origem visível, limites de saltos e taxa, pausa fácil, log auditável. Documentar no threat model que conectar agentes amplia o alcance de uma injeção |
| R7 | **Injeção de teclas/escapes** entre PTYs (Ctrl-C, ESC, sequências OSC) | Média | Alto | Sanitização obrigatória (6.4.3); sem modo de teclas brutas no MVP; testes com payloads de escape |
| R8 | **Processo local do mesmo usuário** lê o token de outro nó (`/proc/<pid>/environ`) e se passa por ele | Baixa | Médio | Fora do modelo de ameaça (atacante com o mesmo UID já controla tudo). Documentado. Mitigação parcial: o token só vale enquanto o nó existe |
| R9 | **Agentes com acesso amplo** ao sistema (rodam com o UID do usuário, worktree não é sandbox) | Alta | Alto | Não-objetivo do MVP, declarado no README e no threat model. Sandbox por nó (bubblewrap/Landlock/podman) é item pós-MVP |
| R10 | **Segredos em workspaces** versionados no git | Média | Alto | `env_secret_keys` + keyring; aviso ao detectar padrões de chave em `env`; nunca serializar tokens |
| R16 | **Agentes não seguem o tutorial** ou usam a CLI de forma errada | Média | Médio | Tutorial curto e testado, erros que dizem o próximo passo, sugestão de comando, medição de sucesso com CLIs reais (spec 001, SC-002) |
| R17 | **Workspace malicioso**: um `.atlasoq/` vindo de um repositório clonado define comandos de agentes, hooks ou `learn.md` com instruções hostis que rodariam ao abrir | Média | Alto | Aprovação explícita antes da primeira execução e a cada mudança (`trust.json` com hash); hooks no MVP só executam `atlasoq learn`, sem scripts arbitrários |
| R18 | **Hook de aprendizado frágil**: CLIs mudam ou removem hooks, demoram a ficar prontas ou ignoram o prompt inicial | Média | Médio | Três modos com fallback (`hook` → `prompt` → `type`), estado visível no nó, teste com CLI falsa por modo e teste manual com CLIs reais a cada release |
| R11 | **Mudanças nas CLIs de agentes** (diálogos de confirmação de paste, modos de daemon) quebram a entrega | Alta | Médio | Entrega genérica (paste + Enter) sem lógica por CLI no MVP; suíte de testes manuais por CLI; perfis permitem ajustar o atraso do Enter |
| R12 | **Corrupção do workspace** por crash | Baixa | Alto | Escrita atômica, backups, teste de `kill -9` no CI |
| R13 | **Processos órfãos** se o app morrer | Média | Médio | `setsid` + `PR_SET_PDEATHSIG` + encerramento do grupo; teste automatizado |
| R14 | **Nome e marca**: "AtlasOQ" pode conflitar com marcas ou projetos existentes; vocabulário copiado de concorrentes | Baixa | Médio | Pesquisar o nome no GitHub, Flathub, crates.io, npm e INPI antes do primeiro release; vocabulário próprio (nó, nota, conexão, workspace) |
| R15 | **Licenças de dependências** incompatíveis (ex.: código BUSL ou AGPL de concorrentes) | Baixa | Alto | Nenhum código de concorrentes é lido para implementar; `cargo-deny` e verificação de licenças npm no CI |

---

## 8. Plano de milestones

Estimativas para uma pessoa em meio período; cada milestone termina com build reproduzível e
testes passando (constituição).

### M0 — Fundações e spikes (1–2 semanas)

- ADR-001 (stack), ADR-002 (modelo de conexão) e ADR-003 (diretório `.atlasoq/` e hooks).
- Levantar quais CLIs (Claude Code, Codex, Gemini CLI, OpenCode, Aider) suportam hooks de sessão
  ou prompt inicial, para definir o modo de cada perfil pronto.
- Repositório com workspace Cargo, app Tauri mínimo, UI com Vite, lint, formatação, CI
  (GitHub Actions) com `cargo test`, Vitest e verificação de licenças.
- **Spike 1 — terminal no WebKitGTK:** 1 xterm.js ligado a um PTY real via `portable-pty`.
- **Spike 2 — zoom:** 12 terminais num mundo com pan/zoom usando a estratégia 6.2.2.
- **Spike 3 — carga:** 8 terminais rodando um gerador de saída contínua.
- **Critérios de aprovação do Tauri** (testados em GNOME/Wayland, KDE/Wayland e uma sessão X11):
  texto nítido após o zoom parar; seleção com mouse correta em qualquer zoom; pan a ≥ 50 fps com
  12 terminais; UI responsiva (input < 50 ms) com 8 terminais em carga; sem tela branca.
  Se falhar: aplicar o plano B (seção 4.5) e revisar este SPEC.

### M1 — Camada PTY (1–2 semanas)

- `atlasoq-pty` completo: `SessionHost`, `LocalSessionHost`, parser headless, agrupamento de
  saída, ciclo de vida sem órfãos.
- Testes unitários e de integração da seção 6.1.

### M2 — Canvas e nós de terminal (2–3 semanas)

- Viewport com pan/zoom, grade, enquadrar, minimapa, seleção, mover/redimensionar, undo/redo.
- Nó de terminal completo (3.2), LOD, máquina de foco com testes.
- E2E básico com `tauri-driver`.

### M3 — Notas e persistência (1–2 semanas)

- `atlasoq-store`: formato, escrita atômica, backups, migrações, lista de workspaces recentes.
- Nó de nota com edição e leitura, arquivos `.md`, recarga via inotify.
- Reabertura de terminais (com a opção "perguntar").
- Teste de `kill -9` durante gravação.

### M4 — Conexões, CLI e tutorial (3–4 semanas)

- UI de conexões (criar, direção, pausar, apagar, histórico).
- `atlasoq-link`: socket, autenticação, ACL, sanitização, fila de entrega, proteção contra loops,
  log de auditoria.
- `atlasoq-cli` com os 8 comandos (`learn`, `list`, `send`, `read`, `ask`, `reply`, `note`, `help`), testes de contrato e mensagens de erro.
- Status `trabalhando/ocioso/encerrado` nos nós e hook de inicialização com os modos
  `hook`, `prompt`, `type` e `off`.
- Aprovação de workspace (`trust.json`).
- Tutorial `docs/agentes/tutorial.md`, guia `docs/guia-orquestracao.md` e o teste que executa os exemplos do tutorial.
- **Testes de integração obrigatórios** (constituição): criar conexão entre dois PTYs, trocar
  dados, pausar, recusar, loop guard disparando, payloads de escape neutralizados.
- Teste manual documentado com pelo menos duas CLIs de agentes reais.

### M5 — Endurecimento e release v0.1.0 (1–2 semanas)

- Pacote `.deb`, arquivo `.desktop` e ícone próprios; CLI em `/usr/bin/atlasoq`.
- Medição com CLIs de agentes reais (spec 001, SC-002).
- CI com smoke test em Wayland (compositor headless) e X11 (Xvfb).
- Teste de carga no CI, revisão de segurança das conexões, `docs/threat-model.md`.
- README com GIF de demonstração, arquitetura e limitações conhecidas.
- Pesquisa de nome concluída (R14). Release v0.1.0.

**Total estimado do MVP:** 9 a 15 semanas.

### Depois do MVP (ordem sugerida, cada item com revisão do SPEC)

1. **Daemon de sessões `atlasoqd`**: agentes sobrevivem a crash e reinício da UI.
2. Detecção de estado do agente (hooks das CLIs + heurística) e notificações D-Bus com ações.
3. Desenhos à mão livre e formas simples.
4. Worktree git por nó, com mapa de arquivos tocados por mais de um agente.
5. Outros formatos de pacote (rpm, Flatpak), por emenda.
6. Sandbox por nó (bubblewrap / Landlock / podman rootless).
7. Servidor MCP expondo notas e conexões aos agentes.

---

## 9. Decisões

Aprovadas em 2026-10-11:

1. **Nome:** AtlasOQ (README e constituição atualizados; o diretório local continua
   `AtlasOrchestrator` até a criação do repositório público).
2. **Stack:** Tauri v2 + xterm.js + portable-pty, com o plano B (Electron) e os critérios do M0.
3. **Foco do canvas:** orquestração de agentes de IA (agentes com nome e papel, status, conexões,
   notas como contexto compartilhado).
4. **CLI:** 8 comandos curtos e fáceis de lembrar (seção 6.4.2); tutorial para agentes via
   `atlasoq learn` (seção 6.4.7).
5. **Empacotamento:** apenas `.deb` por enquanto.
6. **Aprendizado automático:** sem botão; todo agente recebe o tutorial por hook de inicialização.
7. **Diretório `.atlasoq/`** na raiz do workspace, com parte versionável e parte local.
8. **Licença:** Apache-2.0.
9. **Daemon fora do MVP:** agentes encerram com o app e reiniciam ao reabrir; o daemon é o
   primeiro item pós-MVP (clarify 2026-10-11).
10. **Agentes iniciam sozinhos** ao reabrir um workspace aprovado (clarify 2026-10-11).
11. **Memória de longo prazo por agente** no MVP, escrita pelo próprio agente com `atlasoq note
    memory` e entregue no nascimento com limite de tamanho (diretriz do usuário, 2026-10-11).
12. **`ask` com resposta explícita:** `atlasoq reply`; `me` sai da CLI (clarify 2026-10-11).
13. **Sem desenhos no MVP** (clarify 2026-10-11).

Ainda pendentes (a tratar no `/speckit-clarify`):

1. **Idioma:** comandos em inglês; interface e tutorial em português no MVP.

## 10. Rastreabilidade com a constituição

| Princípio | Onde é atendido |
|---|---|
| I. Originalidade e licenciamento | Seções 4.6, 7 (R14, R15); vocabulário próprio |
| II. Linux primeiro | Seções 4, 7 (R1, R4, R5), critérios do M0, CI do M5 |
| III. Terminal como PTY real | Seções 3.2, 6.1 |
| IV. Comunicação com consentimento | Seções 3.4, 6.4 |
| V. Persistência local e legível | Seções 3.5, 6.5 |
| VI. Especificação antes da implementação | Este documento; seção 9 |
| VII. Escopo de MVP | Seções 3 e 3.8; desenhos e demais itens fora do MVP |
| VIII. CLI simples e memorável | Seção 6.4.2 |
| IX. Agentes aprendem pela própria CLI | Seções 3.6, 6.4.7 |
