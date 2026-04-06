import { useState, useEffect, useRef } from "react";
import { Link, useLocation } from "react-router-dom";

interface TOCItem {
  id: string;
  text: string;
  level: number;
}

interface NavItem {
  title: string;
  path: string;
  children?: NavItem[];
}

const NAV: NavItem[] = [
  {
    title: "Getting Started",
    path: "/docs",
    children: [
      { title: "Introduction", path: "/docs" },
      { title: "Quickstart", path: "/docs/quickstart" },
      { title: "Installation", path: "/docs/installation" },
    ],
  },
  {
    title: "Core Concepts",
    path: "/docs/concepts",
    children: [
      { title: "Agents", path: "/docs/concepts/agents" },
      { title: "Quests", path: "/docs/concepts/quests" },
      { title: "Memory", path: "/docs/concepts/memory" },
      { title: "Companies", path: "/docs/concepts/companies" },
    ],
  },
  {
    title: "Platform",
    path: "/docs/platform",
    children: [
      { title: "Dashboard", path: "/docs/platform/dashboard" },
      { title: "Sessions", path: "/docs/platform/sessions" },
      { title: "MCP Integration", path: "/docs/platform/mcp" },
    ],
  },
  {
    title: "Self-Hosting",
    path: "/docs/self-hosting",
    children: [
      { title: "Configuration", path: "/docs/self-hosting/configuration" },
      { title: "Deployment", path: "/docs/self-hosting/deployment" },
    ],
  },
];

function NavTree({ items, location }: { items: NavItem[]; location: string }) {
  return (
    <div className="space-y-6">
      {items.map((section) => (
        <div key={section.path}>
          <div className="text-[11px] font-medium uppercase tracking-[0.1em] text-black/30 mb-2">
            {section.title}
          </div>
          <div className="space-y-0.5">
            {section.children?.map((item) => (
              <Link
                key={item.path}
                to={item.path}
                className={`block text-[13px] py-1.5 px-3 rounded-lg transition-colors ${
                  location === item.path
                    ? "text-black/85 bg-black/[0.04] font-medium"
                    : "text-black/45 hover:text-black/70 hover:bg-black/[0.02]"
                }`}
              >
                {item.title}
              </Link>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}

function Minimap({ items, activeId }: { items: TOCItem[]; activeId: string }) {
  if (items.length === 0) return null;
  return (
    <div className="space-y-1">
      <div className="text-[11px] font-medium uppercase tracking-[0.1em] text-black/25 mb-3">
        On this page
      </div>
      {items.map((item) => (
        <a
          key={item.id}
          href={`#${item.id}`}
          className={`block text-[12px] py-0.5 transition-colors ${
            item.level > 2 ? "pl-3" : ""
          } ${
            activeId === item.id
              ? "text-black/70 font-medium"
              : "text-black/30 hover:text-black/50"
          }`}
        >
          {item.text}
        </a>
      ))}
    </div>
  );
}

export default function DocsLayout({ children }: { children: React.ReactNode }) {
  const location = useLocation();
  const contentRef = useRef<HTMLDivElement>(null);
  const [toc, setToc] = useState<TOCItem[]>([]);
  const [activeId, setActiveId] = useState("");

  // Extract TOC from rendered content
  useEffect(() => {
    if (!contentRef.current) return;
    const headings = contentRef.current.querySelectorAll("h2, h3");
    const items: TOCItem[] = Array.from(headings).map((h) => ({
      id: h.id || h.textContent?.toLowerCase().replace(/\s+/g, "-").replace(/[^\w-]/g, "") || "",
      text: h.textContent || "",
      level: parseInt(h.tagName[1]),
    }));
    // Set IDs on headings that don't have them
    headings.forEach((h, i) => {
      if (!h.id && items[i]) h.id = items[i].id;
    });
    setToc(items);
  }, [location.pathname, children]);

  // Track active heading on scroll
  useEffect(() => {
    if (toc.length === 0) return;
    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            setActiveId(entry.target.id);
          }
        }
      },
      { rootMargin: "-80px 0px -60% 0px" }
    );
    toc.forEach((item) => {
      const el = document.getElementById(item.id);
      if (el) observer.observe(el);
    });
    return () => observer.disconnect();
  }, [toc]);

  return (
    <div className="min-h-screen bg-white">
      {/* Header */}
      <header className="sticky top-0 z-50 bg-white/80 backdrop-blur-xl border-b border-black/[0.06]">
        <div className="max-w-7xl mx-auto px-6 h-14 flex items-center justify-between">
          <div className="flex items-center gap-6">
            <Link to="/" className="text-[18px] font-bold tracking-[-0.08em] text-black/70 hover:text-black/90 transition-colors">
              æq<span className="inline-block translate-y-[0.04em]">i</span>
            </Link>
            <span className="text-[13px] text-black/25 font-medium">Docs</span>
          </div>
          <div className="flex items-center gap-3">
            <a href="https://github.com/0xAEQI/aeqi" target="_blank" rel="noopener noreferrer" className="text-[13px] text-black/40 hover:text-black/70 transition-colors">
              GitHub
            </a>
            <a href="https://app.aeqi.ai" className="text-[13px] bg-black text-white rounded-lg px-3 py-1.5 font-medium hover:bg-black/85 transition-colors">
              Dashboard
            </a>
          </div>
        </div>
      </header>

      <div className="max-w-7xl mx-auto flex">
        {/* Left sidebar */}
        <aside className="w-56 flex-shrink-0 border-r border-black/[0.04] py-8 px-4 sticky top-14 h-[calc(100vh-56px)] overflow-y-auto hidden lg:block">
          <NavTree items={NAV} location={location.pathname} />
        </aside>

        {/* Content */}
        <main ref={contentRef} className="flex-1 min-w-0 py-10 px-8 lg:px-16 max-w-3xl">
          <article className="docs-content">
            {children}
          </article>
        </main>

        {/* Right minimap */}
        <aside className="w-48 flex-shrink-0 py-10 px-4 sticky top-14 h-[calc(100vh-56px)] overflow-y-auto hidden xl:block">
          <Minimap items={toc} activeId={activeId} />
        </aside>
      </div>
    </div>
  );
}
