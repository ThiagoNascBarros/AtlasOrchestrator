// Nó de agente: barra de título (nome, papel, status, ações) e terminal ou cartão (LOD).
import { useState } from "react";
import { api } from "../api/tauri";
import type { Agent } from "../api/types";
import { NodeFrame, TITLE_H } from "../canvas/NodeFrame";
import { t } from "../i18n/pt-BR";
import { errorText, useUi } from "../store";
import { AgentCard } from "./AgentCard";
import { TerminalView } from "./Terminal";

export function StatusPill({ agentId }: { agentId: string }) {
  const st = useUi((s) => s.statuses[agentId]);
  const value = st?.value ?? "starting";
  const label = value === "exited" ? t.status.exited(st?.code) : t.status[value];
  return <span className={`pill pill-${value}`}>{label}</span>;
}

export function AgentNode({ agent, lod }: { agent: Agent; lod: boolean }) {
  const focus = useUi((s) => s.focus.focus);
  const setError = useUi((s) => s.setError);
  const [menu, setMenu] = useState(false);
  const focused = focus.kind === "terminal" && focus.id === agent.id;

  const run = (fn: () => Promise<unknown>) => () => {
    setMenu(false);
    fn().catch((e) => setError(errorText(e)));
  };

  const title = (
    <>
      <span className="node-name">{agent.name}</span>
      {agent.role && <span className="node-role">{agent.role}</span>}
      <StatusPill agentId={agent.id} />
      <span className="node-cwd" title={agent.terminal.cwd}>
        {agent.terminal.cwd}
      </span>
      <button
        className="icon-btn"
        aria-label={t.agent.menu}
        onPointerDown={(e) => e.stopPropagation()}
        onClick={() => setMenu((m) => !m)}
      >
        ⋯
      </button>
      {menu && (
        <div className="menu" onPointerDown={(e) => e.stopPropagation()}>
          <button onClick={run(() => api.agentRestart(agent.id))}>{t.agent.restart}</button>
          <button
            onClick={run(async () => {
              const copy = await api.agentDuplicate(agent.id);
              useUi.setState((s) => ({ nodes: [...s.nodes, copy] }));
            })}
          >
            {t.agent.duplicate}
          </button>
          <button onClick={run(() => api.agentStop(agent.id))}>{t.agent.stop}</button>
          <button className="danger" onClick={run(() => useUi.getState().deleteNodes([agent.id]))}>
            {t.agent.remove}
          </button>
        </div>
      )}
    </>
  );

  const bodyH = agent.rect.h - TITLE_H;
  return (
    <NodeFrame
      id={agent.id}
      rect={agent.rect}
      title={title}
      className="agent-node"
      onTitleDoubleClick={() =>
        useUi.getState().dispatchFocus({ type: "enter_terminal", id: agent.id })
      }
    >
      {lod ? (
        <AgentCard agentId={agent.id} />
      ) : (
        <TerminalView agentId={agent.id} width={agent.rect.w} height={bodyH} focused={focused} />
      )}
    </NodeFrame>
  );
}
