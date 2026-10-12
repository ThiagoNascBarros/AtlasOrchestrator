import { useEffect } from "react";
import { api, events } from "./api/tauri";
import { HistoryPanel } from "./connections/HistoryPanel";
import { t } from "./i18n/pt-BR";
import { Viewport } from "./canvas/Viewport";
import { NewAgentDialog } from "./nodes/NewAgentDialog";
import { useUi } from "./store";
import { StartScreen } from "./workspace/StartScreen";
import { StatusBar, Toolbar } from "./workspace/Toolbar";

export function App() {
  const root = useUi((s) => s.root);

  useEffect(() => {
    // Recupera o workspace se a UI recarregar com o app aberto.
    api
      .workspaceCurrent()
      .then((v) => v && useUi.getState().load(v))
      .catch(() => undefined);
    const unlisten = [
      events.agentStatus((e) =>
        useUi.getState().setStatus(e.agent_id, {
          value: e.kind === "exited" ? "exited" : (e.value ?? "starting"),
          code: e.code,
        }),
      ),
      events.agentSession((id) => useUi.getState().bumpSession(id)),
      events.linkMessage((e) => useUi.getState().addPulse(e.conn_id, e.from_id)),
      events.connectionChanged((c) => {
        const s = useUi.getState();
        const was = s.connections.find((x) => x.id === c.id);
        s.upsertConnection(c);
        if (c.state === "auto_paused" && was?.state !== "auto_paused" && c.pause_reason) {
          const name = (id: string) => s.nodes.find((n) => n.id === id)?.name ?? "?";
          s.setError(
            t.connection.autoPausedNotice(
              name(c.from),
              name(c.to),
              t.connection.reasons[c.pause_reason],
            ),
          );
        }
      }),
    ];
    return () => unlisten.forEach((p) => void p.then((f) => f()));
  }, []);

  if (!root) return <StartScreen />;
  return (
    <div className="app">
      <Toolbar />
      <Viewport />
      <StatusBar />
      <HistoryPanel />
      <NewAgentDialog />
    </div>
  );
}
