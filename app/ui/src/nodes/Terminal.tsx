// Terminal xterm.js ligado à sessão PTY de um agente, com a estratégia de zoom do R3.
import { FitAddon } from "@xterm/addon-fit";
import { SearchAddon } from "@xterm/addon-search";
import { Unicode11Addon } from "@xterm/addon-unicode11";
import { WebLinksAddon } from "@xterm/addon-web-links";
import { WebglAddon } from "@xterm/addon-webgl";
import { Terminal as XTerm } from "@xterm/xterm";
import "@xterm/xterm/css/xterm.css";
import { useEffect, useRef, useState } from "react";
import { api } from "../api/tauri";
import { useUi } from "../store";
import { pathLinkProvider } from "./terminalLinks";
import { BASE_FONT_SIZE, counterScale, SETTLE_MS, settledFontSize } from "./terminalZoom";

interface Props {
  agentId: string;
  /** Tamanho da área do terminal em coordenadas de mundo. */
  width: number;
  height: number;
  focused: boolean;
}

const THEME = {
  background: "#0e1626",
  foreground: "#e8e6dc",
  cursor: "#f2a65a",
  selectionBackground: "#2f4a6b",
};

const USE_WEBGL = !/Linux/.test(navigator.userAgent);

export function TerminalView({ agentId, width, height, focused }: Props) {
  const host = useRef<HTMLDivElement>(null);
  const term = useRef<XTerm | null>(null);
  const fit = useRef<FitAddon | null>(null);
  const zoom = useUi((s) => s.viewport.zoom);
  const gen = useUi((s) => s.sessionGen[agentId] ?? 0);
  const dispatchFocus = useUi((s) => s.dispatchFocus);
  const [settled, setSettled] = useState(zoom);

  // Zoom estabilizado: só troca a fonte quando o gesto para.
  useEffect(() => {
    const t = setTimeout(() => setSettled(zoom), SETTLE_MS);
    return () => clearTimeout(t);
  }, [zoom]);

  // Cria o xterm e conecta à sessão (de novo a cada nova sessão do agente).
  useEffect(() => {
    const el = host.current;
    if (!el) return;
    const xterm = new XTerm({
      fontFamily: '"JetBrainsMono Nerd Font", "FiraCode Nerd Font", "Symbols Nerd Font", monospace',
      fontSize: settledFontSize(useUi.getState().viewport.zoom),
      allowProposedApi: true,
      scrollback: 10_000,
      theme: THEME,
      cursorBlink: true,
    });
    const fitAddon = new FitAddon();
    xterm.loadAddon(fitAddon);
    xterm.loadAddon(new SearchAddon());
    xterm.loadAddon(new Unicode11Addon());
    xterm.unicode.activeVersion = "11";
    xterm.loadAddon(new WebLinksAddon((_e, url) => void api.openUrl(url).catch(() => undefined)));
    xterm.registerLinkProvider(pathLinkProvider(xterm, agentId));
    xterm.open(el);
    // No WebKitGTK (webview do Tauri no Linux) o canvas WebGL do xterm fica em branco: os bytes
    // chegam, mas nada é pintado. Lá usamos o renderizador DOM.
    if (USE_WEBGL) {
      try {
        const webgl = new WebglAddon();
        webgl.onContextLoss(() => webgl.dispose());
        xterm.loadAddon(webgl);
      } catch {
        // Sem WebGL: o xterm usa o renderizador DOM.
      }
    }

    xterm.attachCustomKeyEventHandler((ev) => {
      if (ev.type === "keydown" && ev.key === "Escape") {
        dispatchFocus({ type: "escape", at: performance.now() });
      }
      return true;
    });
    const input = xterm.onData(
      (data) => void api.sessionWrite(agentId, data).catch(() => undefined),
    );
    const onFocus = () => dispatchFocus({ type: "enter_terminal", id: agentId });
    xterm.textarea?.addEventListener("focus", onFocus);

    term.current = xterm;
    fit.current = fitAddon;

    let attachment: number | null = null;
    let disposed = false;
    api
      .sessionAttach(agentId, (bytes) => xterm.write(bytes))
      .then((info) => {
        if (disposed) {
          void api.sessionDetach(info.attachment);
          return;
        }
        attachment = info.attachment;
        xterm.resize(info.cols, info.rows);
      })
      .catch(() => xterm.write("\r\n\x1b[33m[sem sessão ativa]\x1b[0m\r\n"));

    return () => {
      disposed = true;
      if (attachment !== null) void api.sessionDetach(attachment);
      input.dispose();
      xterm.textarea?.removeEventListener("focus", onFocus);
      xterm.dispose();
      term.current = null;
      fit.current = null;
    };
  }, [agentId, gen, dispatchFocus]);

  // Fonte proporcional ao zoom estabilizado, mantendo colunas e linhas.
  useEffect(() => {
    const xterm = term.current;
    if (!xterm) return;
    const { cols, rows } = xterm;
    xterm.options.fontSize = settledFontSize(settled);
    xterm.resize(cols, rows);
  }, [settled]);

  // O usuário redimensionou o nó: recalcula colunas e linhas e avisa o PTY.
  useEffect(() => {
    const t = setTimeout(() => {
      const xterm = term.current;
      const dims = fit.current?.proposeDimensions();
      if (!xterm || !dims || !Number.isFinite(dims.cols) || !Number.isFinite(dims.rows)) return;
      const cols = Math.max(20, dims.cols);
      const rows = Math.max(5, dims.rows);
      if (cols !== xterm.cols || rows !== xterm.rows) {
        xterm.resize(cols, rows);
        void api.sessionResize(agentId, cols, rows).catch(() => undefined);
      }
    }, 100);
    return () => clearTimeout(t);
  }, [width, height, agentId]);

  useEffect(() => {
    if (focused) term.current?.focus();
    else term.current?.blur();
  }, [focused]);

  const scale = counterScale(settled);
  return (
    <div className="terminal-clip" style={{ width, height }}>
      <div
        ref={host}
        className="terminal-host"
        style={{
          width: width * settled,
          height: height * settled,
          transform: `scale(${scale})`,
          transformOrigin: "0 0",
        }}
        data-base-font={BASE_FONT_SIZE}
      />
    </div>
  );
}
