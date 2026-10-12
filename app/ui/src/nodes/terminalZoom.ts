// Estratégia de zoom de terminais (research.md R3): durante o gesto, escala por CSS; ao parar,
// fonte proporcional ao zoom e contra-escala, mantendo colunas e linhas.

export const BASE_FONT_SIZE = 14;
export const SETTLE_MS = 120;

/** Tamanho de fonte nítido para o zoom estabilizado. */
export const settledFontSize = (zoom: number) =>
  Math.max(1, Math.round(BASE_FONT_SIZE * zoom * 10) / 10);

/**
 * Escala CSS a aplicar no terminal, dentro do mundo já escalado por `zoom`, quando a fonte foi
 * calculada para `settledZoom`. Com zoom == settledZoom, a escala efetiva na tela é 1 (nítido).
 */
export const counterScale = (settledZoom: number) => 1 / settledZoom;
