import { useNavigate } from "react-router-dom";
import { useChatStore } from "@/store/chat";
import { useDaemonStore } from "@/store/daemon";
import CompanyPatternIcon from "./CompanyPatternIcon";

export default function CompanyRail() {
  const navigate = useNavigate();
  const channel = useChatStore((s) => s.channel);
  const setChannel = useChatStore((s) => s.setChannel);
  const companies = useDaemonStore((s) => s.companies);
  const tasks = useDaemonStore((s) => s.tasks);

  const activeCounts: Record<string, number> = {};
  for (const t of tasks) {
    if (t.status === "in_progress") {
      activeCounts[t.company] = (activeCounts[t.company] || 0) + 1;
    }
  }

  const selectedCompany = channel ?? null;

  return (
    <div className="rail">
      <div className="rail-inner">
        <div className="rail-add" title="New company" onClick={() => {}}>+</div>

        {companies.map((p) => {
          const isSelected = selectedCompany === p.name;
          const hasActive = (activeCounts[p.name] || 0) > 0;

          return (
            <div key={p.name} className="rail-project-wrapper">
              <button
                className="rail-project-btn"
                onClick={() => { setChannel(p.name); navigate("/"); }}
                title={p.name}
              >
                <CompanyPatternIcon name={p.name} selected={isSelected} />
                {hasActive && (
                  <span className="rail-live-dot">
                    <span className="rail-live-dot-pulse" />
                    <span className="rail-live-dot-core" />
                  </span>
                )}
              </button>
            </div>
          );
        })}
      </div>
    </div>
  );
}
