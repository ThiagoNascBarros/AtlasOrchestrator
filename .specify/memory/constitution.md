# AtlasOrchestrator Constitution

AtlasOrchestrator é um canvas infinito para agentes de código no Linux. Cada terminal é um nó
posicionável, ao lado de notas e desenhos. Dois terminais podem ser conectados por uma linha para
que os agentes conversem entre si via PTY.

## Core Principles

### I. Originalidade e Licenciamento
O projeto MUST ser implementado do zero. Não MUST copiar nome, identidade visual (logo, paleta
proprietária, ícones, tipografia de marca), textos, estrutura de código ou assets de nenhum
aplicativo existente, seja ele proprietário ou open source com licença incompatível. Ao usar
dependências de terceiros, cada uma MUST ter licença compatível com a licença do projeto, e essa
verificação MUST ser registrada.
Rationale: o projeto é inspirado em um conceito, não em uma implementação. Copiar expõe o projeto a
problemas legais e descaracteriza a contribuição open source.

### II. Linux Primeiro
O alvo oficial MUST ser Linux, com suporte a sessões X11 e Wayland. Funcionalidades que dependam de
um único servidor de display MUST ser isoladas atrás de uma abstração e MUST ter fallback
documentado. Suporte a macOS ou Windows NÃO é requisito; qualquer decisão que os inviabilize MUST
ser aceita sem discussão adicional.
Rationale: foco em um único ambiente reduz a superfície de bugs e acelera o MVP.

### III. Terminal como PTY Real
Cada nó de terminal MUST ser backed por um pseudo-terminal real (PTY) do sistema operacional. O
projeto NÃO MUST interpretar ou simular a saída de shells ou agentes para fingir um terminal. A
renderização MUST preservar sequências ANSI, redimensionamento (SIGWINCH) e entrada de teclado.
Rationale: agentes de código dependem de comportamento de TTY (cores, prompts, TUIs). Um terminal
falso quebra essas ferramentas.

### IV. Comunicação Entre Agentes com Consentimento Explícito
Uma conexão entre dois terminais MUST ser criada por ação explícita do usuário e MUST ser visível
no canvas. Nenhum dado MUST ser injetado em um PTY sem que a conexão correspondente exista e esteja
ativa. Toda conexão MUST poder ser desfeita pelo usuário, e o encerramento MUST ser imediato. O
projeto MUST registrar localmente o que foi transmitido entre terminais, para auditoria. Nenhum
conteúdo de terminal MUST sair da máquina por padrão.
Rationale: um canal que transmite comandos entre agentes pode executar ações não autorizadas se for
implícito ou invisível. Tornar a conexão explícita, visível e auditável é a principal barreira de
segurança do produto.

### V. Persistência Local e Legível
O workspace (canvas, posições, notas, desenhos, conexões) MUST ser salvo localmente em um formato
de texto estruturado e versionável, preferencialmente JSON. O app MUST abrir e salvar sem rede. Os
dados MUST sobreviver a reinício e a crash. Uma escrita MUST ser atômica (arquivo temporário e
renomeação), para que um crash não corrompa o workspace.
Rationale: o usuário deve poder inspecionar, versionar com git e recuperar o próprio trabalho sem
depender de um serviço.

### VI. Especificação Antes da Implementação
Nenhuma funcionalidade MUST ser implementada sem um SPEC.md aprovado pelo mantenedor. O SPEC.md
MUST cobrir objetivos, não-objetivos, escopo do MVP, arquitetura, riscos e milestones. Mudanças de
escopo durante a implementação MUST voltar ao SPEC.md antes de serem codificadas.
Rationale: as decisões de arquitetura (stack, camada PTY, modelo de conexão) são caras de reverter.
Discuti-las antes do código evita retrabalho.

### VII. Escopo de MVP
O MVP MUST conter apenas: canvas infinito com pan e zoom, nós de terminal, notas em Markdown,
conexões entre terminais e persistência local. Funcionalidades adicionais (desenhos avançados,
colaboração em tempo real, plugins, sincronização na nuvem) MUST ficar fora do MVP, a menos que
sejam aprovadas por uma emenda MINOR.
Rationale: um escopo pequeno e bem testado vale mais que um escopo amplo e instável.

## Restrições de Plataforma e Segurança

- Dados de terminal (entrada, saída, histórico) MUST permanecer locais. Telemetria MUST ser
  desativada por padrão e MUST ser opt-in explícito, se existir.
- O canal entre PTYs MUST ser unidirecional ou bidirecional apenas conforme a conexão definida
  pelo usuário. Nenhum terminal MUST ter permissão para criar conexões sozinho.
- Processos filhos dos PTYs MUST rodar com as permissões do usuário. O app NÃO MUST elevar
  privilégios nem executar comandos com `sudo` por conta própria.
- Dependências MUST ser fixadas em versões específicas (lockfiles versionados). Atualizações de
  segurança MUST ser avaliadas antes de aplicadas.
- A stack tecnológica (ex.: Tauri, Electron, GTK4/Rust) NÃO é definida aqui. Ela MUST ser escolhida
  no SPEC.md e registrada com prós, contras e justificativa.

## Fluxo de Desenvolvimento e Qualidade

- Cada milestone MUST terminar com um build reproduzível e testes automatizados passando.
- A camada PTY e a persistência MUST ter testes automatizados. Testes de integração MUST cobrir
  a criação de conexão entre dois PTYs e a troca de dados entre eles.
- Toda mudança de comportamento que afete segurança (conexões, consentimento, transmissão de dados)
  MUST ter revisão explícita antes do merge.
- Pull requests MUST referenciar o item do SPEC.md ou do plano de milestones que implementam.
- Decisões de arquitetura MUST ser registradas (ADR curto) quando alterarem a stack, o modelo de
  conexão ou o formato de persistência.

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

**Revisão de conformidade**: cada PR MUST verificar a conformidade com os princípios I a VII. Um
PR que viole um princípio MUST ser bloqueado até que a violação seja corrigida ou que uma emenda
seja aprovada. Complexidade adicional (nova dependência, nova camada, nova abstração) MUST ser
justificada no próprio PR.

**Version**: 1.0.0 | **Ratified**: 2026-10-11 | **Last Amended**: 2026-10-11
