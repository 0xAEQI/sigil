"use client";

import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Scale, Landmark, Link } from "lucide-react";
import { transitions } from "@/lib/animations";
import { useInView, useReducedMotion } from "@/lib/hooks";
import { SectionHeader } from "./SectionHeader";

interface Layer {
  id: number;
  icon: React.ReactNode;
  label: string;
  title: string;
  accentColor: string;
  body: string;
}

const layers: Layer[] = [
  {
    id: 3,
    icon: <Scale className="h-5 w-5" />,
    label: "LEGAL ENTITY",
    title: "Marshall Islands DAO LLC",
    accentColor: "#E8C547",
    body: "Registered under the DAO Act 2022. Recognized legal personhood. Limited liability. No US nexus. No securities registration. Your smart contract address IS your membership registry \u2014 written into the LLC agreement.",
  },
  {
    id: 2,
    icon: <Landmark className="h-5 w-5" />,
    label: "DAO GOVERNANCE",
    title: "Squads Protocol Multisig",
    accentColor: "#6C3CE1",
    body: "M-of-N multisig via Squads v4. Programmable thresholds. Time locks. Spending limits. Role-based access. Every governance action is a signed Solana transaction \u2014 auditable, immutable, on-chain.",
  },
  {
    id: 1,
    icon: <Link className="h-5 w-5" />,
    label: "SOLANA",
    title: "On-Chain Cap Table & Treasury",
    accentColor: "#14F195",
    body: "SPL token membership interests. Real-time cap table. Sub-second finality. $0.001 per transaction. Your cap table is not a spreadsheet \u2014 it\u2019s a token account on the fastest L1 in production.",
  },
];

function PulseDot({ color }: { color: string }) {
  return (
    <div className="relative flex h-6 w-full items-center justify-center">
      <div
        className="h-full"
        style={{
          width: 1,
          backgroundImage: `repeating-linear-gradient(to bottom, #2A2A3A 0, #2A2A3A 4px, transparent 4px, transparent 8px)`,
        }}
      />
      <motion.div
        className="absolute rounded-full"
        style={{
          width: 4,
          height: 4,
          backgroundColor: color,
          boxShadow: `0 0 8px ${color}`,
        }}
        animate={{ y: [-12, 12] }}
        transition={{
          duration: 3,
          repeat: Infinity,
          ease: "linear",
        }}
      />
    </div>
  );
}

function ArchitectureLayer({
  layer,
  isExpanded,
  onClick,
}: {
  layer: Layer;
  isExpanded: boolean;
  onClick: () => void;
}) {
  return (
    <motion.button
      onClick={onClick}
      className="w-full cursor-pointer rounded-xl border border-[rgba(108,60,225,0.2)] bg-bg-card p-6 text-left transition-shadow duration-200 hover:-translate-y-0.5 hover:shadow-lg md:p-8"
      style={{ borderLeftWidth: 3, borderLeftColor: layer.accentColor }}
      whileHover={{ y: -2 }}
      transition={transitions.fast}
      aria-expanded={isExpanded}
      role="button"
    >
      <div className="flex items-center gap-3">
        <span style={{ color: layer.accentColor }}>{layer.icon}</span>
        <span className="text-[13px] font-semibold uppercase tracking-[2px] text-text-secondary">
          {layer.label}
        </span>
      </div>
      <p className="mt-2 text-lg font-semibold text-text-primary">
        {layer.title}
      </p>
      <AnimatePresence initial={false}>
        {isExpanded && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: "auto", opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={transitions.default}
            className="overflow-hidden"
          >
            <p className="mt-3 text-[15px] leading-relaxed text-text-secondary">
              {layer.body}
            </p>
          </motion.div>
        )}
      </AnimatePresence>
    </motion.button>
  );
}

export function ArchitectureStack() {
  const [expanded, setExpanded] = useState(2); // Start with bottom layer (Solana = index 2)
  const { ref, isInView } = useInView(0.2);
  const reduced = useReducedMotion();

  return (
    <section className="bg-bg-secondary px-6 py-24 md:py-[120px]">
      <div className="mx-auto max-w-[1100px]">
        <SectionHeader
          eyebrow="ARCHITECTURE"
          title="Three layers. One sovereign entity."
          subtitle="Your legal entity, your governance protocol, and your financial infrastructure \u2014 unified on a single stack. Each layer is independent. Together, they\u2019re unstoppable."
          maxWidth="600px"
        />

        <div ref={ref} className="mt-12 flex flex-col items-center gap-0">
          {layers.map((layer, idx) => (
            <div key={layer.id} className="w-full max-w-[700px]">
              <motion.div
                initial={reduced ? {} : { opacity: 0, y: 20 }}
                animate={isInView ? { opacity: 1, y: 0 } : {}}
                transition={{
                  ...transitions.default,
                  delay: idx * 0.15,
                }}
              >
                <ArchitectureLayer
                  layer={layer}
                  isExpanded={expanded === idx}
                  onClick={() => setExpanded(idx)}
                />
              </motion.div>
              {idx < layers.length - 1 && (
                <PulseDot color="#6C3CE1" />
              )}
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
