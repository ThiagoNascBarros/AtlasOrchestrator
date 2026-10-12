// Estado da UI (Zustand). O backend é a fonte da verdade do workspace; a UI aplica as mudanças
// de forma otimista e as envia por comandos. Mover, redimensionar, criar e apagar entram no
// histórico de desfazer/refazer (FR-002).
import { create } from "zustand";
import { api } from "../api/tauri";
import type {
  Agent,
  AgentStatusValue,
  CanvasNode,
  Connection,
  Direction,
  NewAgent,
  Rect,
  Viewport,
  WorkspaceView,
} from "../api/types";
import { type FocusEvent, type FocusState, initialFocus, reduceFocus } from "../focus/focusMachine";

export interface RectChange {
  id: string;
  rect: Rect;
}

export type HistoryCmd =
  | { kind: "rects"; before: RectChange[]; after: RectChange[] }
  | { kind: "create"; node: Agent }
  | { kind: "delete"; node: Agent };

/** Mensagem passando por uma conexão (animação da linha). */
export interface Pulse {
  key: number;
  connId: string;
  fromId: string;
}

export interface AgentStatus {
  value: AgentStatusValue;
  code?: number | null;
}

const HISTORY_LIMIT = 200;

interface UiState {
  root: string | null;
  nodes: CanvasNode[];
  connections: Connection[];
  pulses: Pulse[];
  /** Linha sendo arrastada de um agente para outro (coordenadas do mundo). */
  draft: { fromId: string; x: number; y: number } | null;
  /** Conexão com o menu aberto e conexão com o histórico aberto. */
  menuConn: string | null;
  historyConn: string | null;
  /** Sobe a cada mensagem; o painel de histórico recarrega. */
  msgTick: number;
  wsName: string;
  viewport: Viewport;
  focus: FocusState;
  statuses: Record<string, AgentStatus>;
  /** Muda quando o agente ganha nova sessão; o terminal reconecta. */
  sessionGen: Record<string, number>;
  grid: boolean;
  newAgentAt: { x: number; y: number } | null;
  past: HistoryCmd[];
  future: HistoryCmd[];
  error: string | null;

  load(view: WorkspaceView): void;
  unload(): void;
  setViewport(v: Viewport): void;
  dispatchFocus(ev: FocusEvent): void;
  setRects(changes: RectChange[], record?: { before: RectChange[] }): void;
  createAgent(req: NewAgent): Promise<Agent>;
  deleteNodes(ids: string[]): Promise<void>;
  undo(): Promise<void>;
  redo(): Promise<void>;
  upsertConnection(c: Connection): void;
  connect(fromId: string, toId: string): Promise<void>;
  setDirection(id: string, d: Direction): Promise<void>;
  pauseConnection(id: string): Promise<void>;
  resumeConnection(id: string): Promise<void>;
  deleteConnection(id: string): Promise<void>;
  setDraft(d: { fromId: string; x: number; y: number } | null): void;
  selectConnection(id: string | null): void;
  openHistory(id: string | null): void;
  addPulse(connId: string, fromId: string): void;
  setStatus(agentId: string, status: AgentStatus): void;
  bumpSession(agentId: string): void;
  toggleGrid(): void;
  openNewAgent(at: { x: number; y: number } | null): void;
  setError(msg: string | null): void;
}

let pulseKey = 0;
let viewportTimer: ReturnType<typeof setTimeout> | undefined;

const errorText = (e: unknown) => (e instanceof Error ? e.message : String(e));

/** Recria um agente apagado (desfazer) e devolve o novo, com id novo. */
async function recreate(a: Agent): Promise<Agent> {
  const created = await api.agentCreate({
    profile: a.profile,
    name: a.name,
    role: a.role,
    cwd: a.terminal.cwd,
    env: a.terminal.env,
    env_secret_keys: a.terminal.env_secret_keys,
    x: a.rect.x,
    y: a.rect.y,
  });
  if (a.rect.w !== created.rect.w || a.rect.h !== created.rect.h) {
    await api.setRect(created.id, a.rect);
    return { ...created, rect: a.rect };
  }
  return created;
}

/** Troca um id por outro em todo o histórico (após recriar um nó). */
function remapHistory(list: HistoryCmd[], from: string, to: Agent): HistoryCmd[] {
  const fix = (c: RectChange) => (c.id === from ? { ...c, id: to.id } : c);
  return list.map((cmd) => {
    if (cmd.kind === "rects")
      return { ...cmd, before: cmd.before.map(fix), after: cmd.after.map(fix) };
    return cmd.node.id === from ? { ...cmd, node: { ...to, rect: cmd.node.rect } } : cmd;
  });
}

export const useUi = create<UiState>((set, get) => ({
  root: null,
  nodes: [],
  connections: [],
  pulses: [],
  draft: null,
  menuConn: null,
  historyConn: null,
  msgTick: 0,
  wsName: "",
  viewport: { x: 0, y: 0, zoom: 1 },
  focus: initialFocus,
  statuses: {},
  sessionGen: {},
  grid: true,
  newAgentAt: null,
  past: [],
  future: [],
  error: null,

  load: (view) =>
    set({
      root: view.root,
      nodes: view.workspace.nodes,
      connections: view.workspace.connections ?? [],
      pulses: [],
      draft: null,
      menuConn: null,
      historyConn: null,
      wsName: view.workspace.name,
      viewport: view.workspace.viewport,
      focus: initialFocus,
      past: [],
      future: [],
    }),

  unload: () =>
    set({
      root: null,
      nodes: [],
      connections: [],
      menuConn: null,
      historyConn: null,
      draft: null,
      statuses: {},
      sessionGen: {},
      past: [],
      future: [],
    }),

  setViewport: (v) => {
    set({ viewport: v });
    clearTimeout(viewportTimer);
    viewportTimer = setTimeout(() => void api.setViewport(v), 300);
  },

  dispatchFocus: (ev) => set((s) => ({ focus: reduceFocus(s.focus, ev) })),

  setRects: (changes, record) => {
    const byId = new Map(changes.map((c) => [c.id, c.rect]));
    set((s) => ({
      nodes: s.nodes.map((n) => (byId.has(n.id) ? { ...n, rect: byId.get(n.id)! } : n)),
      ...(record
        ? {
            past: [
              ...s.past,
              { kind: "rects" as const, before: record.before, after: changes },
            ].slice(-HISTORY_LIMIT),
            future: [],
          }
        : {}),
    }));
    if (record) for (const c of changes) void api.setRect(c.id, c.rect);
  },

  createAgent: async (req) => {
    const agent = await api.agentCreate(req);
    set((s) => ({
      nodes: [...s.nodes, agent],
      past: [...s.past, { kind: "create" as const, node: agent }].slice(-HISTORY_LIMIT),
      future: [],
    }));
    return agent;
  },

  deleteNodes: async (ids) => {
    const victims = get().nodes.filter((n) => ids.includes(n.id));
    for (const n of victims) {
      await api.nodeDelete(n.id);
      get().dispatchFocus({ type: "node_removed", id: n.id });
    }
    set((s) => ({
      nodes: s.nodes.filter((n) => !ids.includes(n.id)),
      connections: s.connections.filter((c) => !ids.includes(c.from) && !ids.includes(c.to)),
      past: [
        ...s.past,
        ...victims
          .filter((n): n is Agent => n.kind === "agent")
          .map((node) => ({ kind: "delete" as const, node })),
      ].slice(-HISTORY_LIMIT),
      future: [],
    }));
  },

  undo: async () => {
    const cmd = get().past.at(-1);
    if (!cmd) return;
    set((s) => ({ past: s.past.slice(0, -1) }));
    try {
      if (cmd.kind === "rects") {
        get().setRects(cmd.before);
        for (const c of cmd.before) void api.setRect(c.id, c.rect);
        set((s) => ({ future: [...s.future, cmd] }));
      } else if (cmd.kind === "create") {
        await api.nodeDelete(cmd.node.id);
        get().dispatchFocus({ type: "node_removed", id: cmd.node.id });
        set((s) => ({
          nodes: s.nodes.filter((n) => n.id !== cmd.node.id),
          connections: s.connections.filter((c) => c.from !== cmd.node.id && c.to !== cmd.node.id),
          future: [...s.future, cmd],
        }));
      } else {
        const agent = await recreate(cmd.node);
        set((s) => ({
          nodes: [...s.nodes, agent],
          past: remapHistory(s.past, cmd.node.id, agent),
          future: [...remapHistory(s.future, cmd.node.id, agent), { kind: "delete", node: agent }],
        }));
      }
    } catch (e) {
      set({ error: errorText(e) });
    }
  },

  redo: async () => {
    const cmd = get().future.at(-1);
    if (!cmd) return;
    set((s) => ({ future: s.future.slice(0, -1) }));
    try {
      if (cmd.kind === "rects") {
        get().setRects(cmd.after);
        for (const c of cmd.after) void api.setRect(c.id, c.rect);
        set((s) => ({ past: [...s.past, cmd] }));
      } else if (cmd.kind === "delete") {
        await api.nodeDelete(cmd.node.id);
        get().dispatchFocus({ type: "node_removed", id: cmd.node.id });
        set((s) => ({
          nodes: s.nodes.filter((n) => n.id !== cmd.node.id),
          connections: s.connections.filter((c) => c.from !== cmd.node.id && c.to !== cmd.node.id),
          past: [...s.past, cmd],
        }));
      } else {
        const agent = await recreate(cmd.node);
        set((s) => ({
          nodes: [...s.nodes, agent],
          future: remapHistory(s.future, cmd.node.id, agent),
          past: [...remapHistory(s.past, cmd.node.id, agent), { kind: "create", node: agent }],
        }));
      }
    } catch (e) {
      set({ error: errorText(e) });
    }
  },

  upsertConnection: (c) =>
    set((s) => ({
      connections: s.connections.some((x) => x.id === c.id)
        ? s.connections.map((x) => (x.id === c.id ? c : x))
        : [...s.connections, c],
    })),

  connect: async (fromId, toId) => {
    try {
      get().upsertConnection(await api.connectionCreate(fromId, toId));
    } catch (e) {
      set({ error: errorText(e) });
    }
  },

  setDirection: async (id, d) => {
    try {
      get().upsertConnection(await api.connectionSetDirection(id, d));
    } catch (e) {
      set({ error: errorText(e) });
    }
  },

  pauseConnection: async (id) => {
    try {
      get().upsertConnection(await api.connectionPause(id));
    } catch (e) {
      set({ error: errorText(e) });
    }
  },

  resumeConnection: async (id) => {
    try {
      get().upsertConnection(await api.connectionResume(id));
    } catch (e) {
      set({ error: errorText(e) });
    }
  },

  deleteConnection: async (id) => {
    try {
      await api.connectionDelete(id);
      set((s) => ({
        connections: s.connections.filter((c) => c.id !== id),
        menuConn: s.menuConn === id ? null : s.menuConn,
        historyConn: s.historyConn === id ? null : s.historyConn,
      }));
    } catch (e) {
      set({ error: errorText(e) });
    }
  },

  setDraft: (draft) => set({ draft }),
  selectConnection: (id) => set({ menuConn: id }),
  openHistory: (id) => set({ historyConn: id, menuConn: null }),

  addPulse: (connId, fromId) => {
    const key = ++pulseKey;
    set((s) => ({ pulses: [...s.pulses, { key, connId, fromId }], msgTick: s.msgTick + 1 }));
    setTimeout(() => set((s) => ({ pulses: s.pulses.filter((p) => p.key !== key) })), 1100);
  },

  setStatus: (agentId, status) => set((s) => ({ statuses: { ...s.statuses, [agentId]: status } })),
  bumpSession: (agentId) =>
    set((s) => ({ sessionGen: { ...s.sessionGen, [agentId]: (s.sessionGen[agentId] ?? 0) + 1 } })),
  toggleGrid: () => set((s) => ({ grid: !s.grid })),
  openNewAgent: (at) => set({ newAgentAt: at }),
  setError: (msg) => set({ error: msg }),
}));

export { errorText };
