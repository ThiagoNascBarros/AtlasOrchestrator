// Canvas infinito: pan, zoom centrado no cursor, seleção por retângulo, grade e nós.
import { useEffect, useRef, useState } from "react";
import { canvasKeysActive, selectedIds } from "../focus/focusMachine";
import { ConnectionLayer, ConnectionMenu } from "../connections/ConnectionLayer";
import { AgentNode } from "../nodes/AgentNode";
import { useUi } from "../store";
import { GRID } from "./NodeFrame";
import { Minimap } from "./Minimap";
import {
  boundsOf,
  frame,
  intersects,
  LOD_ZOOM,
  pan,
  type Point,
  rectFromPoints,
  screenToWorld,
  visibleWorld,
  wheelZoomFactor,
  zoomAt,
} from "./viewport";

type Gesture =
  | { kind: "pan"; last: Point }
  | { kind: "marquee"; start: Point; current: Point; additive: boolean };

export function frameAll(ids?: string[]) {
  const s = useUi.getState();
  const el = document.querySelector(".viewport");
  const rects = s.nodes.filter((n) => !ids || ids.includes(n.id)).map((n) => n.rect);
  const b = boundsOf(rects);
  if (!b || !el) return;
  s.setViewport(frame(b, { w: el.clientWidth, h: el.clientHeight }));
}

export function zoomBy(factor: number) {
  const s = useUi.getState();
  const el = document.querySelector(".viewport");
  if (!el) return;
  const center = { x: el.clientWidth / 2, y: el.clientHeight / 2 };
  s.setViewport(zoomAt(s.viewport, center, factor === 0 ? 1 : s.viewport.zoom * factor));
}

export function Viewport() {
  const ref = useRef<HTMLDivElement>(null);
  const nodes = useUi((s) => s.nodes);
  const viewport = useUi((s) => s.viewport);
  const grid = useUi((s) => s.grid);
  const [size, setSize] = useState({ w: 1, h: 1 });
  const [gesture, setGesture] = useState<Gesture | null>(null);
  const space = useRef(false);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const ro = new ResizeObserver(() => setSize({ w: el.clientWidth, h: el.clientHeight }));
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  // Roda: Ctrl (ou pinça) dá zoom; sem Ctrl, pan — exceto sobre o terminal com foco (rolagem).
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const onWheel = (e: WheelEvent) => {
      const s = useUi.getState();
      const rect = el.getBoundingClientRect();
      const anchor = { x: e.clientX - rect.left, y: e.clientY - rect.top };
      if (e.ctrlKey) {
        e.preventDefault();
        s.setViewport(zoomAt(s.viewport, anchor, s.viewport.zoom * wheelZoomFactor(e.deltaY)));
        return;
      }
      const f = s.focus.focus;
      const overNode = (e.target as HTMLElement).closest<HTMLElement>("[data-node-id]");
      if (f.kind === "terminal" && overNode?.dataset.nodeId === f.id) return;
      e.preventDefault();
      const dx = e.shiftKey ? -e.deltaY : -e.deltaX;
      const dy = e.shiftKey ? 0 : -e.deltaY;
      s.setViewport(pan(s.viewport, dx, dy));
    };
    el.addEventListener("wheel", onWheel, { passive: false });
    return () => el.removeEventListener("wheel", onWheel);
  }, []);

  // Atalhos do canvas (só quando o foco não está num terminal ou nota).
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const s = useUi.getState();
      if (e.type === "keyup") {
        if (e.code === "Space") space.current = false;
        return;
      }
      if (!canvasKeysActive(s.focus.focus)) return;
      if ((e.target as HTMLElement).closest("input, textarea, select, .dialog")) return;
      const ids = selectedIds(s.focus.focus);
      const ctrl = e.ctrlKey || e.metaKey;
      if (e.code === "Space") {
        space.current = true;
        e.preventDefault();
      } else if (ctrl && e.key.toLowerCase() === "z" && !e.shiftKey) {
        e.preventDefault();
        void s.undo();
      } else if (
        ctrl &&
        (e.key.toLowerCase() === "y" || (e.key.toLowerCase() === "z" && e.shiftKey))
      ) {
        e.preventDefault();
        void s.redo();
      } else if (ctrl && (e.key === "=" || e.key === "+")) {
        e.preventDefault();
        zoomBy(1.2);
      } else if (ctrl && e.key === "-") {
        e.preventDefault();
        zoomBy(1 / 1.2);
      } else if (ctrl && e.key === "0") {
        e.preventDefault();
        zoomBy(0);
      } else if (e.shiftKey && e.code === "Digit1") {
        frameAll();
      } else if (e.shiftKey && e.code === "Digit2" && ids.length) {
        frameAll(ids);
      } else if ((e.key === "Delete" || e.key === "Backspace") && ids.length) {
        e.preventDefault();
        void s.deleteNodes(ids);
      } else if (e.key === "Enter" && ids.length === 1) {
        e.preventDefault();
        s.dispatchFocus({ type: "enter_terminal", id: ids[0]! });
      } else if (e.key.toLowerCase() === "n" && !ctrl) {
        e.preventDefault();
        const el = ref.current;
        const c = el ? { x: el.clientWidth / 2, y: el.clientHeight / 2 } : { x: 0, y: 0 };
        s.openNewAgent(screenToWorld(s.viewport, c));
      } else if (e.key === "Escape") {
        s.dispatchFocus({ type: "escape", at: performance.now() });
      }
    };
    window.addEventListener("keydown", onKey);
    window.addEventListener("keyup", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("keyup", onKey);
    };
  }, []);

  const local = (e: React.PointerEvent): Point => {
    const r = ref.current!.getBoundingClientRect();
    return { x: e.clientX - r.left, y: e.clientY - r.top };
  };

  const onPointerDown = (e: React.PointerEvent) => {
    const p = local(e);
    if (e.button === 0) useUi.getState().selectConnection(null);
    if (e.button === 1 || (e.button === 0 && space.current)) {
      e.preventDefault();
      setGesture({ kind: "pan", last: p });
    } else if (e.button === 0) {
      setGesture({ kind: "marquee", start: p, current: p, additive: e.shiftKey });
    } else {
      return;
    }
    ref.current?.setPointerCapture(e.pointerId);
  };

  const onPointerMove = (e: React.PointerEvent) => {
    if (!gesture) return;
    const p = local(e);
    if (gesture.kind === "pan") {
      const s = useUi.getState();
      s.setViewport(pan(s.viewport, p.x - gesture.last.x, p.y - gesture.last.y));
      setGesture({ kind: "pan", last: p });
    } else {
      setGesture({ ...gesture, current: p });
    }
  };

  const onPointerUp = () => {
    const g = gesture;
    setGesture(null);
    if (!g || g.kind !== "marquee") return;
    const s = useUi.getState();
    const box = rectFromPoints(
      screenToWorld(s.viewport, g.start),
      screenToWorld(s.viewport, g.current),
    );
    if (box.w * s.viewport.zoom < 4 && box.h * s.viewport.zoom < 4) {
      if (!g.additive) s.dispatchFocus({ type: "click_background" });
      return;
    }
    const hit = s.nodes.filter((n) => intersects(n.rect, box)).map((n) => n.id);
    const prev = g.additive ? selectedIds(s.focus.focus) : [];
    s.dispatchFocus({ type: "select", ids: [...new Set([...prev, ...hit])] });
  };

  const visible = visibleWorld(viewport, size);
  const margin = 200 / viewport.zoom;
  const area = {
    x: visible.x - margin,
    y: visible.y - margin,
    w: visible.w + 2 * margin,
    h: visible.h + 2 * margin,
  };
  const gridPx = GRID * viewport.zoom;

  return (
    <div
      ref={ref}
      className={`viewport ${gesture?.kind === "pan" ? "panning" : ""}`}
      style={
        grid && gridPx >= 6
          ? {
              backgroundSize: `${gridPx}px ${gridPx}px`,
              backgroundPosition: `${viewport.x}px ${viewport.y}px`,
            }
          : { backgroundImage: "none" }
      }
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
    >
      <div
        className="world"
        style={{ transform: `translate(${viewport.x}px, ${viewport.y}px) scale(${viewport.zoom})` }}
      >
        <ConnectionLayer />
        {nodes.map((n) =>
          n.kind === "agent" ? (
            <AgentNode
              key={n.id}
              agent={n}
              lod={viewport.zoom < LOD_ZOOM || !intersects(n.rect, area)}
            />
          ) : null,
        )}
      </div>
      {gesture?.kind === "marquee" && (
        <div
          className="marquee"
          style={(() => {
            const r = rectFromPoints(gesture.start, gesture.current);
            return { left: r.x, top: r.y, width: r.w, height: r.h };
          })()}
        />
      )}
      <ConnectionMenu viewport={viewport} />
      <Minimap size={size} />
    </div>
  );
}
