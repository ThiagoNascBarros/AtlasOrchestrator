// Cliente tipado dos comandos Tauri (app/src-tauri/src/lib.rs).
import { Channel, invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  Agent,
  AgentStatusEvent,
  AttachInfo,
  NewAgent,
  Profile,
  RecentEntry,
  Rect,
  Viewport,
  WorkspaceView,
} from "./types";

export const api = {
  workspaceCreate: (root: string, name: string) =>
    invoke<WorkspaceView>("workspace_create", { root, name }),
  workspaceOpen: (root: string) => invoke<WorkspaceView>("workspace_open", { root }),
  workspaceClose: () => invoke<null>("workspace_close"),
  workspaceCurrent: () => invoke<WorkspaceView | null>("workspace_current"),
  workspaceRecent: () => invoke<RecentEntry[]>("workspace_recent"),
  setViewport: (viewport: Viewport) => invoke<null>("workspace_set_viewport", { viewport }),
  setRect: (id: string, rect: Rect) => invoke<null>("node_set_rect", { id, rect }),
  profiles: () => invoke<Profile[]>("profiles_list"),

  agentCreate: (req: NewAgent) => invoke<Agent>("agent_create", { req }),
  agentRestart: (id: string) => invoke<null>("agent_restart", { id }),
  agentStop: (id: string) => invoke<null>("agent_stop", { id }),
  agentDuplicate: (id: string) => invoke<Agent>("agent_duplicate", { id }),
  nodeDelete: (id: string) => invoke<null>("node_delete", { id }),

  sessionAttach: (agentId: string, onBytes: (bytes: Uint8Array) => void) => {
    const output = new Channel<ArrayBuffer>();
    output.onmessage = (buf) => onBytes(new Uint8Array(buf));
    return invoke<AttachInfo>("session_attach", { agentId, output });
  },
  sessionDetach: (attachment: number) => invoke<null>("session_detach", { attachment }),
  sessionWrite: (agentId: string, data: string) => invoke<null>("session_write", { agentId, data }),
  sessionResize: (agentId: string, cols: number, rows: number) =>
    invoke<null>("session_resize", { agentId, cols, rows }),
  sessionText: (agentId: string, lines: number) =>
    invoke<string>("session_text", { agentId, lines }),

  openUrl: (url: string) => invoke<null>("open_url", { url }),
  openPath: (agentId: string, path: string, line?: number) =>
    invoke<null>("open_path", { agentId, path, line: line ?? null }),
};

export const events = {
  agentStatus: (cb: (e: AgentStatusEvent) => void): Promise<UnlistenFn> =>
    listen<AgentStatusEvent>("agent-status", (e) => cb(e.payload)),
  agentSession: (cb: (agentId: string) => void): Promise<UnlistenFn> =>
    listen<string>("agent-session", (e) => cb(e.payload)),
};
