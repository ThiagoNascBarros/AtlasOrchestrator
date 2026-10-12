// Cartão leve exibido no lugar do terminal quando o zoom está baixo ou o nó está fora da tela.
import { useEffect, useState } from "react";
import { api } from "../api/tauri";

export function AgentCard({ agentId, lines = 6 }: { agentId: string; lines?: number }) {
  const [text, setText] = useState("");
  useEffect(() => {
    let alive = true;
    const load = () =>
      api
        .sessionText(agentId, lines)
        .then((t) => alive && setText(t))
        .catch(() => alive && setText(""));
    void load();
    const timer = setInterval(load, 2000);
    return () => {
      alive = false;
      clearInterval(timer);
    };
  }, [agentId, lines]);
  return <pre className="agent-card-lines">{text}</pre>;
}
