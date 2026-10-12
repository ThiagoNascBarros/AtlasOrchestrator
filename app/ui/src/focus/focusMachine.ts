// Máquina de estados de foco (SPEC 6.2.3). Só um lugar recebe teclado por vez.

export type Focus =
  | { kind: "canvas" }
  | { kind: "selected"; ids: string[] }
  | { kind: "terminal"; id: string }
  | { kind: "note"; id: string };

export type FocusEvent =
  | { type: "click_background" }
  | { type: "select"; ids: string[] }
  | { type: "enter_terminal"; id: string }
  | { type: "enter_note"; id: string }
  | { type: "escape"; at: number }
  | { type: "node_removed"; id: string };

/** Intervalo máximo entre os dois Esc que devolvem o foco ao canvas. */
export const DOUBLE_ESC_MS = 400;

export interface FocusState {
  focus: Focus;
  lastEscAt: number | null;
}

export const initialFocus: FocusState = { focus: { kind: "canvas" }, lastEscAt: null };

export function reduceFocus(state: FocusState, ev: FocusEvent): FocusState {
  const { focus } = state;
  switch (ev.type) {
    case "click_background":
      return initialFocus;
    case "select":
      return {
        focus: ev.ids.length ? { kind: "selected", ids: ev.ids } : { kind: "canvas" },
        lastEscAt: null,
      };
    case "enter_terminal":
      return { focus: { kind: "terminal", id: ev.id }, lastEscAt: null };
    case "enter_note":
      return { focus: { kind: "note", id: ev.id }, lastEscAt: null };
    case "escape": {
      if (focus.kind === "terminal") {
        // Um Esc segue para o programa no terminal; dois Esc seguidos voltam ao canvas.
        if (state.lastEscAt !== null && ev.at - state.lastEscAt <= DOUBLE_ESC_MS) {
          return { focus: { kind: "selected", ids: [focus.id] }, lastEscAt: null };
        }
        return { focus, lastEscAt: ev.at };
      }
      if (focus.kind === "note")
        return { focus: { kind: "selected", ids: [focus.id] }, lastEscAt: null };
      return initialFocus;
    }
    case "node_removed": {
      if ((focus.kind === "terminal" || focus.kind === "note") && focus.id === ev.id)
        return initialFocus;
      if (focus.kind === "selected") {
        const ids = focus.ids.filter((id) => id !== ev.id);
        return {
          focus: ids.length ? { kind: "selected", ids } : { kind: "canvas" },
          lastEscAt: null,
        };
      }
      return state;
    }
  }
}

/** Se os atalhos do canvas devem responder ao teclado. */
export const canvasKeysActive = (f: Focus) => f.kind === "canvas" || f.kind === "selected";

export const selectedIds = (f: Focus): string[] =>
  f.kind === "selected" ? f.ids : f.kind === "terminal" || f.kind === "note" ? [f.id] : [];
