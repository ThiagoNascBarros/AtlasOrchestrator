// Estado da UI (Zustand). O backend é a fonte da verdade do workspace; a UI aplica as mudanças
// de forma otimista e as envia por comandos. Mover, redimensionar, criar e apagar entram no
// histórico de desfazer/refazer (FR-002).
import { create } from "zustand";
import { api } from "../api/tauri";
import type {
  Agent,
  AgentStatusValue,
  CanvasNode,
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

export interface AgentStatus {
  value: AgentStatusValue;
  code?: number | null;
}

const HISTORY_LIMIT = 200;

interface UiState {
  root: string | null;
  nodes: CanvasNode[];
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
  setStatus(agentId: string, status: AgentStatus): void;
  bumpSession(agentId: string): void;
  toggleGrid(): void;
  openNewAgent(at: { x: number; y: number } | null): void;
  setError(msg: string | null): void;
}

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
      wsName: view.workspace.name,
      viewport: view.workspace.viewport,
      focus: initialFocus,
      past: [],
      future: [],
    }),

  unload: () => set({ root: null, nodes: [], statuses: {}, sessionGen: {}, past: [], future: [] }),

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

  setStatus: (agentId, status) => set((s) => ({ statuses: { ...s.statuses, [agentId]: status } })),
  bumpSession: (agentId) =>
    set((s) => ({ sessionGen: { ...s.sessionGen, [agentId]: (s.sessionGen[agentId] ?? 0) + 1 } })),
  toggleGrid: () => set((s) => ({ grid: !s.grid })),
  openNewAgent: (at) => set({ newAgentAt: at }),
  setError: (msg) => set({ error: msg }),
}));

export { errorText };
