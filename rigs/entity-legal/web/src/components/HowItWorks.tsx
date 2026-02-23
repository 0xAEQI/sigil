"use client";

import { motion } from "framer-motion";
import { transitions } from "@/lib/animations";
import { useInView, useReducedMotion } from "@/lib/hooks";
import { SectionHeader } from "./SectionHeader";

interface Step {
  number: string;
  title: string;
  duration: string;
  body: string;
}

const steps: Step[] = [
  {
    number: "01",
    title: "Choose your structure",
    duration: "~5 minutes",
    body: "Select your entity type: DAO LLC, Non-profit DAO, Series LLC, or Traditional LLC. Tell us about your organization \u2014 number of members, governance model, treasury size. No legal jargon required. We translate your intent into the right structure.",
  },
  {
    number: "02",
    title: "KYC and formation documents",
    duration: "1\u20133 business days",
    body: "Founding members complete KYC verification (required by Marshall Islands law for members with 25%+ governance rights). We draft your LLC agreement with your Solana smart contract address named as the official membership registry. You review. You sign.\n\nNo surprises in the documents. We publish our template LLC agreement publicly so you can review it before you even start.",
  },
  {
    number: "03",
    title: "Entity registration + on-chain deployment",
    duration: "7\u201310 business days",
    body: "We file with the Marshall Islands Registrar of Corporations. Simultaneously, we deploy your Squads multisig and SPL token cap table on Solana mainnet. By the time your registration is confirmed, your on-chain infrastructure is already live.\n\nYour registered agent in the Marshall Islands is provisioned. Your annual compliance calendar is set. Your entity exists \u2014 both legally and on-chain.",
  },
  {
    number: "04",
    title: "You\u2019re sovereign",
    duration: "Day one",
    body: "Receive your Certificate of Formation, your executed LLC agreement, your Squads multisig address, and your SPL token mint address. Your cap table is live. Your governance is active. Your entity is real.\n\nIssue membership interests by minting tokens. Transfer ownership by transferring tokens. Govern by signing multisig transactions. Everything your entity does is verifiable, auditable, and yours.",
  },
];

function TimelineStep({ step, index }: { step: Step; index: number }) {
  const { ref, isInView } = useInView(0.3);
  const reduced = useReducedMotion();

  return (
    <motion.div
      ref={ref}
      initial={reduced ? {} : { opacity: 0, y: 20 }}
      animate={isInView ? { opacity: 1, y: 0 } : {}}
      transition={{ ...transitions.default, delay: index * 0.15 }}
      className="relative flex gap-6 pb-12 last:pb-0 md:gap-8"
    >
      {/* Timeline line and marker */}
      <div className="flex flex-col items-center">
        <motion.div
          className="flex h-10 w-10 shrink-0 items-center justify-center rounded-full border-2 text-sm font-bold md:h-12 md:w-12"
          animate={
            isInView
              ? {
                  backgroundColor: "#6C3CE1",
                  borderColor: "#6C3CE1",
                  color: "#FFFFFF",
                  boxShadow: "0 0 12px rgba(108, 60, 225, 0.4)",
                }
              : {
                  backgroundColor: "transparent",
                  borderColor: "#2A2A3A",
                  color: "#606070",
                  boxShadow: "none",
                }
          }
          transition={{ ...transitions.default, delay: index * 0.15 + 0.2 }}
        >
          {step.number}
        </motion.div>
        {index < steps.length - 1 && (
          <div className="w-px flex-1 bg-border" />
        )}
      </div>

      {/* Content */}
      <div className="flex-1 pb-4">
        <div className="flex flex-wrap items-center gap-3">
          <h3 className="text-lg font-semibold text-text-primary md:text-xl">
            {step.title}
          </h3>
          <span className="inline-block rounded-full bg-accent/10 px-3 py-1 text-xs font-medium text-accent">
            {step.duration}
          </span>
        </div>
        <div className="mt-3 space-y-3 text-[15px] leading-relaxed text-text-secondary">
          {step.body.split("\n\n").map((paragraph, i) => (
            <p key={i}>{paragraph}</p>
          ))}
        </div>
      </div>
    </motion.div>
  );
}

export function HowItWorks() {
  return (
    <section id="how-it-works" className="bg-bg-secondary px-6 py-24 md:py-[120px]">
      <div className="mx-auto max-w-[900px]">
        <SectionHeader
          eyebrow="PROCESS"
          title="From zero to sovereign in four steps."
          subtitle="Formation takes 10\u201315 business days. The on-chain infrastructure deploys the same day your entity is registered. No back-and-forth. No \u201Cwe\u2019ll get back to you.\u201D A defined process with defined timelines."
          maxWidth="700px"
        />

        <div className="mt-12">
          {steps.map((step, i) => (
            <TimelineStep key={step.number} step={step} index={i} />
          ))}
        </div>
      </div>
    </section>
  );
}
