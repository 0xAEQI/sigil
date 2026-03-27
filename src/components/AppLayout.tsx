import { useState, useEffect, useCallback, useRef } from "react";
import { useLocation, Link } from "react-router-dom";
import Sidebar from "./Sidebar";
import CommandPalette from "./CommandPalette";
import ContextPanel from "./ContextPanel";
import { useUIStore, type LayoutMode } from "@/store/ui";
import { useChatStore } from "@/store/chat";

// ── Draggable divider ──
function ResizeHandle() {
  const setSplitRatio = useUIStore((s) => s.setSplitRatio);
  const panelsRef = useRef<HTMLElement | null>(null);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    const panels = (e.target as HTMLElement).parentElement;
    if (!panels) return;
    panelsRef.current = panels;

    const onMouseMove = (ev: MouseEvent) => {
      if (!panelsRef.current) return;
      const rect = panelsRef.current.getBoundingClientRect();
      const ratio = (ev.clientX - rect.left) / rect.width;
      setSplitRatio(ratio);
    };

    const onMouseUp = () => {
      document.removeEventListener("mousemove", onMouseMove);
      document.removeEventListener("mouseup", onMouseUp);
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    };

    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
    document.addEventListener("mousemove", onMouseMove);
    document.addEventListener("mouseup", onMouseUp);
  }, [setSplitRatio]);

  return <div className="resize-handle" onMouseDown={handleMouseDown} />;
}

// ── Layout picker ──
function LayoutPicker() {
  const layout = useUIStore((s) => s.layout);
  const setLayout = useUIStore((s) => s.setLayout);
  const pickerOpen = useUIStore((s) => s.layoutPickerOpen);
  const togglePicker = useUIStore((s) => s.toggleLayoutPicker);
  const closePicker = useUIStore((s) => s.closeLayoutPicker);

  const options: { mode: LayoutMode; label: string; icon: React.ReactNode }[] = [
    { mode: "focus", label: "Focus", icon: <svg width="16" height="11" viewBox="0 0 16 11" fill="none" stroke="currentColor" strokeWidth="1"><rect x="0.5" y="0.5" width="15" height="10" rx="1" /></svg> },
    { mode: "split", label: "Split", icon: <svg width="16" height="11" viewBox="0 0 16 11" fill="none" stroke="currentColor" strokeWidth="1"><rect x="0.5" y="0.5" width="15" height="10" rx="1" /><path d="M11 0.5v10" /></svg> },
    { mode: "stack", label: "Stack", icon: <svg width="16" height="11" viewBox="0 0 16 11" fill="none" stroke="currentColor" strokeWidth="1"><rect x="0.5" y="0.5" width="15" height="10" rx="1" /><path d="M0.5 6h15" /></svg> },
  ];

  return (
    <div className="layout-picker-wrapper">
      <button className="layout-picker-trigger" onClick={togglePicker} title="Layout">
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
          <rect x="1" y="1" width="12" height="12" rx="1" />
          <path d="M6 1v12" />
        </svg>
      </button>
      {pickerOpen && (
        <>
          <div className="layout-picker-backdrop" onClick={closePicker} />
          <div className="layout-picker-dropdown layout-picker-dropdown-down">
            {options.map((opt) => (
              <button key={opt.mode} className={`layout-option ${layout === opt.mode ? "layout-option-active" : ""}`} onClick={() => setLayout(opt.mode)}>
                {opt.icon}
                <span>{opt.label}</span>
              </button>
            ))}
          </div>
        </>
      )}
    </div>
  );
}

// ── Breadcrumbs ──
function Breadcrumbs() {
  const { pathname } = useLocation();
  if (pathname === "/login") return null;
  if (pathname === "/") return <ChatBreadcrumb />;

  const segments = pathname.split("/").filter(Boolean);
  const crumbs: { label: string; href: string }[] = [];
  for (let i = 0; i < segments.length; i++) {
    crumbs.push({ label: segments[i], href: "/" + segments.slice(0, i + 1).join("/") });
  }

  return (
    <div className="topbar">
      <Link to="/" className="breadcrumb-item">sigil</Link>
      {crumbs.map((crumb, i) => (
        <span key={crumb.href} className="breadcrumb-segment">
          <span className="breadcrumb-sep">/</span>
          {i === crumbs.length - 1 ? (
            <span className="breadcrumb-current">{crumb.label}</span>
          ) : (
            <Link to={crumb.href} className="breadcrumb-item">{crumb.label}</Link>
          )}
        </span>
      ))}
      <div className="topbar-right">
        <LayoutPicker />
      </div>
    </div>
  );
}

function ChatBreadcrumb() {
  const channel = useChatStore((s) => s.channel);
  return (
    <div className="topbar">
      <span className={channel ? "breadcrumb-item" : "breadcrumb-current"} style={{ cursor: "default" }}>sigil</span>
      {channel && (
        <span className="breadcrumb-segment">
          <span className="breadcrumb-sep">/</span>
          <span className="breadcrumb-current">{channel.includes("/") ? channel.split("/").pop() : channel}</span>
        </span>
      )}
      <div className="topbar-right">
        <LayoutPicker />
      </div>
    </div>
  );
}

// ── Main layout ──
export default function AppLayout({ children }: { children: React.ReactNode }) {
  const [paletteOpen, setPaletteOpen] = useState(false);
  const { pathname } = useLocation();
  const collapsed = useUIStore((s) => s.sidebarCollapsed);
  const layout = useUIStore((s) => s.layout);
  const splitRatio = useUIStore((s) => s.splitRatio);
  const isChatHome = pathname === "/";

  const openPalette = useCallback(() => setPaletteOpen(true), []);
  const closePalette = useCallback(() => setPaletteOpen(false), []);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") { e.preventDefault(); setPaletteOpen((p) => !p); }
      if ((e.metaKey || e.ctrlKey) && e.key === "b") { e.preventDefault(); useUIStore.getState().toggleSidebar(); }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  const showContext = isChatHome && layout !== "focus";
  const isStack = layout === "stack";
  const isSplit = layout === "split" && isChatHome;

  const panelStyle = isSplit ? {
    "--split-main": `${splitRatio * 100}%`,
    "--split-ctx": `${(1 - splitRatio) * 100}%`,
  } as React.CSSProperties : undefined;

  return (
    <div className={`app-shell ${collapsed ? "app-shell-collapsed" : ""}`}>
      <div className="app-layout">
        <Sidebar onCommandPalette={openPalette} />
        <div className={`main-wrapper ${isStack && isChatHome ? "main-wrapper-stack" : ""}`}>
          <Breadcrumbs />
          <div
            className={`main-panels ${isStack && isChatHome ? "main-panels-stack" : ""} ${isSplit ? "main-panels-split" : ""}`}
            style={panelStyle}
          >
            <main className={`main-content ${isChatHome ? "main-content-chat" : ""}`}>
              {children}
            </main>
            {showContext && (
              <>
                {isSplit && <ResizeHandle />}
                <ContextPanel />
              </>
            )}
          </div>
        </div>
      </div>
      <CommandPalette open={paletteOpen} onClose={closePalette} />
    </div>
  );
}
