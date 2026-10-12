// Diálogo "novo agente": perfil, nome único, papel, diretório e variáveis.
import { useEffect, useMemo, useState } from "react";
import { api } from "../api/tauri";
import type { Profile } from "../api/types";
import { t } from "../i18n/pt-BR";
import { errorText, useUi } from "../store";

const NAME_RE = /^[a-z][a-z0-9-]{0,31}$/;

export function validateName(name: string, taken: string[]): string | null {
  if (!name) return "informe um nome";
  if (!NAME_RE.test(name)) return t.newAgent.nameHint;
  if (name === "memory") return '"memory" é um nome reservado';
  if (taken.includes(name)) {
    const free = Array.from({ length: 998 }, (_, i) => `${name.slice(0, 29)}-${i + 2}`).find(
      (n) => !taken.includes(n),
    );
    return `já existe "${name}"; sugestão: "${free}"`;
  }
  return null;
}

export function parseEnv(text: string): Record<string, string> {
  const env: Record<string, string> = {};
  for (const line of text.split("\n")) {
    const i = line.indexOf("=");
    if (i > 0) env[line.slice(0, i).trim()] = line.slice(i + 1).trim();
  }
  return env;
}

export function NewAgentDialog() {
  const at = useUi((s) => s.newAgentAt);
  // Montado só quando aberto: o estado do formulário recomeça a cada abertura.
  return at ? <NewAgentForm at={at} /> : null;
}

function NewAgentForm({ at }: { at: { x: number; y: number } }) {
  const nodes = useUi((s) => s.nodes);
  const [profiles, setProfiles] = useState<Profile[]>([]);
  const [profile, setProfile] = useState("claude-code");
  const [name, setName] = useState("");
  const [role, setRole] = useState("");
  const [cwd, setCwd] = useState(".");
  const [env, setEnv] = useState("");
  const [secrets, setSecrets] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    api
      .profiles()
      .then(setProfiles)
      .catch((e) => setError(errorText(e)));
  }, []);

  const taken = useMemo(() => nodes.map((n) => n.name), [nodes]);
  const nameError = name ? validateName(name, taken) : null;

  const close = () => useUi.getState().openNewAgent(null);
  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    const problem = validateName(name, taken);
    if (problem) return setError(problem);
    setBusy(true);
    try {
      const agent = await useUi.getState().createAgent({
        profile,
        name,
        role,
        cwd,
        env: parseEnv(env),
        env_secret_keys: secrets
          .split(",")
          .map((s) => s.trim())
          .filter(Boolean),
        x: Math.round(at.x - 360),
        y: Math.round(at.y - 230),
      });
      close();
      useUi.getState().dispatchFocus({ type: "enter_terminal", id: agent.id });
    } catch (err) {
      setError(errorText(err));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="dialog-backdrop" onPointerDown={close}>
      <form className="dialog" onPointerDown={(e) => e.stopPropagation()} onSubmit={submit}>
        <h2>{t.newAgent.title}</h2>
        <label>
          {t.newAgent.profile}
          <select value={profile} onChange={(e) => setProfile(e.target.value)}>
            {profiles.map((p) => (
              <option key={p.name} value={p.name}>
                {p.name} — {p.command}
              </option>
            ))}
          </select>
        </label>
        <label>
          {t.newAgent.name}
          <input
            autoFocus
            value={name}
            onChange={(e) => setName(e.target.value.toLowerCase())}
            placeholder="coder"
            aria-invalid={!!nameError}
          />
          <small className={nameError ? "field-error" : ""}>
            {nameError ?? t.newAgent.nameHint}
          </small>
        </label>
        <label>
          {t.newAgent.role}
          <input
            value={role}
            maxLength={200}
            onChange={(e) => setRole(e.target.value)}
            placeholder={t.newAgent.rolePlaceholder}
          />
        </label>
        <label>
          {t.newAgent.cwd}
          <input value={cwd} onChange={(e) => setCwd(e.target.value)} />
        </label>
        <label>
          {t.newAgent.env}
          <textarea
            rows={3}
            value={env}
            onChange={(e) => setEnv(e.target.value)}
            placeholder="FOO=bar"
          />
        </label>
        <label>
          {t.newAgent.secrets}
          <input
            value={secrets}
            onChange={(e) => setSecrets(e.target.value)}
            placeholder="ANTHROPIC_API_KEY"
          />
        </label>
        {error && <p className="field-error">{error}</p>}
        <div className="dialog-actions">
          <button type="button" onClick={close}>
            {t.start.cancel}
          </button>
          <button type="submit" className="primary" disabled={busy || !!nameError || !name}>
            {t.newAgent.create}
          </button>
        </div>
      </form>
    </div>
  );
}
