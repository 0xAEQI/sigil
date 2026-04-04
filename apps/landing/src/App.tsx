import { useState, useEffect, useRef } from "react";
import { Hero } from "./components/Hero";
import { AgentTree } from "./components/AgentTree";
import { Process } from "./components/Process";
import { CallToAction } from "./components/CallToAction";
import { VerticalLines } from "./components/VerticalLines";
import { motion } from "framer-motion";

function GitHubStars() {
  const [stars, setStars] = useState<number | null>(null);

  useEffect(() => {
    fetch("https://api.github.com/repos/0xAEQI/aeqi")
      .then((r) => r.json())
      .then((d) => {
        if (typeof d.stargazers_count === "number") setStars(d.stargazers_count);
      })
      .catch(() => {});
  }, []);

  return (
    <a
      href="https://github.com/0xAEQI/aeqi"
      className="flex items-center gap-2 text-white/35 hover:text-white/60 transition-colors"
    >
      <svg viewBox="0 0 16 16" fill="currentColor" className="w-[14px] h-[14px]">
        <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0016 8c0-4.42-3.58-8-8-8z" />
      </svg>
      <span className="text-[12px]">Star</span>
      {stars !== null && (
        <span className="text-[11px] bg-white/[0.06] px-1.5 py-0.5 rounded">
          {stars}
        </span>
      )}
    </a>
  );
}

function ToriiGate() {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      className="w-[20px] h-[20px]"
      stroke="#c0392b"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d="M2 6 C2 4.5, 12 3, 12 3 C12 3, 22 4.5, 22 6" />
      <line x1="4" y1="9" x2="20" y2="9" />
      <line x1="6" y1="6" x2="6" y2="22" />
      <line x1="18" y1="6" x2="18" y2="22" />
    </svg>
  );
}

function Nav() {
  return (
    <motion.nav
      className="fixed top-5 left-1/2 -translate-x-1/2 z-50"
      initial={{ opacity: 0, y: -10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.6, delay: 0.4, ease: "easeOut" }}
    >
      <div
        className="backdrop-blur-2xl bg-white/[0.03] border border-white/[0.07] rounded-full px-5 py-2.5 flex items-center gap-5"
        style={{ fontFamily: "'Space Grotesk', sans-serif" }}
      >
        <a
          href="/"
          className="flex items-center gap-2 hover:opacity-80 transition-opacity"
        >
          <ToriiGate />
          <span className="text-[14px] font-bold tracking-[0.06em] text-white">
            &#xC6;QI
          </span>
        </a>
        <div className="w-px h-3.5 bg-white/[0.08]" />
        <GitHubStars />
        <a
          href="https://github.com/0xAEQI/aeqi/blob/main/docs/architecture.md"
          className="text-[12px] text-white/35 hover:text-white/60 transition-colors hidden sm:block"
        >
          Docs
        </a>
        <a
          href="https://app.aeqi.ai"
          className="bg-white text-[#06060E] rounded-full px-4 py-1.5 text-[12px] font-semibold hover:bg-white/90 transition-colors"
          style={{ fontFamily: "'Space Grotesk', sans-serif" }}
        >
          Enter
        </a>
      </div>
    </motion.nav>
  );
}

function Backdrop() {
  return (
    <>
      <div
        className="fixed inset-0 z-0 bg-cover bg-center bg-no-repeat"
        style={{
          backgroundImage: "url('/bg.jpg')",
          filter: "blur(8px) saturate(0.4) brightness(0.45)",
          transform: "scale(1.03)",
        }}
      />
      <div
        className="fixed inset-0 z-0"
        style={{ background: "rgba(6, 6, 18, 0.35)", mixBlendMode: "multiply" }}
      />
      <div
        className="fixed inset-0 z-0"
        style={{
          background: "radial-gradient(ellipse 70% 60% at 50% 45%, transparent 0%, rgba(6,6,18,0.7) 100%)",
        }}
      />
      <div
        className="fixed inset-0 z-0"
        style={{
          background: "linear-gradient(to bottom, rgba(6,6,18,0.7) 0%, transparent 25%)",
        }}
      />
    </>
  );
}

/**
 * Cursor reveal — a clear version of bg.jpg masked to a radial gradient
 * that follows the mouse. Sits BELOW the vertical lines canvas (z-[1] vs z-2),
 * so the lines occlude the reveal and you only see the image between the lines.
 */
function CursorReveal() {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;

    const update = (e: MouseEvent) => {
      const mask = `radial-gradient(circle 220px at ${e.clientX}px ${e.clientY}px, rgba(0,0,0,0.85) 0%, rgba(0,0,0,0.4) 40%, transparent 100%)`;
      el.style.maskImage = mask;
      el.style.webkitMaskImage = mask;
    };

    const hide = () => {
      el.style.maskImage = "radial-gradient(circle 220px at -1000px -1000px, black 0%, transparent 100%)";
      el.style.webkitMaskImage = el.style.maskImage;
    };

    window.addEventListener("mousemove", update);
    window.addEventListener("mouseleave", hide);
    return () => {
      window.removeEventListener("mousemove", update);
      window.removeEventListener("mouseleave", hide);
    };
  }, []);

  return (
    <div
      ref={ref}
      className="fixed inset-0 z-[1] bg-cover bg-center bg-no-repeat pointer-events-none"
      style={{
        backgroundImage: "url('/bg.jpg')",
        filter: "saturate(0.55) brightness(0.65)",
        transform: "scale(1.03)",
        maskImage: "radial-gradient(circle 220px at -1000px -1000px, black 0%, transparent 100%)",
        WebkitMaskImage: "radial-gradient(circle 220px at -1000px -1000px, black 0%, transparent 100%)",
      }}
    />
  );
}

export default function App() {
  return (
    <div className="relative min-h-screen">
      <Backdrop />
      <CursorReveal />
      <VerticalLines />
      <Nav />
      <Hero />
      <AgentTree />
      <Process />
      <CallToAction />
    </div>
  );
}
