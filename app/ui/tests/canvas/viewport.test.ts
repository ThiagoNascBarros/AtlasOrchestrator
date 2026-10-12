import { describe, expect, it } from "vitest";
import {
  clampZoom,
  frame,
  intersects,
  screenToWorld,
  worldToScreen,
  zoomAt,
  ZOOM_MAX,
  ZOOM_MIN,
} from "../../src/canvas/viewport";

const v0 = { x: 100, y: 50, zoom: 2 };

describe("viewport", () => {
  it("converte ida e volta entre tela e mundo", () => {
    const p = { x: 37, y: -12 };
    const back = screenToWorld(v0, worldToScreen(v0, p));
    expect(back.x).toBeCloseTo(p.x);
    expect(back.y).toBeCloseTo(p.y);
  });

  it("limita o zoom entre 10% e 400%", () => {
    expect(clampZoom(0.01)).toBe(ZOOM_MIN);
    expect(clampZoom(10)).toBe(ZOOM_MAX);
    expect(ZOOM_MIN).toBe(0.1);
    expect(ZOOM_MAX).toBe(4);
  });

  it("zoom mantém o ponto sob o cursor", () => {
    const cursor = { x: 400, y: 300 };
    const before = screenToWorld(v0, cursor);
    const v1 = zoomAt(v0, cursor, 0.5);
    const after = screenToWorld(v1, cursor);
    expect(v1.zoom).toBe(0.5);
    expect(after.x).toBeCloseTo(before.x);
    expect(after.y).toBeCloseTo(before.y);
  });

  it("enquadrar cabe na tela", () => {
    const rect = { x: -500, y: 200, w: 1000, h: 400 };
    const v = frame(rect, { w: 800, h: 600 });
    const tl = worldToScreen(v, { x: rect.x, y: rect.y });
    const br = worldToScreen(v, { x: rect.x + rect.w, y: rect.y + rect.h });
    expect(tl.x).toBeGreaterThanOrEqual(0);
    expect(br.x).toBeLessThanOrEqual(800);
    expect(tl.y).toBeGreaterThanOrEqual(0);
    expect(br.y).toBeLessThanOrEqual(600);
  });

  it("interseção de retângulos", () => {
    expect(intersects({ x: 0, y: 0, w: 10, h: 10 }, { x: 5, y: 5, w: 10, h: 10 })).toBe(true);
    expect(intersects({ x: 0, y: 0, w: 10, h: 10 }, { x: 20, y: 0, w: 5, h: 5 })).toBe(false);
  });
});
