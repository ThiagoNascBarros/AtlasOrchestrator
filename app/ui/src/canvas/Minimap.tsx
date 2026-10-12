// Minimapa: retângulos dos nós e da área visível; clique centraliza o canvas.
import { useUi } from "../store";
import { boundsOf, visibleWorld } from "./viewport";

const W = 180;
const H = 120;

export function Minimap({ size }: { size: { w: number; h: number } }) {
  const nodes = useUi((s) => s.nodes);
  const viewport = useUi((s) => s.viewport);
  const view = visibleWorld(viewport, size);
  const b = boundsOf([...nodes.map((n) => n.rect), view]);
  if (!b || nodes.length === 0) return null;
  const k = Math.min(W / b.w, H / b.h);
  const tx = (x: number) => (x - b.x) * k;
  const ty = (y: number) => (y - b.y) * k;

  const onClick = (e: React.MouseEvent<SVGSVGElement>) => {
    e.stopPropagation();
    const r = e.currentTarget.getBoundingClientRect();
    const wx = (e.clientX - r.left) / k + b.x;
    const wy = (e.clientY - r.top) / k + b.y;
    const s = useUi.getState();
    s.setViewport({
      ...s.viewport,
      x: size.w / 2 - wx * s.viewport.zoom,
      y: size.h / 2 - wy * s.viewport.zoom,
    });
  };

  return (
    <svg
      className="minimap"
      width={W}
      height={H}
      onPointerDown={(e) => e.stopPropagation()}
      onClick={onClick}
      aria-label="Minimapa"
    >
      {nodes.map((n) => (
        <rect
          key={n.id}
          x={tx(n.rect.x)}
          y={ty(n.rect.y)}
          width={Math.max(2, n.rect.w * k)}
          height={Math.max(2, n.rect.h * k)}
          className={`mm-node mm-${n.kind}`}
        />
      ))}
      <rect
        x={tx(view.x)}
        y={ty(view.y)}
        width={view.w * k}
        height={view.h * k}
        className="mm-view"
      />
    </svg>
  );
}
