// Criar conexão arrastando de um agente até outro: a linha segue o ponteiro e, ao soltar sobre
// outro nó, o backend cria a conexão (sentido: do nó de origem para o de destino).
import { screenToWorld } from "../canvas/viewport";
import { useUi } from "../store";

export function startConnect(fromId: string, e: React.PointerEvent) {
  if (e.button !== 0) return;
  e.stopPropagation();
  e.preventDefault();
  const viewportEl = document.querySelector(".viewport");

  const toWorld = (ev: PointerEvent) => {
    const r = viewportEl?.getBoundingClientRect();
    return screenToWorld(useUi.getState().viewport, {
      x: ev.clientX - (r?.left ?? 0),
      y: ev.clientY - (r?.top ?? 0),
    });
  };

  const move = (ev: PointerEvent) => useUi.getState().setDraft({ fromId, ...toWorld(ev) });
  const finish = (ev: PointerEvent) => {
    window.removeEventListener("pointermove", move);
    window.removeEventListener("pointerup", finish);
    window.removeEventListener("pointercancel", finish);
    const s = useUi.getState();
    s.setDraft(null);
    const target = document.elementFromPoint(ev.clientX, ev.clientY);
    const toId = target?.closest<HTMLElement>("[data-node-id]")?.dataset.nodeId;
    if (toId && toId !== fromId) void s.connect(fromId, toId);
  };

  window.addEventListener("pointermove", move);
  window.addEventListener("pointerup", finish);
  window.addEventListener("pointercancel", finish);
  move(e.nativeEvent);
}
