// Espelha os tipos serializados pelo backend (crates/atlasoq-store/src/model.rs).

export interface Rect {
  x: number;
  y: number;
  w: number;
  h: number;
}

export interface Viewport {
  x: number;
  y: number;
  zoom: number;
}

export type LearnMode = "hook" | "prompt" | "type" | "off";

export interface LearnSettings {
  mode: LearnMode;
  ready_timeout_s: number;
  prompt_flag?: string;
}

export interface Terminal {
  command: string;
  args: string[];
  cwd: string;
  env: Record<string, string>;
  env_secret_keys: string[];
  cols: number;
  rows: number;
}

export interface Agent {
  kind: "agent";
  id: string;
  name: string;
  role: string;
  profile: string;
  rect: Rect;
  terminal: Terminal;
  learn: LearnSettings;
}

export interface Note {
  kind: "note";
  id: string;
  name: string;
  rect: Rect;
  file: string;
}

export type CanvasNode = Agent | Note;

export type Direction = "forward" | "backward" | "both";
export type ConnState = "active" | "paused" | "auto_paused";

export interface ConnectionLimits {
  max_exchanges: number;
  rate_per_min: number;
  burst: number;
  ping_pong_repeats: number;
}

export interface Connection {
  id: string;
  from: string;
  to: string;
  direction: Direction;
  state: ConnState;
  pause_reason?: "max_exchanges" | "rate_limit" | "ping_pong";
  limits: ConnectionLimits;
}

export interface Workspace {
  schema: number;
  id: string;
  name: string;
  viewport: Viewport;
  settings: { memory_versioned: boolean; limits_default: ConnectionLimits };
  nodes: CanvasNode[];
  connections: Connection[];
}

export interface WorkspaceView {
  root: string;
  workspace: Workspace;
}

export interface RecentEntry {
  root: string;
  name: string;
  opened_at: number;
}

export interface Profile {
  name: string;
  command: string;
  args: string[];
  env: Record<string, string>;
  learn: LearnSettings;
}

export interface NewAgent {
  profile: string;
  name: string;
  role: string;
  cwd: string;
  env: Record<string, string>;
  env_secret_keys: string[];
  x: number;
  y: number;
}

export type AgentStatusValue = "starting" | "working" | "idle" | "exited";

export interface AgentStatusEvent {
  agent_id: string;
  session: number;
  kind: "status" | "exited";
  value?: Exclude<AgentStatusValue, "exited">;
  code?: number | null;
}

export interface AttachInfo {
  attachment: number;
  cols: number;
  rows: number;
}
