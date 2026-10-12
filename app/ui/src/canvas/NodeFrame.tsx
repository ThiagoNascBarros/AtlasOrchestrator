// Moldura genérica de nó: posição no mundo, seleção, arrastar pela barra de título e redimensionar.
import { type ReactNode, useRef } from "react";
import type { Rect } from "../api/types";
import { selectedIds } from "../focus/focusMachine";
import { type RectChange, useUi } from "../store";
import { snap } from "./viewport";

export const TITLE_H = 34;
export const GRID = 24;
const MIN_W = 240;
const MIN_H = 160;

interface Props {
  id: string;
  rect: Rect;
  title: ReactNode;
  children: ReactNode;
  className?: string;
  onTitleDoubleClick?: () => void;
}

type Drag =
  | { mode: "move"; startX: number; startY: number; before: RectChange[] }
  | { mode: "resize"; startX: number; startY: number; before: RectChange[] };

export function NodeFrame({ id, rect, title, children, className, onTitleDoubleClick }: Props) {
  const focus = useUi((s) => s.focus.focus);
  const drag = useRef<Drag | null>(null);
  const selected = selectedIds(focus).includes(id);

  const begin = (mode: Drag["mode"], e: React.PointerEvent) => {
    if (e.button !== 0) return;
    e.stopPropagation();
    const st = useUi.getState();
    let ids = selectedIds(st.focus.focus);
    if (!ids.includes(id)) {
      ids = e.shiftKey ? [...ids, id] : [id];
      st.dispatchFocus({ type: "select", ids });
    }
    if (mode === "resize") ids = [id];
    const before = st.nodes
      .filter((n) => ids.includes(n.id))
      .map((n) => ({ id: n.id, rect: n.rect }));
    drag.current = { mode, startX: e.clientX, startY: e.clientY, before };
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  };

  const move = (e: React.PointerEvent) => {
    const d = drag.current;
    if (!d) return;
    const { zoom } = useUi.getState().viewport;
    const grid = useUi.getState().grid && !e.altKey;
    const dx = (e.clientX - d.startX) / zoom;
    const dy = (e.clientY - d.startY) / zoom;
    const changes = d.before.map(({ id: nid, rect: r }) => {
      if (d.mode === "move") {
        const x = grid ? snap(r.x + dx, GRID) : r.x + dx;
        const y = grid ? snap(r.y + dy, GRID) : r.y + dy;
        return { id: nid, rect: { ...r, x, y } };
      }
      const w = Math.max(MIN_W, grid ? snap(r.w + dx, GRID) : r.w + dx);
      const h = Math.max(MIN_H, grid ? snap(r.h + dy, GRID) : r.h + dy);
      return { id: nid, rect: { ...r, w, h } };
    });
    useUi.getState().setRects(changes);
  };

  const end = () => {
    const d = drag.current;
    drag.current = null;
    if (!d) return;
    const nodes = useUi.getState().nodes;
    const after = d.before.map(({ id: nid }) => ({
      id: nid,
      rect: nodes.find((n) => n.id === nid)!.rect,
    }));
    const changed = after.some(
      (a, i) => JSON.stringify(a.rect) !== JSON.stringify(d.before[i]!.rect),
    );
    if (changed) useUi.getState().setRects(after, { before: d.before });
  };

  return (
    <div
      className={`node ${selected ? "node-selected" : ""} ${className ?? ""}`}
      style={{ left: rect.x, top: rect.y, width: rect.w, height: rect.h }}
      onPointerDown={(e) => {
        e.stopPropagation();
        if (!selected) useUi.getState().dispatchFocus({ type: "select", ids: [id] });
      }}
      data-node-id={id}
    >
      <div
        className="node-title"
        style={{ height: TITLE_H }}
        onPointerDown={(e) => begin("move", e)}
        onPointerMove={move}
        onPointerUp={end}
        onDoubleClick={onTitleDoubleClick}
      >
        {title}
      </div>
      <div className="node-body" style={{ height: rect.h - TITLE_H }}>
        {children}
      </div>
      <div
        className="node-resize"
        onPointerDown={(e) => begin("resize", e)}
        onPointerMove={move}
        onPointerUp={end}
        aria-label="Redimensionar"
      />
    </div>
  );
}
