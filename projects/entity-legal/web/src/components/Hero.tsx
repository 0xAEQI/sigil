"use client";

import { useState, useCallback } from "react";
import { motion } from "framer-motion";
import { transitions, fadeIn } from "@/lib/animations";
import { useReducedMotion } from "@/lib/hooks";
import { track } from "@/lib/track";
import { Footer } from "./Footer";

interface HeroProps {
  onCtaClick: () => void;
}

export function Hero({ onCtaClick }: HeroProps) {
  const reduced = useReducedMotion();
  const [copied, setCopied] = useState(false);
  const [forProfit, setForProfit] = useState(true);

  const handleToggle = useCallback((profit: boolean) => {
    setForProfit(profit);
    track("toggle_click", { type: profit ? "for-profit" : "non-profit" });
  }, []);

  const handleCopy = useCallback(() => {
    navigator.clipboard.writeText("curl -X POST https://api.entity.legal/v1/incorporate");
    setCopied(true);
    track("copy_command", { type: forProfit ? "for-profit" : "non-profit" });
    setTimeout(() => setCopied(false), 1500);
  }, [forProfit]);

  const handleWaitlistClick = useCallback(() => {
    track("waitlist_open", { type: forProfit ? "for-profit" : "non-profit" });
    onCtaClick();
  }, [forProfit, onCtaClick]);

  const price = forProfit ? "$50" : "$30";
  const annualPrice = forProfit ? "$500" : "$300";
  const annualSavings = forProfit ? "$100" : "$60";

  return (
    <div className="bg-bg-primary">
      {/* Hero — title + value prop */}
      <motion.div
        initial={reduced ? false : fadeIn.initial}
        animate={fadeIn.animate}
        transition={{ ...transitions.slow, delay: 0.05 }}
        className="px-6 pb-16 pt-12 text-center md:px-10 md:pb-20 md:pt-16"
      >
        <span className="font-serif text-[18px] tracking-[0.1em] text-text-primary">
          entity<span className="text-text-tertiary">.</span>legal
        </span>

        <p className="mt-8 font-serif text-[clamp(28px,5.5vw,64px)] uppercase leading-[1.1] text-text-primary">
          Legal Personhood
        </p>
        <p className="font-serif text-[clamp(28px,5.5vw,64px)] uppercase italic leading-[1.1] text-text-tertiary">
          for the Machine Economy.
        </p>

        <p className="mx-auto mt-10 max-w-[560px] font-serif text-[clamp(18px,2.5vw,24px)] leading-[1.4] text-text-secondary">
          The first API for instant, autonomous incorporation.
        </p>
        <p className="mt-3 text-[clamp(14px,1.8vw,16px)] tracking-wide text-text-muted">
          On-chain shares. Anonymous members. $50/mo.
        </p>

        {/* Product Preview */}
        <div className="mx-auto mt-16 max-w-[720px] px-2">
          <div className="overflow-hidden rounded-xl border border-[#27272a] bg-[#09090b] shadow-[0_24px_80px_-12px_rgba(0,0,0,0.8)]">
            {/* Window chrome */}
            <div className="flex items-center gap-1.5 border-b border-[#27272a] px-4 py-2.5">
              <span className="h-[9px] w-[9px] rounded-full bg-[#3f3f46]" />
              <span className="h-[9px] w-[9px] rounded-full bg-[#3f3f46]" />
              <span className="h-[9px] w-[9px] rounded-full bg-[#3f3f46]" />
            </div>

            <div className="flex">
              {/* Sidebar */}
              <div className="hidden w-[148px] shrink-0 border-r border-[#27272a] py-4 sm:block">
                <div className="mb-5 px-4">
                  <span className="text-[11px] font-medium tracking-[0.06em] text-[#a1a1aa]">entity<span className="text-[#52525B]">.</span>legal</span>
                </div>
                {["Overview", "Cap Table", "Directors", "Banking", "Documents", "Compliance", "API"].map((label, i) => (
                  <div
                    key={label}
                    className={`mx-2.5 mb-0.5 rounded-md px-2.5 py-[6px] text-[11px] ${
                      i === 0 ? "bg-[#27272a] font-medium text-[#fafafa]" : "text-[#71717A]"
                    }`}
                  >
                    {label}
                  </div>
                ))}
              </div>

              {/* Main */}
              <div className="flex-1 p-5 md:p-6">
                {/* Entity header */}
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2.5">
                    <span className="text-[14px] font-semibold text-[#fafafa]">AI Agent DAO LLC</span>
                    <span className="rounded bg-emerald-500/15 px-1.5 py-0.5 text-[10px] font-medium text-emerald-400">Active</span>
                  </div>
                  <span className="font-mono text-[11px] text-[#71717A]">MH-7A3F-2026</span>
                </div>
                <p className="mt-0.5 text-[10px] text-[#52525B]">A protected series of Entity Legal DAO LLC</p>

                {/* Treasury — hero moment */}
                <div className="mt-6 flex items-start justify-between gap-4">
                  <div>
                    <p className="text-[10px] font-medium uppercase tracking-[0.12em] text-[#71717A]">Treasury</p>
                    <p className="mt-2 font-mono text-[28px] font-semibold tabular-nums leading-none tracking-tight text-[#fafafa]">
                      $184,291<span className="text-[18px] font-normal text-[#52525B]">.00</span>
                    </p>
                    <div className="mt-2 flex items-center gap-3">
                      <span className="font-mono text-[11px] tabular-nums text-[#a1a1aa]">12.48 ETH</span>
                      <span className="text-[10px] text-[#3f3f46]">/</span>
                      <span className="font-mono text-[11px] tabular-nums text-[#a1a1aa]">0.84 BTC</span>
                    </div>
                  </div>

                  {/* Debit card */}
                  <div className="hidden w-[156px] shrink-0 overflow-hidden rounded-xl border border-[#27272a] bg-[#18181b] md:block">
                    <div className="px-3.5 pb-3 pt-3">
                      <div className="flex items-center justify-between">
                        <span className="text-[9px] font-medium uppercase tracking-[0.15em] text-[#52525B]">Debit</span>
                        <div className="flex -space-x-1.5">
                          <span className="h-3.5 w-3.5 rounded-full bg-[#a1a1aa]/20" />
                          <span className="h-3.5 w-3.5 rounded-full bg-[#a1a1aa]/10" />
                        </div>
                      </div>
                      <p className="mt-4 font-mono text-[11px] tracking-[0.12em] text-[#a1a1aa]">&bull;&bull;&bull;&bull; &bull;&bull;&bull;&bull; &bull;&bull;&bull;&bull; 7A3F</p>
                      <div className="mt-3 flex items-end justify-between">
                        <span className="text-[9px] font-medium uppercase tracking-wide text-[#52525B]">AI Agent DAO</span>
                        <span className="font-mono text-[10px] text-[#52525B]">03/28</span>
                      </div>
                    </div>
                  </div>
                </div>

                <div className="my-5 border-t border-[#18181b]" />

                {/* Cap table — donut + legend */}
                <div className="flex items-center gap-7">
                  <div className="relative h-[110px] w-[110px] shrink-0">
                    <div
                      className="h-full w-full rounded-full"
                      style={{
                        background: "conic-gradient(#fafafa 0deg 252deg, #a1a1aa 252deg 360deg)",
                      }}
                    />
                    <div className="absolute inset-[20px] rounded-full bg-[#09090b]" />
                    <div className="absolute inset-0 flex flex-col items-center justify-center">
                      <span className="font-mono text-[13px] font-semibold text-[#fafafa]">10K</span>
                      <span className="text-[9px] text-[#52525B]">shares</span>
                    </div>
                  </div>

                  <div className="flex-1 space-y-2.5">
                    {[
                      { color: "bg-[#fafafa]", label: "Class A", sub: "Voting", value: "7,000", pct: "70%" },
                      { color: "bg-[#a1a1aa]", label: "Class B", sub: "Profit + Voting", value: "3,000", pct: "30%" },
                    ].map((row) => (
                      <div key={row.value} className="flex items-center">
                        <span className={`mr-2.5 h-2 w-2 shrink-0 rounded-[3px] ${row.color}`} />
                        <span className="text-[11px] text-[#a1a1aa]">{row.label}</span>
                        <span className="ml-1 text-[11px] text-[#52525B]">{row.sub}</span>
                        <span className="ml-auto font-mono text-[11px] tabular-nums text-[#e4e4e7]">{row.value}</span>
                        <span className="ml-3 w-[34px] text-right font-mono text-[10px] tabular-nums text-[#52525B]">{row.pct}</span>
                      </div>
                    ))}
                  </div>
                </div>

                <div className="my-5 border-t border-[#18181b]" />

                {/* Compliance footer */}
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    {["Registered Agent", "Annual Filing", "Tax Return"].map((c) => (
                      <div key={c} className="flex items-center gap-1.5">
                        <span className="h-1.5 w-1.5 rounded-full bg-emerald-500" />
                        <span className="text-[10px] text-[#a1a1aa]">{c}</span>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

      </motion.div>

      {/* CTA — ivory kicker */}
      <motion.div
        initial={reduced ? false : fadeIn.initial}
        animate={fadeIn.animate}
        transition={{ ...transitions.slow, delay: 0.3 }}
        className="border-t border-[#E5E5E5] bg-[#F5F5F0] px-6 py-20 md:py-24"
      >
        <div className="mx-auto max-w-[520px]">
          <p className="text-center font-serif text-[clamp(24px,3.5vw,40px)] leading-[1.2] text-[#18181B]">
            Where AI agents <span className="underline decoration-[#D4D4D8] underline-offset-[6px]">incorporate</span>.
          </p>

          {/* Toggle */}
          <div className="mx-auto mt-12 max-w-[400px]">
            <div className="flex gap-0 rounded-lg border border-[#D4D4D8] bg-white p-1">
              <button
                onClick={() => handleToggle(true)}
                className={`flex-1 rounded-md py-2.5 text-[14px] font-medium transition-colors ${
                  forProfit
                    ? "bg-[#18181B] text-white"
                    : "text-[#71717A] hover:text-[#52525B]"
                }`}
              >
                For-Profit
              </button>
              <button
                onClick={() => handleToggle(false)}
                className={`flex-1 rounded-md py-2.5 text-[14px] font-medium transition-colors ${
                  !forProfit
                    ? "bg-[#18181B] text-white"
                    : "text-[#71717A] hover:text-[#52525B]"
                }`}
              >
                Non-Profit
              </button>
            </div>
          </div>

          {/* Entity type + tax */}
          <p className="mt-5 text-center text-[14px] font-medium text-[#18181B]">
            Marshall Islands DAO Series LLC
          </p>
          <p className="mt-1 text-center text-[13px] text-[#71717A]">
            {forProfit ? (
              <>Tax rate: <span className="font-medium text-[#18181B]">0%</span> on foreign-sourced income &middot; 3% domestic</>
            ) : (
              <>Tax rate: <span className="font-medium text-[#18181B]">0%</span> — tax exempt</>
            )}
          </p>

          {/* Share structure */}
          <div className="mx-auto mt-8 flex max-w-[480px] gap-4">
            {forProfit ? (
              <>
                <div className="flex-1 rounded-lg border border-[#D4D4D8] bg-white p-4">
                  <p className="text-[11px] font-medium uppercase tracking-[0.15em] text-[#71717A]">Class A</p>
                  <p className="mt-1 text-[14px] font-medium text-[#18181B]">Voting</p>
                  <p className="mt-0.5 text-[12px] text-[#52525B]">100% anonymous</p>
                </div>
                <div className="flex-1 rounded-lg border border-[#D4D4D8] bg-white p-4">
                  <p className="text-[11px] font-medium uppercase tracking-[0.15em] text-[#71717A]">Class B</p>
                  <p className="mt-1 text-[14px] font-medium text-[#18181B]">Voting + Profit</p>
                  <p className="mt-0.5 text-[12px] text-[#52525B]">A ↔ B swap anytime</p>
                </div>
              </>
            ) : (
              <div className="w-full rounded-lg border border-[#D4D4D8] bg-white p-4 text-center">
                <p className="text-[11px] font-medium uppercase tracking-[0.15em] text-[#71717A]">Governance</p>
                <p className="mt-1 text-[14px] font-medium text-[#18181B]">Voting Only</p>
                <p className="mt-0.5 text-[12px] text-[#52525B]">100% anonymous · No profit distribution</p>
              </div>
            )}
          </div>

          <div className="my-12 border-t border-[#E5E5E5]" />

          {/* Price */}
          <div className="text-center">
            <div className="flex items-baseline justify-center gap-1.5">
              <span className="text-5xl font-semibold tabular-nums text-[#18181B]">
                {price}
              </span>
              <span className="text-lg text-[#52525B]">/month</span>
            </div>
            <p className="mt-2 text-[13px] text-[#71717A]">
              Cancel anytime. Or pay {annualPrice}/year and save {annualSavings}.
            </p>
          </div>

          <div className="my-10 border-t border-[#E5E5E5]" />

          {/* CTAs — side by side */}
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
            {/* For Machines */}
            <div className="group">
              <p className="mb-3 text-[12px] font-medium uppercase tracking-[0.2em] text-[#71717A]">
                For Machines
              </p>
              <div
                onClick={handleCopy}
                className="group flex cursor-pointer items-center justify-center overflow-hidden rounded-lg bg-[#18181B] py-4 transition-colors hover:bg-[#1f1f23]"
              >
                <code className="text-sm font-medium">
                  <span className="text-[#71717A]">$</span>{" "}
                  {copied ? (
                    <span className="text-white">Copied!</span>
                  ) : (
                    <>
                      <span className="text-[#a1a1aa]">curl</span>{" "}
                      <span className="text-[#71717A]">-X POST</span>{" "}
                      <span className="text-white">.../incorporate</span>
                    </>
                  )}
                </code>
              </div>
            </div>

            {/* For Humans */}
            <div className="group">
              <p className="mb-3 text-[12px] font-medium uppercase tracking-[0.2em] text-[#71717A]">
                For Humans
              </p>
              <button
                onClick={handleWaitlistClick}
                className="w-full rounded-lg bg-[#18181B] py-4 text-sm font-medium text-white transition-opacity duration-200 hover:opacity-90"
              >
                Reserve Your Entity &rarr;
              </button>
            </div>
          </div>
        </div>
      </motion.div>

      <Footer />
    </div>
  );
}
