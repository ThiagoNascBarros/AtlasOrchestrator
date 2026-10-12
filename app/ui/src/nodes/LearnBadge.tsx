// Selo com o estado do aprendizado do agente: `aprendendo…`, `pronto` ou o motivo da falha.
import { t } from "../i18n/pt-BR";
import { useUi } from "../store";

export function LearnBadge({ agentId }: { agentId: string }) {
  const learn = useUi((s) => s.learn[agentId]);
  if (!learn || learn.status === "off") return null;
  if (learn.status === "pending") {
    return <span className="learn learn-pending">{t.learn.pending}</span>;
  }
  if (learn.status === "done") {
    return <span className="learn learn-done">{t.learn.done}</span>;
  }
  return (
    <span className="learn learn-failed" title={learn.reason ?? undefined}>
      {t.learn.failed(learn.reason ?? "motivo desconhecido")}
    </span>
  );
}
