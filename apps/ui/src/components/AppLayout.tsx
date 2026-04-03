import { useState, useEffect, useCallback, useRef } from "react";
import { Outlet, NavLink, useNavigate } from "react-router-dom";
import CompanyRail from "./CompanyRail";
import AgentNav from "./Sidebar";
import CompanyPatternIcon from "./CompanyPatternIcon";
import { useChatStore } from "@/store/chat";

const NAV_ITEMS = [
  { to: "/", label: "home", end: true },
  { to: "/sessions", label: "sessions" },
  { to: "/issues", label: "tasks" },
  { to: "/triggers", label: "triggers" },
  { to: "/skills", label: "skills" },
  { to: "/memories", label: "memories" },
  { to: "/notes", label: "notes" },
];

export default function AppLayout() {
  const navigate = useNavigate();
  const channel = useChatStore((s) => s.channel);
  const setChannel = useChatStore((s) => s.setChannel);
  const [searching, setSearching] = useState(false);
  const [query, setQuery] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);

  const openSearch = useCallback(() => {
    setSearching(true);
    setQuery("");
    setTimeout(() => inputRef.current?.focus(), 50);
  }, []);

  const closeSearch = useCallback(() => {
    setSearching(false);
    setQuery("");
  }, []);

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
    <div className="shell">
      <div className="left-sidebar">
        <div className="left-sidebar-bar">
          <div
            className={`rail-icon rail-home${!channel ? " active" : ""}`}
            onClick={() => { setChannel(null); navigate("/"); }}
            title="AEQI"
          >Æ</div>
          {channel && (
            <div className="left-sidebar-company" onClick={() => navigate("/")}>
              <CompanyPatternIcon name={channel} selected />
              <span className="scope-header-text">{channel}</span>
            </div>
          )}
        </div>
        <div className="left-sidebar-body">
          <CompanyRail />
          <AgentNav />
        </div>
      </div>
      <div className="content-area">
        <div className="content-scroll">
          <div className="floating-nav">
            {searching ? (
              <>
                <span className="floating-nav-btn" onClick={closeSearch}>
                  <svg width="15" height="15" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round">
                    <circle cx="7" cy="7" r="4.5" />
                    <path d="M10.5 10.5L14 14" />
                  </svg>
                </span>
                <input
                  ref={inputRef}
                  className="floating-nav-search"
                  type="text"
                  placeholder="Search agents, tasks, memory..."
                  value={query}
                  onChange={(e) => setQuery(e.target.value)}
                  onKeyDown={(e) => e.key === "Escape" && closeSearch()}
                />
              </>
            ) : (
              <>
                <span className="floating-nav-btn" onClick={openSearch} title="Search (⌘K)">
                  <svg width="15" height="15" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round">
                    <circle cx="7" cy="7" r="4.5" />
                    <path d="M10.5 10.5L14 14" />
                  </svg>
                </span>
                <div className="floating-nav-items">
                  {NAV_ITEMS.map((item) => (
                    <NavLink
                      key={item.to}
                      to={item.to}
                      end={item.end}
                      className={({ isActive }) =>
                        `floating-nav-item${isActive ? " active" : ""}`
                      }
                    >
                      {item.label}
                    </NavLink>
                  ))}
                </div>
              </>
            )}
            <span className="floating-nav-btn" title="Notifications">
              <svg width="15" height="15" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round">
                <path d="M4 6a4 4 0 0 1 8 0c0 3 1.5 4.5 2 5H2c.5-.5 2-2 2-5z" />
                <path d="M6.5 13a1.5 1.5 0 0 0 3 0" />
              </svg>
            </span>
          </div>

          <div className="content-panel">
            <Outlet />
          </div>
        </div>
      </div>
    </div>
  );
}
