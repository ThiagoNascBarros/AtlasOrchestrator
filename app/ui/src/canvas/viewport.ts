// Geometria do canvas: conversão tela ↔ mundo, zoom centrado no cursor e pan.
import type { Rect, Viewport } from "../api/types";

export const ZOOM_MIN = 0.1;
export const ZOOM_MAX = 4;
/** Abaixo deste zoom os agentes viram cartões (FR-004). */
export const LOD_ZOOM = 0.35;

export interface Point {
  x: number;
  y: number;
}

export const clampZoom = (z: number) => Math.min(ZOOM_MAX, Math.max(ZOOM_MIN, z));

/** O viewport guarda o deslocamento em pixels de tela: tela = mundo × zoom + (x, y). */
export const worldToScreen = (v: Viewport, p: Point): Point => ({
  x: p.x * v.zoom + v.x,
  y: p.y * v.zoom + v.y,
});

export const screenToWorld = (v: Viewport, p: Point): Point => ({
  x: (p.x - v.x) / v.zoom,
  y: (p.y - v.y) / v.zoom,
});

/** Zoom para `zoom`, mantendo fixo o ponto de tela `anchor`. */
export function zoomAt(v: Viewport, anchor: Point, zoom: number): Viewport {
  const z = clampZoom(zoom);
  const world = screenToWorld(v, anchor);
  return { zoom: z, x: anchor.x - world.x * z, y: anchor.y - world.y * z };
}

/** Fator de zoom para um evento de roda (deltaY negativo aproxima). */
export const wheelZoomFactor = (deltaY: number) => Math.exp(-deltaY * 0.0015);

export const pan = (v: Viewport, dx: number, dy: number): Viewport => ({
  ...v,
  x: v.x + dx,
  y: v.y + dy,
});

export function boundsOf(rects: Rect[]): Rect | null {
  if (rects.length === 0) return null;
  const x1 = Math.min(...rects.map((r) => r.x));
  const y1 = Math.min(...rects.map((r) => r.y));
  const x2 = Math.max(...rects.map((r) => r.x + r.w));
  const y2 = Math.max(...rects.map((r) => r.y + r.h));
  return { x: x1, y: y1, w: x2 - x1, h: y2 - y1 };
}

/** Viewport que enquadra `rect` numa área de tela `screen` com margem. */
export function frame(rect: Rect, screen: { w: number; h: number }, margin = 48): Viewport {
  const zoom = clampZoom(
    Math.min((screen.w - 2 * margin) / rect.w, (screen.h - 2 * margin) / rect.h, 1.5),
  );
  return {
    zoom,
    x: (screen.w - rect.w * zoom) / 2 - rect.x * zoom,
    y: (screen.h - rect.h * zoom) / 2 - rect.y * zoom,
  };
}

export const intersects = (a: Rect, b: Rect) =>
  a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h && a.y + a.h > b.y;

/** Retângulo visível do mundo para uma área de tela. */
export const visibleWorld = (v: Viewport, screen: { w: number; h: number }): Rect => ({
  x: -v.x / v.zoom,
  y: -v.y / v.zoom,
  w: screen.w / v.zoom,
  h: screen.h / v.zoom,
});

export const snap = (value: number, grid: number) => Math.round(value / grid) * grid;

/** Normaliza um retângulo definido por dois cantos (seleção por arrasto). */
export const rectFromPoints = (a: Point, b: Point): Rect => ({
  x: Math.min(a.x, b.x),
  y: Math.min(a.y, b.y),
  w: Math.abs(a.x - b.x),
  h: Math.abs(a.y - b.y),
});
