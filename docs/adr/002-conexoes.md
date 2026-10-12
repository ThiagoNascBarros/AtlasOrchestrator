# ADR-002: Modelo de conexão entre agentes

- **Status:** aceito (2026-10-11)
- **Contexto:** agentes precisam conversar sem abrir caminho para injeção de teclas, loops
  infinitos ou comunicação invisível (constituição, princípio IV).
- **Decisão:**
  - Conexões criadas só pelo usuário, com direção (`forward` →, `backward` ←, `both` ↔) e estado
    (`active`, `paused`, `auto_paused`). A direção define quem pode **iniciar**.
  - Agentes usam a CLI `atlasoq`, que fala com um broker no app por socket Unix (`0600`,
    `SO_PEERCRED`, token por nó). Nunca há pipe bruto entre PTYs.
  - Mensagens são sanitizadas (sem C0/C1 exceto `\n`/`\t`, sem ESC/CSI/OSC/DCS), limitadas a
    16 KiB, entregues com cabeçalho de origem, em bracketed paste quando suportado, uma por vez.
  - `ask` espera resposta explícita via `reply`; a resposta vai só para o `ask` que espera.
  - Proteção contra loops: 20 trocas sem intervenção humana, 6 mensagens/min (rajada 3) e
    detector de ping-pong pausam a conexão. Toda decisão vai para o log de auditoria.
- **Consequências:** comunicação previsível e auditável; agentes dependem de aprender a CLI
  (ADR-003).
- **Alternativas:** pipe de saída para entrada (inseguro, gera loops); leitura de tela como
  resposta do `ask` (ruidosa em TUIs; mantida só como fallback no tempo-limite).
