# Research: MVP do AtlasOQ

**Feature**: [spec.md](spec.md) · **Plan**: [plan.md](plan.md) · **Data**: 2026-10-11

Cada item segue o formato Decisão / Justificativa / Alternativas. Itens marcados **[validar no M0]**
dependem de teste prático antes de virar código definitivo.

---

## R1. Stack da aplicação

- **Decisão:** Tauri v2 (Rust + WebView WebKitGTK 4.1) com UI em TypeScript + React 18 + Zustand,
  build com Vite; núcleo em crates Rust sem dependência de Tauri.
- **Justificativa:** aprovada no SPEC.md (seção 4.5) e na constituição v1.3.0. Rust dá controle de
  processos, sinais e sockets Unix; a UI web resolve canvas, terminais (xterm.js) e Markdown com
  pouco código.
- **Alternativas:** Electron (plano B se o spike do M0 reprovar o WebKitGTK); GTK4 + VTE (zoom de
  terminais e Markdown difíceis).

## R2. Emulação de terminal e PTY

- **Decisão:** `portable-pty` no backend; `@xterm/xterm` com addons `webgl`, `fit`, `search`,
  `web-links`, `unicode11` na UI; parser VT headless no backend com o crate `vt100`.
- **Justificativa:** o parser headless é necessário para `atlasoq read`, para o fallback do `ask`,
  para saber se o destino ativou bracketed paste (modo 2004) e para remontar o terminal a partir de
  snapshot quando o nó sai do LOD. `vt100` é pequeno e expõe tela, scrollback e modos.
- **Alternativas:** `alacritty_terminal` (mais completo, API mais pesada e menos estável como
  biblioteca); reconstruir o estado só na UI (impede `read` com a UI fechada ou nós fora da tela).
- **Versões:** fixadas no M0 nas últimas estáveis disponíveis e registradas em `Cargo.lock` e
  `pnpm-lock.yaml`.

## R3. Zoom de terminais no canvas **[validar no M0]**

- **Decisão:** durante o gesto, terminais escalados por CSS junto com o mundo; ao parar (~120 ms),
  cada terminal visível é re-renderizado fora do container escalado com `fontSize = base × zoom`,
  mantendo colunas e linhas (sem SIGWINCH). Abaixo de 35% de zoom ou fora da tela, o xterm.js é
  desmontado e o nó vira cartão (nome, papel, status, últimas linhas).
- **Justificativa:** `transform: scale()` em xterm.js borra o texto e desalinha mouse e seleção;
  trocar a fonte mantém nitidez e coordenadas corretas. O LOD limita contextos WebGL.
- **Critério de aprovação (M0):** texto nítido após o zoom; seleção correta em 25%, 50%, 100%, 200%;
  ≥ 50 fps de pan com 12 terminais; sem tela branca em GNOME/Wayland, KDE/Wayland e X11.
- **Alternativas:** só CSS scale (borrado); redimensionar o PTY a cada zoom (quebra TUIs e muda o
  layout do agente).

## R4. Aprendizado automático: modo por CLI de agente

Restrição: o app não pode alterar arquivos fora de `.atlasoq/` (constituição, princípio IX).

| CLI | Modo | Mecanismo | Status |
|---|---|---|---|
| Claude Code | `hook` | `claude --settings .atlasoq/agents/<nome>/claude-settings.json`; o arquivo define um hook `SessionStart` (matchers `startup`, `resume`, `clear`, `compact`) que roda `atlasoq learn --hook`. A saída em texto do SessionStart entra no contexto do agente | Confirmado na documentação oficial de hooks (saída do SessionStart vira contexto; limite de 10.000 caracteres). Carregar hooks pelo `--settings` **[validar no M0]** |
| Codex CLI | `prompt` | `codex "<texto inicial>"` | Hooks do Codex são experimentais e lidos de `~/.codex/` ou `<repo>/.codex/`, fora do `.atlasoq/` → não usados |
| Gemini CLI | `prompt` | `gemini --prompt-interactive "<texto inicial>"` | Hooks existem, mas são configurados em `~/.gemini/` ou `<repo>/.gemini/` → não usados. Flag **[validar no M0]** |
| OpenCode | `prompt` | `opencode --prompt "<texto inicial>"` | Documentação recente lista `--prompt` para o TUI; versões antigas tratavam como não interativo **[validar no M0]** |
| Aider | `type` | Envio após a primeira ociosidade | `--message` executa e sai, não serve |
| Shell | `off` | — | — |

- **Decisão adicional (importante):** nos modos `prompt` e `type`, o texto inicial **contém o
  próprio tutorial e a memória**, em vez de pedir ao agente que rode `atlasoq learn`. Pedir para
  rodar um comando faria a CLI do agente pedir permissão ao usuário (Claude Code e Codex pedem
  aprovação para comandos de shell), o que seria um passo manual.
  O FR-028a da spec foi ajustado para refletir isso.
- **Redação do conteúdo:** escrito como afirmações factuais ("Você é `coder` no AtlasOQ. Para
  falar com outro agente, use ..."), não como ordens de sistema. A documentação do Claude Code
  avisa que texto imperativo injetado como contexto pode acionar defesas contra injeção de prompt.
- **Tamanho:** tutorial (≤ 60 linhas, ~3 KB) + memória (≤ 2 KB) cabe no limite do SessionStart e em
  argumentos de linha de comando.
- **Alternativas:** escrever `CLAUDE.md`/`AGENTS.md` no projeto (proibido pela constituição); apontar
  `CODEX_HOME`/configuração global para `.atlasoq/` (quebra a autenticação do usuário nas CLIs).

## R5. Permissão das CLIs para executar `atlasoq`

- **Problema:** mesmo aprendendo, o agente pede permissão ao usuário para rodar `atlasoq send/ask`,
  o que reintroduz passos manuais.
- **Decisão:** no modo `hook` do Claude Code, o mesmo `claude-settings.json` gerado inclui a regra de
  permissão que libera apenas comandos `atlasoq` (`permissions.allow: ["Bash(atlasoq:*)"]`). Para
  as outras CLIs, o perfil pronto documenta como liberar `atlasoq` nas configurações de aprovação
  de cada uma, e o guia `docs/guia-orquestracao.md` explica. O app não altera a configuração
  global das CLIs.
- **Justificativa:** libera só a CLI do AtlasOQ, que por sua vez é limitada pelas conexões, ACL e
  proteção contra loops; não amplia permissões de shell em geral.
- **Alternativas:** modo "sem aprovação" das CLIs (amplo demais); não fazer nada (passos manuais
  constantes). **[validar no M0]** a sintaxe exata da regra.

## R6. Detecção de "pronto" e "ocioso"

- **Decisão:** heurística por atividade de saída: `trabalhando` quando houve saída nos últimos 2 s;
  `ocioso` após 3 s sem saída (configurável); "pronto" para o modo `type` = primeira transição para
  `ocioso` depois do início, com tempo-limite de 60 s. Bytes de spinners contam como atividade.
- **Justificativa:** genérica, sem dependência de CLI. O `ask` não depende mais dela (usa `reply`).
- **Alternativas:** hooks de cada CLI (post-MVP); análise do conteúdo da tela (frágil).

## R7. Protocolo CLI ↔ app

- **Decisão:** socket Unix `SOCK_STREAM` em `$XDG_RUNTIME_DIR/atlasoq/<instância>.sock` (diretório
  `0700`, socket `0600`), JSON delimitado por linha, versão `v: 1`. Autenticação: `SO_PEERCRED`
  (mesmo UID) + `ATLASOQ_NODE_TOKEN` (256 bits, por spawn, só em memória). Requisição/resposta
  síncrona; `ask` mantém a conexão aberta até `reply`, tempo-limite ou cancelamento.
- **Justificativa:** simples, local, sem portas de rede, fácil de testar.
- **Alternativas:** D-Bus (mais complexo, sem ganho); HTTP em localhost (porta exposta a outros
  usuários e processos).
- **Contrato:** [contracts/socket-protocol.md](contracts/socket-protocol.md).

## R8. Entrega de mensagens no PTY

- **Decisão:** sanitizar (remover C0/C1 exceto `\n` e `\t`, remover sequências ESC/CSI/OSC/DCS,
  normalizar `\r\n`), limite de 16 KiB, cabeçalho de origem, bracketed paste se o destino ativou o
  modo 2004, `\r` após 150 ms, fila serial por destino.
- **Justificativa:** impede injeção de teclas e escapes (FR-015, SC-006) e evita mensagens coladas.
- **Alternativas:** escrever bytes crus (inseguro); simular digitação caractere a caractere (lento,
  sujeito a autocomplete das TUIs).

## R9. Persistência e escrita atômica

- **Decisão:** `serde_json` com `schema` versionado; escrita via arquivo temporário no mesmo
  diretório + `fsync` + `rename` + `fsync` do diretório; debounce de 500 ms; 10 backups em
  `.atlasoq/backups/`; notas e memórias em Markdown, uma por arquivo; observação de mudanças com
  `notify` (inotify).
- **Alternativas:** SQLite (não é legível nem versionável com diff simples, contraria o princípio V).

## R10. Memória de longo prazo

- **Decisão:** `.atlasoq/memory/<nome>.md`, formato de linhas `- AAAA-MM-DD HH:MM <tipo>: <texto>`,
  tipos `feito`, `decisão`, `pendente`; até 300 caracteres por anotação; entrega no nascimento das
  últimas 20 anotações limitadas a 2 KB, com aviso de anotações antigas. Escrita só pelo próprio
  agente (`atlasoq note memory`); detecção simples de segredos por padrões (prefixos de chaves
  conhecidas, `password=`, blocos PEM, alta entropia em tokens longos).
- **Justificativa:** custo de tokens fixo e baixo (SC-012), sem LLM no app (princípio III).
- **Alternativas:** compactação automática pelo agente (post-MVP); banco vetorial (desproporcional).

## R11. Confiança no workspace

- **Decisão:** hash SHA-256 do conjunto "executável" do workspace (comandos, args, env não secreto
  e modo de hook de cada agente; perfis do workspace; `learn.md`), guardado em
  `~/.local/state/atlasoq/trust.json` por caminho canônico da raiz. Workspace criado pelo app nesta
  máquina já nasce aprovado. Divergência → diálogo listando o que será executado.
- **Alternativas:** confiar sempre (risco R17); pedir sempre (contraria "agentes iniciam sozinhos").

## R12. Processos sem órfãos

- **Decisão:** cada agente em nova sessão (`setsid`), `prctl(PR_SET_PDEATHSIG, SIGHUP)` no filho
  antes do `exec`; ao fechar, SIGHUP ao grupo, 3 s, SIGKILL. Lock exclusivo (`flock`) em
  `.atlasoq/.lock` impede abrir o mesmo workspace duas vezes.
- **Ressalva:** `PR_SET_PDEATHSIG` dispara na morte da *thread* que criou o filho; o spawn deve
  ocorrer numa thread dedicada e de vida longa. **[validar no M1]** com teste de `kill -9`.

## R13. Distribuição `.deb` e CLI no PATH

- **Decisão:** bundler `.deb` do Tauri. Binário da UI: `atlasoq-desktop`; CLI `atlasoq` como binário
  separado instalado em `/usr/bin/atlasoq` (via `externalBin` do Tauri ou arquivo extra no pacote).
  Dependências declaradas: `libwebkit2gtk-4.1-0`, `libgtk-3-0`. Dentro dos agentes, o app também
  coloca o diretório da CLI no início do `PATH` (útil no modo de desenvolvimento).
- **Alvos de teste:** Ubuntu 26.04 LTS e 24.04 LTS, Debian 13 (estável).
- **Alternativas:** binário único multi-chamada (inicialização mais lenta da CLI, que é chamada
  com frequência pelos agentes). **[validar no M0]** o caminho de instalação do `externalBin`.

## R14. Testes

- **Decisão:** `cargo test` (unidade e integração dos crates, incluindo dois PTYs reais conectados);
  testes de contrato da CLI com `assert_cmd` + snapshots (`insta`) de saída e códigos; Vitest para
  a UI (store, máquina de foco, geometria do canvas); E2E com WebdriverIO + `tauri-driver` em
  Xvfb e compositor Wayland headless (`weston --backend=headless`); teste do tutorial que extrai e
  executa cada bloco de exemplo; CLIs falsas (scripts) para os modos `hook`, `prompt` e `type`.
- **Alternativas:** Playwright (não dirige o WebKitGTK do Tauri diretamente).

## R15. Idioma

- **Decisão:** comandos e flags em inglês; interface, mensagens de erro da CLI e tutorial em
  português no MVP, com textos centralizados (catálogo único) para tradução futura.
- **Justificativa:** registrado como premissa na spec; não bloqueia a implementação.

## Fontes consultadas

- Claude Code — Hooks reference: https://code.claude.com/docs/en/hooks
- Gemini CLI — Hooks reference: https://geminicli.com/docs/hooks/reference/
- Codex CLI hooks (artigos de terceiros, março/2026): https://codex.danielvaughan.com/2026/03/26/codex-cli-hooks-deep-dive
- OpenCode — CLI e plugins: https://opencode.ai/docs/plugins/ · https://docs.opencode.ai/docs/cli
