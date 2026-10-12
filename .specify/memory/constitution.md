<!--
Sync Impact Report
==================
Versão: 1.0.0 → 1.1.0 (MINOR: novos princípios VIII e IX, decisões de stack e empacotamento,
expansão do escopo do MVP para a orquestração de agentes)

Princípios alterados:
- Título e preâmbulo: "AtlasOrchestrator" → "AtlasOQ"; o canvas passa a ser descrito como canvas
  de orquestração de agentes de IA.
- IV. Comunicação Entre Agentes com Consentimento Explícito: a comunicação passa a ocorrer por
  mensagens mediadas pela CLI `atlasoq`; pipe bruto entre PTYs fica proibido.
- VII. Escopo de MVP: inclui a CLI `atlasoq` e o tutorial para agentes; desenhos ficam fora do MVP.

Princípios adicionados:
- VIII. CLI Simples e Memorável
- IX. Agentes Aprendem pela Própria CLI

Seções alteradas:
- Restrições de Plataforma e Segurança: stack definida (Tauri v2 + xterm.js + portable-pty);
  empacotamento oficial apenas em .deb.

Templates:
- .specify/templates/plan-template.md ✅ sem alteração necessária (Constitution Check é genérico)
- .specify/templates/spec-template.md ✅ sem alteração necessária
- .specify/templates/tasks-template.md ✅ sem alteração necessária

Documentos dependentes:
- README.md ✅ atualizado (nome, stack, empacotamento)
- SPEC.md ✅ atualizado (v0.2.0)
- specs/001-mvp-orquestracao-agentes/spec.md ✅ criado

TODOs pendentes:
- TODO(LICENSE): licença do projeto ainda não decidida (recomendação: Apache-2.0).
-->

# AtlasOQ Constitution

AtlasOQ é um canvas infinito para orquestrar agentes de IA de código no Linux. Cada agente roda
num terminal real, representado por um nó posicionável no canvas, ao lado de notas. O usuário
liga dois agentes com uma linha, e só por essa linha um agente consegue mandar mensagens ao outro,
usando a CLI `atlasoq`.

## Core Principles

### I. Originalidade e Licenciamento
O projeto MUST ser implementado do zero. Não MUST copiar nome, identidade visual (logo, paleta
proprietária, ícones, tipografia de marca), textos, vocabulário de produto, estrutura de código
ou assets de nenhum aplicativo existente, seja ele proprietário ou open source com licença
incompatível. Ao usar dependências de terceiros, cada uma MUST ter licença compatível com a
licença do projeto, e essa verificação MUST ser registrada.
Rationale: o projeto é inspirado em um conceito, não em uma implementação. Copiar expõe o projeto a
problemas legais e descaracteriza a contribuição open source.

### II. Linux Primeiro
O alvo oficial MUST ser Linux, com suporte a sessões X11 e Wayland. Funcionalidades que dependam de
um único servidor de display MUST ser isoladas atrás de uma abstração e MUST ter fallback
documentado. Suporte a macOS ou Windows NÃO é requisito; qualquer decisão que os inviabilize MUST
ser aceita sem discussão adicional.
Rationale: foco em um único ambiente reduz a superfície de bugs e acelera o MVP.

### III. Terminal como PTY Real
Cada agente no canvas MUST rodar num pseudo-terminal real (PTY) do sistema operacional. O projeto
NÃO MUST interpretar ou simular a saída de shells ou agentes para fingir um terminal. A
renderização MUST preservar sequências ANSI, redimensionamento (SIGWINCH) e entrada de teclado.
O AtlasOQ hospeda CLIs de agentes de terceiros; ele NÃO MUST chamar modelos de IA por conta
própria.
Rationale: agentes de código dependem de comportamento de TTY (cores, prompts, TUIs). Um terminal
falso quebra essas ferramentas.

### IV. Comunicação Entre Agentes com Consentimento Explícito
Uma conexão entre dois agentes MUST ser criada por ação explícita do usuário e MUST ser visível
no canvas, com sua direção. Nenhum dado MUST ser injetado em um PTY sem que a conexão
correspondente exista, esteja ativa e permita aquela direção. A comunicação MUST ocorrer por
mensagens mediadas pelo app (via CLI `atlasoq`), sanitizadas antes da entrega; pipe bruto de bytes
entre PTYs e envio de teclas de controle NÃO são permitidos. Toda conexão MUST poder ser pausada ou
desfeita pelo usuário, com efeito imediato. Trocas automáticas entre agentes MUST ter limites
contra loops. O projeto MUST registrar localmente o que foi transmitido, para auditoria. Nenhum
conteúdo de terminal MUST sair da máquina por padrão.
Rationale: um canal que transmite comandos entre agentes pode executar ações não autorizadas se for
implícito, invisível ou ilimitado. Tornar a conexão explícita, visível, limitada e auditável é a
principal barreira de segurança do produto.

### V. Persistência Local e Legível
O workspace (canvas, posições, notas, conexões) MUST ser salvo localmente em um formato de texto
estruturado e versionável (JSON para o workspace, Markdown para as notas). O app MUST abrir e
salvar sem rede. Os dados MUST sobreviver a reinício e a crash. Uma escrita MUST ser atômica
(arquivo temporário e renomeação), para que um crash não corrompa o workspace. Segredos NÃO MUST
ser serializados no workspace.
Rationale: o usuário deve poder inspecionar, versionar com git e recuperar o próprio trabalho sem
depender de um serviço.

### VI. Especificação Antes da Implementação
Nenhuma funcionalidade MUST ser implementada sem um SPEC.md aprovado pelo mantenedor. O SPEC.md
MUST cobrir objetivos, não-objetivos, escopo do MVP, arquitetura, riscos e milestones. Cada
funcionalidade MUST ter uma spec em `specs/NNN-nome/` antes do código. Mudanças de escopo durante a
implementação MUST voltar ao SPEC.md antes de serem codificadas.
Rationale: as decisões de arquitetura (stack, camada PTY, modelo de conexão) são caras de reverter.
Discuti-las antes do código evita retrabalho.

### VII. Escopo de MVP
O MVP MUST conter apenas: canvas infinito com pan e zoom, agentes em nós de terminal, notas em
Markdown, conexões entre agentes, a CLI `atlasoq` com seu tutorial para agentes, e persistência
local. Funcionalidades adicionais (desenhos, colaboração em tempo real, plugins, sincronização na
nuvem, sandbox de agentes, daemon de sessões) MUST ficar fora do MVP, a menos que sejam aprovadas
por uma emenda MINOR.
Rationale: um escopo pequeno e bem testado vale mais que um escopo amplo e instável.

### VIII. CLI Simples e Memorável
A CLI `atlasoq` é a interface dos agentes com o canvas e MUST ser fácil de lembrar:
- comandos MUST ser uma palavra curta em inglês, no formato `atlasoq <verbo> [alvo] [texto]`;
- o MVP MUST ter no máximo 8 comandos; cada comando novo MUST ser justificado no SPEC.md;
- agentes MUST ser referenciados pelo nome visível no canvas, nunca por ids internos;
- o caso comum NÃO MUST exigir flags; flags são só para casos avançados;
- toda mensagem de erro MUST dizer o que fazer em seguida (ex.: "não existe conexão de
  `planner` para `coder`; peça ao usuário para ligar os dois no canvas");
- a saída MUST ser texto legível por padrão, com `--json` opcional para uso por scripts;
- nomes e semântica de comandos publicados NÃO MUST mudar sem emenda MAJOR.
Rationale: quem usa a CLI são agentes de IA guiados por instruções curtas. Uma CLI pequena e
previsível reduz erros, tokens gastos e a necessidade de intervenção humana.

### IX. Agentes Aprendem pela Própria CLI
O projeto MUST manter um tutorial para agentes, acessível pela própria CLI (`atlasoq learn`) e
publicado em `docs/`. O tutorial MUST ser curto (cabe numa tela e meia de terminal), escrito para
ser lido por um agente de IA, com exemplos copiáveis de cada comando. O tutorial MUST ser
atualizado no mesmo PR que alterar a CLI, e MUST existir um teste automatizado que executa os
exemplos do tutorial contra o app. O usuário MUST conseguir enviar o tutorial a um agente com uma
ação explícita no canvas; o app NÃO MUST gravar instruções no repositório do usuário por conta
própria.
Rationale: a orquestração só funciona se os agentes souberem usar a CLI sem depender de o usuário
explicar tudo a cada sessão. Um tutorial testado não envelhece em silêncio.

## Restrições de Plataforma e Segurança

- Dados de terminal (entrada, saída, histórico) MUST permanecer locais. Telemetria MUST ser
  desativada por padrão e MUST ser opt-in explícito, se existir.
- O canal entre agentes MUST ser unidirecional ou bidirecional apenas conforme a conexão definida
  pelo usuário. Nenhum agente MUST ter permissão para criar, alterar ou remover conexões.
- O socket local da CLI MUST ser acessível só pelo usuário dono do app e MUST autenticar o nó de
  origem de cada requisição.
- Processos filhos dos PTYs MUST rodar com as permissões do usuário. O app NÃO MUST elevar
  privilégios nem executar comandos com `sudo` por conta própria.
- Dependências MUST ser fixadas em versões específicas (lockfiles versionados). Atualizações de
  segurança MUST ser avaliadas antes de aplicadas.
- **Stack definida:** Tauri v2 (UI em TypeScript sobre WebKitGTK), xterm.js para renderizar
  terminais e núcleo em Rust com `portable-pty`. O núcleo Rust NÃO MUST depender do Tauri, para
  permitir trocar a camada de UI. Mudar a stack MUST passar por ADR e emenda.
- **Empacotamento:** o único formato oficial de instalação é `.deb`. Outros formatos (rpm, Flatpak,
  AppImage) MUST passar por emenda antes de serem distribuídos.

## Fluxo de Desenvolvimento e Qualidade

- O trabalho MUST acontecer em branches `ATQ-NNN` e as mensagens de commit MUST começar com o
  identificador da branch (ex.: `ATQ-001 ...`).
- Cada milestone MUST terminar com um build reproduzível e testes automatizados passando.
- A camada PTY e a persistência MUST ter testes automatizados. Testes de integração MUST cobrir
  a criação de conexão entre dois PTYs e a troca de dados entre eles.
- A CLI `atlasoq` MUST ter testes de contrato para cada comando, incluindo as mensagens de erro, e
  o teste dos exemplos do tutorial (princípio IX).
- Toda mudança de comportamento que afete segurança (conexões, consentimento, transmissão de dados)
  MUST ter revisão explícita antes do merge.
- Pull requests MUST referenciar o item do SPEC.md, da spec da funcionalidade ou do plano de
  milestones que implementam.
- Decisões de arquitetura MUST ser registradas (ADR curto) quando alterarem a stack, o modelo de
  conexão, a CLI ou o formato de persistência.

## Governance

Esta constituição tem precedência sobre as demais práticas do projeto. Em caso de conflito, a
constituição prevalece e o documento conflitante MUST ser corrigido.

**Procedimento de emenda**: toda alteração MUST ser proposta por escrito, com justificativa e
impacto nos princípios existentes. Ela MUST ser aprovada pelo mantenedor do projeto antes de entrar
em vigor. Após a aprovação, o número de versão, a data de última alteração e o Sync Impact Report
MUST ser atualizados.

**Política de versionamento** (semântico):
- MAJOR: remoção ou redefinição incompatível de um princípio ou de uma restrição.
- MINOR: novo princípio ou seção, ou expansão material de uma regra existente.
- PATCH: esclarecimentos, correções de texto e refinamentos sem efeito semântico.

**Revisão de conformidade**: cada PR MUST verificar a conformidade com os princípios I a IX. Um
PR que viole um princípio MUST ser bloqueado até que a violação seja corrigida ou que uma emenda
seja aprovada. Complexidade adicional (nova dependência, nova camada, nova abstração) MUST ser
justificada no próprio PR.

**Version**: 1.1.0 | **Ratified**: 2026-10-11 | **Last Amended**: 2026-10-11
