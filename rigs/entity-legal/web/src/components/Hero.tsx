"use client";

import { forwardRef } from "react";
import { motion } from "framer-motion";
import { fadeUp } from "@/lib/animations";
import { config } from "@/lib/config";
import { useReducedMotion } from "@/lib/hooks";
import { ParticleField } from "./ParticleField";
import { ScrollIndicator } from "./ScrollIndicator";

interface HeroProps {
  onCtaClick: () => void;
}

export const Hero = forwardRef<HTMLElement, HeroProps>(function Hero(
  { onCtaClick },
  ref
) {
  const reduced = useReducedMotion();

  const lines = [
    "Your cap table lives on-chain.",
    "Your investors verify in seconds.",
    "Your company exists in a jurisdiction",
    "that respects your sovereignty.",
  ];

  return (
    <section
      ref={ref}
      className="relative flex h-screen items-start justify-center overflow-hidden bg-bg-primary"
    >
      {/* Background gradient */}
      <div
        className="pointer-events-none absolute inset-0"
        style={{
          background:
            "radial-gradient(ellipse at center, #1A0A2E 0%, transparent 60%)",
        }}
        aria-hidden="true"
      />

      {/* Particle field */}
      <ParticleField />

      {/* Content */}
      <div className="relative z-10 mx-auto max-w-[800px] px-6 pt-[40vh] text-center">
        {/* Eyebrow */}
        <motion.p
          initial={reduced ? false : { opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ duration: 0.6, delay: 0.2 }}
          className="text-[13px] font-medium uppercase tracking-[4px] text-accent"
        >
          MARSHALL ISLANDS DAO LLC + SOLANA
        </motion.p>

        {/* Headline */}
        <h1 className="mt-6 text-[32px] font-bold leading-[1.2] tracking-tight text-text-primary md:text-[48px]">
          {lines.map((line, i) => (
            <motion.span
              key={i}
              className="block"
              initial={reduced ? false : fadeUp.initial}
              animate={fadeUp.animate}
              transition={{
                duration: 0.6,
                delay: 0.4 + i * 0.1,
                ease: [0.16, 1, 0.3, 1],
              }}
            >
              {line}
            </motion.span>
          ))}
        </h1>

        {/* Sub-headline */}
        <motion.p
          initial={reduced ? false : fadeUp.initial}
          animate={fadeUp.animate}
          transition={{ duration: 0.4, delay: 0.9 }}
          className="mx-auto mt-6 max-w-[600px] text-base leading-relaxed text-text-secondary md:text-lg"
        >
          entity.legal forms Marshall Islands DAO LLCs with on-chain cap
          tables on Solana. Real legal structure. No US regulatory overhead.
          Your keys, your entity.
        </motion.p>

        {/* CTA Group */}
        <motion.div
          initial={reduced ? false : fadeUp.initial}
          animate={fadeUp.animate}
          transition={{ duration: 0.4, delay: 1.1 }}
          className="mt-10 flex flex-col items-center gap-4 sm:flex-row sm:justify-center"
        >
          <button
            onClick={onCtaClick}
            className="rounded-lg bg-accent px-8 py-4 text-base font-semibold text-white transition-colors duration-200 hover:bg-accent-hover"
          >
            Form Your Entity {"\u2192"}
          </button>
          <a
            href="#how-it-works"
            className="text-sm font-medium text-accent transition-colors hover:underline"
          >
            See How It Works {"\u2193"}
          </a>
        </motion.div>

        {/* Social Proof Line */}
        <motion.p
          initial={reduced ? false : { opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ duration: 0.3, delay: 1.3 }}
          className="mt-8 text-[13px] tracking-[0.5px] text-text-tertiary"
        >
          {config.stats.entitiesFormed} entities formed
          <span className="mx-3">{"\u00B7"}</span>$
          {config.stats.capTableValue}M+ in on-chain cap tables
          <span className="mx-3">{"\u00B7"}</span>
          {config.stats.custodyIncidents} custody incidents
        </motion.p>
      </div>

      {/* Scroll Indicator */}
      <ScrollIndicator />
    </section>
  );
});
