// Linhas das conexões entre agentes (com direção, estado e animação de mensagem) e o menu de
// uma conexão. A camada fica dentro do "mundo", então acompanha pan e zoom.
import type { Connection, Rect } from "../api/types";
import { t } from "../i18n/pt-BR";
import { useUi } from "../store";
import { type Point, worldToScreen } from "../canvas/viewport";
import type { Viewport } from "../api/types";

const center = (r: Rect): Point => ({ x: r.x + r.w / 2, y: r.y + r.h / 2 });

/** Ponto onde a reta do centro de `r` até `toward` cruza a borda de `r`. */
export function edgePoint(r: Rect, toward: Point): Point {
  const c = center(r);
  const dx = toward.x - c.x;
  const dy = toward.y - c.y;
  if (dx === 0 && dy === 0) return c;
  const k = Math.min(r.w / 2 / Math.abs(dx || 1e-9), r.h / 2 / Math.abs(dy || 1e-9));
  return { x: c.x + dx * k, y: c.y + dy * k };
}

export function linePoints(a: Rect, b: Rect): [Point, Point] {
  return [edgePoint(a, center(b)), edgePoint(b, center(a))];
}

const stateClass = (c: Connection) =>
  c.state === "active" ? "conn-active" : c.state === "paused" ? "conn-paused" : "conn-auto";

export function ConnectionLayer() {
  const nodes = useUi((s) => s.nodes);
  const connections = useUi((s) => s.connections);
  const pulses = useUi((s) => s.pulses);
  const draft = useUi((s) => s.draft);
  const menuConn = useUi((s) => s.menuConn);
  const rects = new Map(nodes.map((n) => [n.id, n.rect]));

  return (
    <svg className="conn-layer" width={1} height={1}>
      <defs>
        {["active", "paused", "auto"].map((k) => (
          <marker
            key={k}
            id={`arrow-${k}`}
            viewBox="0 0 10 10"
            refX="9"
            refY="5"
            markerWidth="14"
            markerHeight="14"
            markerUnits="userSpaceOnUse"
            orient="auto-start-reverse"
          >
            <path d="M0 0 L10 5 L0 10 z" className={`conn-arrow conn-arrow-${k}`} />
          </marker>
        ))}
      </defs>
      {connections.map((c) => {
        const a = rects.get(c.from);
        const b = rects.get(c.to);
        if (!a || !b) return null;
        const [p1, p2] = linePoints(a, b);
        const d = `M${p1.x} ${p1.y} L${p2.x} ${p2.y}`;
        const kind = c.state === "active" ? "active" : c.state === "paused" ? "paused" : "auto";
        return (
          <g key={c.id} className={`conn ${stateClass(c)} ${menuConn === c.id ? "conn-open" : ""}`}>
            <path
              d={d}
              className="conn-line"
              vectorEffect="non-scaling-stroke"
              markerEnd={c.direction !== "backward" ? `url(#arrow-${kind})` : undefined}
              markerStart={c.direction !== "forward" ? `url(#arrow-${kind})` : undefined}
            />
            <path
              d={d}
              className="conn-hit"
              vectorEffect="non-scaling-stroke"
              onPointerDown={(e) => {
                e.stopPropagation();
                useUi.getState().selectConnection(c.id);
              }}
            />
            {pulses
              .filter((p) => p.connId === c.id)
              .map((p) => (
                <circle key={p.key} r={5} className="conn-pulse">
                  <animateMotion
                    dur="0.9s"
                    fill="freeze"
                    path={p.fromId === c.from ? d : `M${p2.x} ${p2.y} L${p1.x} ${p1.y}`}
                  />
                </circle>
              ))}
          </g>
        );
      })}
      {draft && rects.get(draft.fromId) && (
        <path
          className="conn-draft"
          vectorEffect="non-scaling-stroke"
          d={(() => {
            const from = edgePoint(rects.get(draft.fromId)!, { x: draft.x, y: draft.y });
            return `M${from.x} ${from.y} L${draft.x} ${draft.y}`;
          })()}
        />
      )}
    </svg>
  );
}

/** Menu da conexão selecionada, sobre o canvas (tamanho fixo, não escala com o zoom). */
export function ConnectionMenu({ viewport }: { viewport: Viewport }) {
  const id = useUi((s) => s.menuConn);
  const conn = useUi((s) => s.connections.find((c) => c.id === id));
  const nodes = useUi((s) => s.nodes);
  if (!conn) return null;
  const a = nodes.find((n) => n.id === conn.from);
  const b = nodes.find((n) => n.id === conn.to);
  if (!a || !b) return null;
  const [p1, p2] = linePoints(a.rect, b.rect);
  const at = worldToScreen(viewport, { x: (p1.x + p2.x) / 2, y: (p1.y + p2.y) / 2 });
  const ui = useUi.getState();
  const dirs = [
    ["forward", `${a.name} → ${b.name}`],
    ["backward", `${b.name} → ${a.name}`],
    ["both", `${a.name} ↔ ${b.name}`],
  ] as const;
  const paused = conn.state !== "active";
  return (
    <div
      className="conn-menu"
      style={{ left: at.x, top: at.y }}
      onPointerDown={(e) => e.stopPropagation()}
    >
      <div className="conn-menu-title">{t.connection.direction}</div>
      {dirs.map(([value, label]) => (
        <button
          key={value}
          className={conn.direction === value ? "on" : ""}
          onClick={() => void ui.setDirection(conn.id, value)}
        >
          {label}
        </button>
      ))}
      <hr />
      {paused ? (
        <button onClick={() => void ui.resumeConnection(conn.id)}>{t.connection.resume}</button>
      ) : (
        <button onClick={() => void ui.pauseConnection(conn.id)}>{t.connection.pause}</button>
      )}
      <button onClick={() => ui.openHistory(conn.id)}>{t.connection.history}</button>
      <button className="danger" onClick={() => void ui.deleteConnection(conn.id)}>
        {t.connection.remove}
      </button>
      <button onClick={() => ui.selectConnection(null)}>{t.connection.close}</button>
    </div>
  );
}
