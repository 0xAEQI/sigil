import { useEffect, useState } from "react";
import Header from "@/components/Header";
import StatCard from "@/components/StatCard";
import { api } from "@/lib/api";

export default function CostPage() {
  const [cost, setCost] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    api.getCost().then((data) => {
      setCost(data);
      setLoading(false);
    }).catch(() => setLoading(false));
  }, []);

  if (loading) return <div className="loading">Loading cost data...</div>;
  if (!cost) return <div className="loading">Failed to load cost data</div>;

  const projectBudgets = cost.project_budgets || {};

  return (
    <>
      <Header title="Cost Tracking" />

      <div className="stat-cards">
        <StatCard
          label="Spent Today"
          value={`$${Number(cost.spent_today_usd || 0).toFixed(3)}`}
        />
        <StatCard
          label="Daily Budget"
          value={`$${Number(cost.daily_budget_usd || 0).toFixed(2)}`}
        />
        <StatCard
          label="Remaining"
          value={`$${Number(cost.remaining_usd || 0).toFixed(3)}`}
        />
      </div>

      <div className="budget-bar" style={{ marginBottom: "var(--space-8)" }}>
        <div className="budget-label">
          <span>Daily Budget Utilization</span>
          <span>
            {((Number(cost.spent_today_usd || 0) / Number(cost.daily_budget_usd || 1)) * 100).toFixed(1)}%
          </span>
        </div>
        <div className="progress-bar-bg">
          <div
            className="progress-bar-fill"
            style={{
              width: `${Math.min(100, (Number(cost.spent_today_usd || 0) / Number(cost.daily_budget_usd || 1)) * 100)}%`,
            }}
          />
        </div>
      </div>

      <h2 style={{ fontSize: "var(--font-size-lg)", marginBottom: "var(--space-4)" }}>
        Per-Project Breakdown
      </h2>
      <div className="cost-breakdown">
        {Object.entries(projectBudgets).map(([name, budget]: [string, any]) => (
          <div key={name} className="cost-item">
            <div className="cost-item-name">{name}</div>
            <div className="cost-item-value">${Number(budget.spent_usd || 0).toFixed(3)}</div>
            <div style={{ fontSize: "var(--font-size-xs)", color: "var(--text-muted)", marginTop: "var(--space-1)" }}>
              Budget: ${Number(budget.budget_usd || 0).toFixed(2)} / Remaining: ${Number(budget.remaining_usd || 0).toFixed(3)}
            </div>
          </div>
        ))}
      </div>
    </>
  );
}
