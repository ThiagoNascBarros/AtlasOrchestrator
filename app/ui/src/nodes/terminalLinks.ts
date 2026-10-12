// Links clicáveis no terminal: URLs (navegador) e `caminho:linha` (editor). FR-011.
import type { ILinkProvider, Terminal } from "@xterm/xterm";
import { api } from "../api/tauri";

/** `src/main.rs:42`, `./a/b.ts:7:3`, `/abs/file.py:10` */
export const PATH_LINE = /((?:\.{0,2}\/)?[\w.\-/]+\.[A-Za-z0-9]+):(\d+)(?::\d+)?/g;

export interface PathMatch {
  path: string;
  line: number;
  start: number;
  end: number;
}

export function findPathLinks(text: string): PathMatch[] {
  const out: PathMatch[] = [];
  for (const m of text.matchAll(PATH_LINE)) {
    const idx = m.index ?? 0;
    out.push({ path: m[1]!, line: Number(m[2]), start: idx, end: idx + m[0].length });
  }
  return out;
}

export function pathLinkProvider(term: Terminal, agentId: string): ILinkProvider {
  return {
    provideLinks(y, callback) {
      const line = term.buffer.active.getLine(y - 1)?.translateToString(true) ?? "";
      const links = findPathLinks(line).map((m) => ({
        range: { start: { x: m.start + 1, y }, end: { x: m.end, y } },
        text: line.slice(m.start, m.end),
        activate: () => void api.openPath(agentId, m.path, m.line).catch(() => undefined),
      }));
      callback(links.length ? links : undefined);
    },
  };
}
