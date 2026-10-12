import { api } from "../api/tauri";
import { frameAll, zoomBy } from "../canvas/Viewport";
import { screenToWorld } from "../canvas/viewport";
import { selectedIds } from "../focus/focusMachine";
import { t } from "../i18n/pt-BR";
import { useUi } from "../store";

export function Toolbar() {
  const name = useUi((s) => s.wsName);
  const root = useUi((s) => s.root);
  const canUndo = useUi((s) => s.past.length > 0);
  const canRedo = useUi((s) => s.future.length > 0);
  const grid = useUi((s) => s.grid);
  const focus = useUi((s) => s.focus.focus);
  const zoom = useUi((s) => s.viewport.zoom);

  const newAgent = () => {
    const s = useUi.getState();
    const el = document.querySelector(".viewport");
    const c = el ? { x: el.clientWidth / 2, y: el.clientHeight / 2 } : { x: 0, y: 0 };
    s.openNewAgent(screenToWorld(s.viewport, c));
  };

  return (
    <nav className="toolbar">
      <span className="ws-name" title={root ?? ""}>
        {name}
      </span>
      <button className="primary" onClick={newAgent} title="N">
        + {t.toolbar.newAgent}
      </button>
      <span className="sep" />
      <button onClick={() => void useUi.getState().undo()} disabled={!canUndo} title="Ctrl+Z">
        {t.toolbar.undo}
      </button>
      <button onClick={() => void useUi.getState().redo()} disabled={!canRedo} title="Ctrl+Shift+Z">
        {t.toolbar.redo}
      </button>
      <span className="sep" />
      <button onClick={() => frameAll()} title="Shift+1">
        {t.toolbar.frameAll}
      </button>
      <button
        onClick={() => frameAll(selectedIds(focus))}
        disabled={!selectedIds(focus).length}
        title="Shift+2"
      >
        {t.toolbar.frameSelection}
      </button>
      <button onClick={() => zoomBy(1 / 1.2)} aria-label="Afastar">
        −
      </button>
      <button onClick={() => zoomBy(0)} aria-label="Zoom 100%" title="100%">
        {Math.round(zoom * 100)}%
      </button>
      <button onClick={() => zoomBy(1.2)} aria-label="Aproximar">
        +
      </button>
      <label className="toggle">
        <input type="checkbox" checked={grid} onChange={() => useUi.getState().toggleGrid()} />
        {t.toolbar.grid}
      </label>
      <span className="grow" />
      <button
        onClick={async () => {
          await api.workspaceClose();
          useUi.getState().unload();
        }}
      >
        {t.toolbar.close}
      </button>
    </nav>
  );
}

export function StatusBar() {
  const focus = useUi((s) => s.focus.focus);
  const zoom = useUi((s) => s.viewport.zoom);
  const error = useUi((s) => s.error);
  const label =
    focus.kind === "terminal"
      ? t.focus.terminal
      : focus.kind === "note"
        ? t.focus.note
        : focus.kind === "selected"
          ? `${t.focus.selected} (${focus.ids.length})`
          : t.focus.canvas;
  return (
    <footer className="statusbar">
      <span className={`focus-indicator focus-${focus.kind}`}>{label}</span>
      {error && (
        <button
          className="error-msg"
          onClick={() => useUi.getState().setError(null)}
          title="Fechar"
        >
          {error}
        </button>
      )}
      <span className="grow" />
      <span>{Math.round(zoom * 100)}%</span>
    </footer>
  );
}
