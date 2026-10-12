import { describe, expect, it } from "vitest";
import {
  canvasKeysActive,
  DOUBLE_ESC_MS,
  type FocusEvent,
  initialFocus,
  reduceFocus,
} from "../../src/focus/focusMachine";

describe("máquina de foco", () => {
  it("canvas → selecionado → terminal", () => {
    let s = reduceFocus(initialFocus, { type: "select", ids: ["a1"] });
    expect(s.focus).toEqual({ kind: "selected", ids: ["a1"] });
    s = reduceFocus(s, { type: "enter_terminal", id: "a1" });
    expect(s.focus).toEqual({ kind: "terminal", id: "a1" });
    expect(canvasKeysActive(s.focus)).toBe(false);
  });

  it("um Esc fica no terminal; dois Esc seguidos saem", () => {
    let s = reduceFocus(initialFocus, { type: "enter_terminal", id: "a1" });
    s = reduceFocus(s, { type: "escape", at: 1000 });
    expect(s.focus.kind).toBe("terminal");
    s = reduceFocus(s, { type: "escape", at: 1000 + DOUBLE_ESC_MS - 1 });
    expect(s.focus).toEqual({ kind: "selected", ids: ["a1"] });
    expect(canvasKeysActive(s.focus)).toBe(true);
  });

  it("dois Esc espaçados não saem do terminal", () => {
    let s = reduceFocus(initialFocus, { type: "enter_terminal", id: "a1" });
    s = reduceFocus(s, { type: "escape", at: 1000 });
    s = reduceFocus(s, { type: "escape", at: 1000 + DOUBLE_ESC_MS + 50 });
    expect(s.focus.kind).toBe("terminal");
  });

  it("clique no fundo sempre volta ao canvas", () => {
    const events: FocusEvent[] = [
      { type: "enter_terminal", id: "a" },
      { type: "enter_note", id: "n" },
      { type: "select", ids: ["a", "b"] },
    ];
    for (const ev of events) {
      const s = reduceFocus(reduceFocus(initialFocus, ev), { type: "click_background" });
      expect(s.focus).toEqual({ kind: "canvas" });
    }
  });

  it("apagar o nó com foco devolve ao canvas", () => {
    let s = reduceFocus(initialFocus, { type: "enter_terminal", id: "a1" });
    s = reduceFocus(s, { type: "node_removed", id: "a1" });
    expect(s.focus).toEqual({ kind: "canvas" });
    s = reduceFocus(reduceFocus(initialFocus, { type: "select", ids: ["a", "b"] }), {
      type: "node_removed",
      id: "a",
    });
    expect(s.focus).toEqual({ kind: "selected", ids: ["b"] });
  });
});
