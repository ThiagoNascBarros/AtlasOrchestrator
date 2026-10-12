// Tela inicial: novo workspace (escolhendo a pasta do projeto), abrir e recentes.
import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useState } from "react";
import { api } from "../api/tauri";
import type { RecentEntry } from "../api/types";
import { t } from "../i18n/pt-BR";
import { errorText, useUi } from "../store";

const basename = (p: string) => p.replace(/\/+$/, "").split("/").pop() ?? p;

export function StartScreen() {
  const [recent, setRecent] = useState<RecentEntry[]>([]);
  const [creating, setCreating] = useState<{ root: string; name: string } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const load = useUi((s) => s.load);

  useEffect(() => {
    api
      .workspaceRecent()
      .then(setRecent)
      .catch(() => setRecent([]));
  }, []);

  const pick = async () => {
    const dir = await open({ directory: true, multiple: false });
    return typeof dir === "string" ? dir : null;
  };

  const openRoot = async (root: string) => {
    setError(null);
    try {
      load(await api.workspaceOpen(root));
    } catch (e) {
      setError(errorText(e));
    }
  };

  return (
    <main className="start">
      <header>
        <img src="/icon.svg" alt="" width={56} height={56} />
        <div>
          <h1>{t.appName}</h1>
          <p>{t.start.title}</p>
        </div>
      </header>
      <div className="start-actions">
        <button
          className="primary"
          onClick={async () => {
            const root = await pick();
            if (root) setCreating({ root, name: basename(root) });
          }}
        >
          {t.start.newWorkspace}
        </button>
        <button
          onClick={async () => {
            const root = await pick();
            if (root) await openRoot(root);
          }}
        >
          {t.start.openWorkspace}
        </button>
      </div>
      {creating && (
        <form
          className="start-create"
          onSubmit={async (e) => {
            e.preventDefault();
            setError(null);
            try {
              load(await api.workspaceCreate(creating.root, creating.name));
            } catch (err) {
              setError(errorText(err));
            }
          }}
        >
          <label>
            {t.start.folderLabel}
            <input value={creating.root} readOnly />
          </label>
          <label>
            {t.start.nameLabel}
            <input
              autoFocus
              value={creating.name}
              onChange={(e) => setCreating({ ...creating, name: e.target.value })}
            />
          </label>
          <div className="dialog-actions">
            <button type="button" onClick={() => setCreating(null)}>
              {t.start.cancel}
            </button>
            <button type="submit" className="primary" disabled={!creating.name.trim()}>
              {t.start.create}
            </button>
          </div>
        </form>
      )}
      {error && <p className="field-error">{error}</p>}
      <section>
        <h2>{t.start.recent}</h2>
        {recent.length === 0 ? (
          <p className="muted">{t.start.noRecent}</p>
        ) : (
          <ul className="recent">
            {recent.map((r) => (
              <li key={r.root}>
                <button onClick={() => void openRoot(r.root)}>
                  <strong>{r.name}</strong>
                  <span className="muted">{r.root}</span>
                </button>
              </li>
            ))}
          </ul>
        )}
      </section>
    </main>
  );
}
