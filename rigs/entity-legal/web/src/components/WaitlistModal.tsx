"use client";

import { useState, useEffect, useRef } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { X } from "lucide-react";
import { transitions } from "@/lib/animations";
import { config } from "@/lib/config";
import { AnimatedCounter } from "./AnimatedCounter";

interface WaitlistModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export function WaitlistModal({ isOpen, onClose }: WaitlistModalProps) {
  const [formState, setFormState] = useState<"form" | "submitting" | "success">(
    "form"
  );
  const [email, setEmail] = useState("");
  const [walletAddress, setWalletAddress] = useState("");
  const [entityType, setEntityType] = useState("");
  const [teamSize, setTeamSize] = useState("");
  const [honeypot, setHoneypot] = useState("");
  const formOpenedAt = useRef(0);

  useEffect(() => {
    if (isOpen) {
      formOpenedAt.current = Date.now();
      document.body.style.overflow = "hidden";
    } else {
      document.body.style.overflow = "";
    }
    return () => {
      document.body.style.overflow = "";
    };
  }, [isOpen]);

  // Close on Escape
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    if (isOpen) {
      window.addEventListener("keydown", handler);
      return () => window.removeEventListener("keydown", handler);
    }
  }, [isOpen, onClose]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    // Honeypot check
    if (honeypot) return;

    // Bot speed check (< 2 seconds)
    if (Date.now() - formOpenedAt.current < 2000) return;

    setFormState("submitting");

    // TODO: Submit to API endpoint
    // For now, simulate success
    await new Promise((resolve) => setTimeout(resolve, 800));
    setFormState("success");
  };

  const inputClass =
    "w-full rounded-lg border border-border bg-bg-primary px-4 py-3.5 text-sm text-text-primary placeholder:text-text-muted focus:border-accent focus:outline-none transition-colors";

  const selectClass =
    "w-full rounded-lg border border-border bg-bg-primary px-4 py-3.5 text-sm text-text-primary focus:border-accent focus:outline-none transition-colors appearance-none cursor-pointer";

  return (
    <AnimatePresence>
      {isOpen && (
        <>
          {/* Overlay */}
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.2 }}
            className="fixed inset-0 z-50 bg-[#0A0A0F]/80 backdrop-blur-[8px]"
            onClick={onClose}
            aria-hidden="true"
          />

          {/* Modal */}
          <motion.div
            initial={{ opacity: 0, scale: 0.95 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0.95 }}
            transition={transitions.default}
            className="fixed inset-0 z-50 flex items-center justify-center p-4"
            role="dialog"
            aria-modal="true"
            aria-label="Join the waitlist"
          >
            <div
              className="relative w-full max-w-[480px] rounded-2xl border border-border bg-bg-card p-8 md:p-10"
              onClick={(e) => e.stopPropagation()}
            >
              {/* Close button */}
              <button
                onClick={onClose}
                className="absolute right-4 top-4 p-1 text-text-tertiary transition-colors hover:text-text-primary"
                aria-label="Close"
              >
                <X className="h-5 w-5" />
              </button>

              {formState === "success" ? (
                /* Success state */
                <div className="text-center">
                  <h2 className="text-2xl font-bold text-text-primary">
                    You&rsquo;re on the list.
                  </h2>
                  <p className="mt-3 text-[15px] leading-relaxed text-text-secondary">
                    We&rsquo;ll reach out when formation opens. In the meantime,
                    follow us on Twitter for updates on Marshall Islands DAO law
                    and on-chain governance.
                  </p>
                  <div className="mt-8 flex justify-center gap-4">
                    <a
                      href={config.social.twitter}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="rounded-lg border border-border px-6 py-2.5 text-sm font-medium text-text-primary transition-colors hover:border-accent hover:text-accent"
                    >
                      Follow on X
                    </a>
                    <a
                      href={config.social.discord}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="rounded-lg border border-border px-6 py-2.5 text-sm font-medium text-text-primary transition-colors hover:border-accent hover:text-accent"
                    >
                      Join Discord
                    </a>
                  </div>
                </div>
              ) : (
                /* Form state */
                <>
                  <h2 className="text-2xl font-bold text-text-primary">
                    Reserve your on-chain entity
                  </h2>
                  <p className="mt-2 text-[15px] leading-relaxed text-text-secondary">
                    Join the waitlist. We&rsquo;ll notify you when formation
                    opens. Early waitlist members get priority access and a
                    founder discount.
                  </p>

                  <form onSubmit={handleSubmit} className="mt-8 space-y-5">
                    {/* Honeypot field (hidden) */}
                    <div className="absolute -left-[9999px]" aria-hidden="true">
                      <label htmlFor="website">Website</label>
                      <input
                        type="text"
                        id="website"
                        name="website"
                        tabIndex={-1}
                        autoComplete="off"
                        value={honeypot}
                        onChange={(e) => setHoneypot(e.target.value)}
                      />
                    </div>

                    {/* Email */}
                    <div>
                      <label
                        htmlFor="email"
                        className="mb-2 block text-sm font-medium text-text-secondary"
                      >
                        Email address
                      </label>
                      <input
                        type="email"
                        id="email"
                        value={email}
                        onChange={(e) => setEmail(e.target.value)}
                        placeholder="founder@yourdao.xyz"
                        required
                        className={inputClass}
                      />
                    </div>

                    {/* Wallet Address */}
                    <div>
                      <label
                        htmlFor="wallet"
                        className="mb-2 block text-sm font-medium text-text-secondary"
                      >
                        Solana wallet address
                      </label>
                      <input
                        type="text"
                        id="wallet"
                        value={walletAddress}
                        onChange={(e) => setWalletAddress(e.target.value)}
                        placeholder="Connect wallet or paste address"
                        pattern="[1-9A-HJ-NP-Za-km-z]{32,44}"
                        className={inputClass}
                      />
                      <p className="mt-1.5 text-xs text-text-muted">
                        Optional &mdash; connecting your wallet reserves your
                        cap table slot
                      </p>
                    </div>

                    {/* Entity Type */}
                    <div>
                      <label
                        htmlFor="entityType"
                        className="mb-2 block text-sm font-medium text-text-secondary"
                      >
                        What are you forming?
                      </label>
                      <select
                        id="entityType"
                        value={entityType}
                        onChange={(e) => setEntityType(e.target.value)}
                        className={selectClass}
                      >
                        <option value="">Select entity type</option>
                        <option value="dao-llc">DAO LLC</option>
                        <option value="non-profit-dao">Non-Profit DAO</option>
                        <option value="series-llc">Series LLC</option>
                        <option value="traditional-llc">Traditional LLC</option>
                        <option value="not-sure">Not sure yet</option>
                      </select>
                    </div>

                    {/* Team Size */}
                    <div>
                      <label
                        htmlFor="teamSize"
                        className="mb-2 block text-sm font-medium text-text-secondary"
                      >
                        How many founding members?
                      </label>
                      <select
                        id="teamSize"
                        value={teamSize}
                        onChange={(e) => setTeamSize(e.target.value)}
                        className={selectClass}
                      >
                        <option value="">Select team size</option>
                        <option value="1">Just me</option>
                        <option value="2-3">2-3</option>
                        <option value="4-10">4-10</option>
                        <option value="11-50">11-50</option>
                        <option value="50+">50+</option>
                      </select>
                    </div>

                    {/* Submit */}
                    <button
                      type="submit"
                      disabled={formState === "submitting"}
                      className="w-full rounded-lg bg-accent py-4 text-base font-semibold text-white transition-colors hover:bg-accent-hover disabled:opacity-50"
                    >
                      {formState === "submitting"
                        ? "Joining..."
                        : "Join Waitlist \u2192"}
                    </button>

                    {/* Live counter */}
                    <p className="text-center text-[13px] text-text-tertiary">
                      <AnimatedCounter
                        target={config.stats.waitlistCount}
                        duration={1500}
                      />{" "}
                      founders already on the waitlist
                    </p>
                  </form>
                </>
              )}
            </div>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  );
}
