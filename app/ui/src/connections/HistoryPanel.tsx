// Painel de histórico de uma conexão (horário, origem, destino, texto, resultado) com o aviso de
// pausa automática e o botão de reativar.
import { useEffect, useState } from "react";
import { api } from "../api/tauri";
import type { HistoryEntry } from "../api/types";
import { t } from "../i18n/pt-BR";
import { useUi } from "../store";

const time = (ts: string) => ts.slice(11, 19);

export function HistoryPanel() {
  const id = useUi((s) => s.historyConn);
  const conn = useUi((s) => s.connections.find((c) => c.id === id));
  const nodes = useUi((s) => s.nodes);
  const tick = useUi((s) => s.msgTick);
  const [rows, setRows] = useState<HistoryEntry[]>([]);

  useEffect(() => {
    if (!id) return;
    let alive = true;
    api
      .connectionHistory(id)
      .then((r) => alive && setRows(r))
      .catch(() => undefined);
    return () => {
      alive = false;
    };
  }, [id, tick, conn?.state]);

  if (!id || !conn) return null;
  const name = (nid: string) => nodes.find((n) => n.id === nid)?.name ?? "?";
  const state =
    conn.state === "active"
      ? t.connection.active
      : conn.state === "paused"
        ? t.connection.paused
        : t.connection.autoPaused;
  const ui = useUi.getState();
  const why = conn.pause_reason ? t.connection.reasons[conn.pause_reason] : "";

  return (
    <aside className="history-panel" aria-label={t.connection.history}>
      <header>
        <strong>
          {name(conn.from)}{" "}
          {conn.direction === "both" ? "↔" : conn.direction === "forward" ? "→" : "←"}{" "}
          {name(conn.to)}
        </strong>
        <span className={`pill conn-state-${conn.state}`}>{state}</span>
        <button
          className="icon-btn"
          onClick={() => ui.openHistory(null)}
          aria-label={t.connection.close}
        >
          ✕
        </button>
      </header>
      {conn.state === "auto_paused" && (
        <div className="history-warn">
          {t.connection.autoPausedNotice(name(conn.from), name(conn.to), why)}
        </div>
      )}
      {conn.state !== "active" && (
        <button className="primary" onClick={() => void ui.resumeConnection(conn.id)}>
          {t.connection.resume}
        </button>
      )}
      <ul>
        {rows.length === 0 && <li className="muted">{t.connection.empty}</li>}
        {rows.map((r, i) => (
          <li key={`${r.ts}-${i}`} className={r.result?.startsWith("refused") ? "refused" : ""}>
            <span className="h-time">{time(r.ts)}</span>
            {r.event === "message" ? (
              <>
                <span className="h-who">
                  {r.from} → {r.to} <em>{r.kind}</em>
                </span>
                {r.text && (
                  <span className="h-text" title={r.text}>
                    {r.text}
                  </span>
                )}
                <span className="h-result">{r.result}</span>
              </>
            ) : (
              <span className="h-who">
                {t.connection.events[r.event as keyof typeof t.connection.events] ?? r.event}
                {r.result && r.event === "connection_paused" ? ` (${r.result})` : ""}
              </span>
            )}
          </li>
        ))}
      </ul>
    </aside>
  );
}
