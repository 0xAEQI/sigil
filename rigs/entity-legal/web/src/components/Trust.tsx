"use client";

import { motion } from "framer-motion";
import { transitions } from "@/lib/animations";
import { useInView, useReducedMotion } from "@/lib/hooks";
import { config } from "@/lib/config";
import { AnimatedCounter } from "./AnimatedCounter";
import { SectionHeader } from "./SectionHeader";

export function Trust() {
  const { ref, isInView } = useInView(0.2);
  const reduced = useReducedMotion();

  return (
    <section className="bg-bg-primary px-6 py-20 md:py-[100px]">
      <div className="mx-auto max-w-[1000px]">
        {/* Live Stats Bar */}
        <motion.div
          ref={ref}
          initial={reduced ? {} : { opacity: 0, y: 20 }}
          animate={isInView ? { opacity: 1, y: 0 } : {}}
          transition={transitions.default}
          className="grid grid-cols-1 gap-8 text-center sm:grid-cols-3"
        >
          <div>
            <div className="text-[48px] font-bold leading-none text-text-primary">
              <AnimatedCounter
                target={config.stats.capTableEntries}
                duration={2000}
              />
            </div>
            <p className="mt-2 text-sm text-text-secondary">
              cap table entries secured on Solana
            </p>
          </div>
          <div>
            <div className="text-[48px] font-bold leading-none text-text-primary">
              <AnimatedCounter
                target={config.stats.entitiesFormed}
                duration={2000}
              />
            </div>
            <p className="mt-2 text-sm text-text-secondary">entities formed</p>
          </div>
          <div>
            <div className="text-[48px] font-bold leading-none text-text-primary">
              <AnimatedCounter
                target={Number(config.stats.capTableValue)}
                duration={2000}
                prefix="$"
                suffix="M+"
              />
            </div>
            <p className="mt-2 text-sm text-text-secondary">
              in on-chain cap tables
            </p>
          </div>
        </motion.div>

        {/* Credibility Markers */}
        <motion.p
          initial={reduced ? {} : { opacity: 0 }}
          animate={isInView ? { opacity: 1 } : {}}
          transition={{ ...transitions.default, delay: 0.3 }}
          className="mt-12 text-center text-sm tracking-[0.5px] text-text-tertiary"
        >
          Built by Web3 natives
          <span className="mx-4">{"\u00B7"}</span>
          Marshall Islands licensed registered agent
          <span className="mx-4">{"\u00B7"}</span>
          Squads Protocol partner
          <span className="mx-4">{"\u00B7"}</span>
          Solana ecosystem
          <span className="mx-4">{"\u00B7"}</span>
          Open-source LLC templates
        </motion.p>

        {/* Testimonials (feature-flagged) */}
        {config.showTestimonials && (
          <div className="mt-16 grid gap-6 md:grid-cols-2">
            <div className="rounded-xl border border-border bg-bg-card p-8">
              <p className="text-lg italic leading-relaxed text-text-primary">
                &ldquo;entity.legal will be the first formation service we
                recommend to portfolio companies building on Solana.&rdquo;
              </p>
              <p className="mt-4 text-sm text-text-secondary">
                &mdash; [Name], [Title], [Fund/Company]
              </p>
            </div>
          </div>
        )}

        {/* Built By Section */}
        <div className="mx-auto mt-16 max-w-[700px] text-center">
          <p className="text-lg font-medium leading-relaxed text-text-primary">
            Built by a team that has formed entities, managed DAOs, and written
            smart contracts &mdash; not by a law firm that read a blog post about
            blockchain.
          </p>
          <p className="mt-4 text-[15px] leading-relaxed text-text-secondary">
            We are Web3 natives building legal infrastructure for Web3 natives.
            Our team has collectively managed over $50M in on-chain treasuries.
            We&rsquo;ve formed entities in 6 jurisdictions. We&rsquo;ve been
            burned by the alternatives, and we built entity.legal because we
            needed it ourselves.
          </p>
        </div>
      </div>
    </section>
  );
}
