import { useState, useEffect, useCallback } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import AgentTree from "./Sidebar";
import ContextPanel from "./ContextPanel";
import BlockAvatar from "./BlockAvatar";
import CommandPalette from "./CommandPalette";
import AgentSessionView from "./AgentSessionView";
import DashboardHome from "./DashboardHome";
import { useDaemonStore } from "@/store/daemon";
import { useDaemonSocket } from "@/hooks/useDaemonSocket";

export default function AppLayout() {
  const navigate = useNavigate();
  const [params] = useSearchParams();
  const [searching, setSearching] = useState(false);

  const agentId = params.get("agent");
  const sessionId = params.get("session");

  const fetchAll = useDaemonStore((s) => s.fetchAll);
  useEffect(() => { fetchAll(); const i = setInterval(fetchAll, 30000); return () => clearInterval(i); }, [fetchAll]);
  useDaemonSocket();

  const userName = localStorage.getItem("aeqi_user_name") || "Operator";

  const openSearch = useCallback(() => setSearching(true), []);
  const closeSearch = useCallback(() => setSearching(false), []);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        if (searching) closeSearch();
        else openSearch();
      }
      if (e.key === "Escape" && searching) {
        closeSearch();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [searching, openSearch, closeSearch]);

  return (
    <>
      <div className="shell">
        {/* Left sidebar: Agent tree */}
        <div className="left-sidebar">
          <div className="sidebar-header">
            <a href="/" className="sidebar-brand">aeqi</a>
            <span className="sidebar-search-btn" onClick={openSearch} title="Search (Cmd+K)">
              <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round">
                <circle cx="7" cy="7" r="4.5" />
                <path d="M10.5 10.5L14 14" />
              </svg>
            </span>
          </div>
          <div className="left-sidebar-body">
            <AgentTree />
          </div>
          <div className="left-sidebar-footer" onClick={() => navigate("/settings")}>
            <BlockAvatar name={userName} size={28} />
            <div className="left-sidebar-footer-info">
              <span className="user-profile-name">{userName}</span>
            </div>
          </div>
        </div>

        {/* Main content: Session view or Dashboard */}
        <div className="content-area">
          {agentId ? (
            <AgentSessionView agentId={agentId} sessionId={sessionId} />
          ) : (
            <div className="content-scroll">
              <DashboardHome />
            </div>
          )}
        </div>

        {/* Right context panel: visible when agent selected */}
        {agentId && <ContextPanel />}
      </div>
      <CommandPalette open={searching} onClose={closeSearch} />
    </>
  );
}
