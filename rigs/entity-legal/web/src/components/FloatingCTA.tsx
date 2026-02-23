"use client";

import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { transitions } from "@/lib/animations";
import { useReducedMotion } from "@/lib/hooks";

interface FloatingCTAProps {
  visible: boolean;
  onCtaClick: () => void;
}

export function FloatingCTA({ visible, onCtaClick }: FloatingCTAProps) {
  const [email, setEmail] = useState("");
  const reduced = useReducedMotion();

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    // Trigger the main waitlist modal with prefilled email
    onCtaClick();
  };

  return (
    <AnimatePresence>
      {visible && (
        <>
          {/* Desktop: top bar */}
          <motion.div
            initial={reduced ? {} : { y: -48, opacity: 0 }}
            animate={{ y: 0, opacity: 1 }}
            exit={{ y: -48, opacity: 0 }}
            transition={transitions.default}
            className="fixed left-0 right-0 top-0 z-40 hidden border-b border-border/50 bg-bg-card/80 backdrop-blur-[12px] md:block"
          >
            <div className="mx-auto flex h-12 max-w-[800px] items-center justify-center gap-4 px-4">
              <span className="text-sm text-text-secondary">
                Reserve your on-chain entity
              </span>
              <span className="text-text-muted">{"\u00B7"}</span>
              <form onSubmit={handleSubmit} className="flex">
                <input
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="your@email.com"
                  className="h-8 w-48 rounded-l border border-border bg-bg-primary px-3 text-xs text-text-primary placeholder:text-text-muted focus:border-accent focus:outline-none"
                />
                <button
                  type="submit"
                  className="h-8 rounded-r bg-accent px-4 text-xs font-semibold text-white transition-colors hover:bg-accent-hover"
                >
                  Submit
                </button>
              </form>
            </div>
          </motion.div>

          {/* Mobile: bottom bar */}
          <motion.div
            initial={reduced ? {} : { y: 100, opacity: 0 }}
            animate={{ y: 0, opacity: 1 }}
            exit={{ y: 100, opacity: 0 }}
            transition={transitions.default}
            className="fixed bottom-0 left-0 right-0 z-40 border-t border-border/50 bg-bg-card/90 p-3 backdrop-blur-[12px] md:hidden"
          >
            <p className="mb-2 text-center text-xs text-text-secondary">
              Reserve your on-chain entity
            </p>
            <button
              onClick={onCtaClick}
              className="w-full rounded-lg bg-accent py-3 text-sm font-semibold text-white transition-colors hover:bg-accent-hover"
            >
              Join Waitlist {"\u2192"}
            </button>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  );
}
