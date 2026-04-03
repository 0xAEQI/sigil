import { useEffect, useState } from "react";
import Header from "@/components/Header";
import { DataState } from "@/components/ui";
import { api } from "@/lib/api";

export default function TriggersPage() {
  const [triggers, setTriggers] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    api.getTriggers()
      .then((d: any) => setTriggers(d.triggers || []))
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  const enabled = triggers.filter((t: any) => t.enabled !== false);
  const disabled = triggers.filter((t: any) => t.enabled === false);

  function triggerSchedule(t: any): string {
    const tt = t.trigger_type;
    if (!tt) return "\u2014";
    if (tt.Schedule) return tt.Schedule.expr;
    if (tt.Once) return `once at ${tt.Once.at}`;
    if (tt.Event) return `on ${tt.Event.pattern}`;
    if (tt.Webhook) return `webhook`;
    return "\u2014";
  }

  function timeAgo(ts: string): string {
    if (!ts) return "never";
    const diff = Date.now() - new Date(ts).getTime();
    const mins = Math.floor(diff / 60000);
    if (mins < 1) return "now";
    if (mins < 60) return `${mins}m ago`;
    const hrs = Math.floor(mins / 60);
    if (hrs < 24) return `${hrs}h ago`;
    return `${Math.floor(hrs / 24)}d ago`;
  }

  return (
    <>
      <Header title="Triggers" />

      <div className="trigger-stats">
        <span className="trigger-stat">{enabled.length} active</span>
        <span className="trigger-stat dim">{disabled.length} disabled</span>
        <span className="trigger-stat dim">{triggers.reduce((s: number, t: any) => s + (t.fire_count || 0), 0)} total fires</span>
      </div>

      <DataState loading={loading} empty={triggers.length === 0}
        emptyTitle="No triggers" emptyDescription="No triggers configured yet.">

        <div className="trigger-list">
          {triggers.map((t: any) => (
            <div key={t.id} className={`trigger-card${t.enabled === false ? " disabled" : ""}`}>
              <div className="trigger-card-header">
                <span className={`trigger-card-status${t.enabled === false ? " off" : " on"}`}>
                  {t.enabled === false ? "\u25CB" : "\u25CF"}
                </span>
                <span className="trigger-card-name">{t.name}</span>
                <code className="trigger-card-schedule">{triggerSchedule(t)}</code>
              </div>
              <div className="trigger-card-body">
                <span className="trigger-card-meta">
                  <span className="trigger-card-skill">{t.skill || "\u2014"}</span>
                  {t.max_budget_usd != null && (
                    <span className="trigger-card-budget">${t.max_budget_usd}</span>
                  )}
                </span>
                <span className="trigger-card-meta">
                  <span>{t.fire_count || 0} fires</span>
                  {t.last_fired && <span>last: {timeAgo(t.last_fired)}</span>}
                  {t.total_cost_usd > 0 && <span>${t.total_cost_usd.toFixed(2)} total</span>}
                </span>
              </div>
            </div>
          ))}
        </div>

      </DataState>
    </>
  );
}
