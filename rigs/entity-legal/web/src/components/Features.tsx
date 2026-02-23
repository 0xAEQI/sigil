"use client";

import { motion } from "framer-motion";
import {
  Table2,
  ShieldCheck,
  KeyRound,
  Globe,
  Layers,
  FileCode2,
} from "lucide-react";
import { transitions } from "@/lib/animations";
import { useInView, useReducedMotion } from "@/lib/hooks";
import { SectionHeader } from "./SectionHeader";

interface Feature {
  icon: React.ReactNode;
  iconColor: string;
  title: string;
  body: string;
}

const features: Feature[] = [
  {
    icon: <Table2 className="h-6 w-6" />,
    iconColor: "#14F195",
    title: "On-chain cap table",
    body: "Your membership interests are SPL tokens on Solana. Not a PDF. Not a spreadsheet someone emails you quarterly. A live, queryable, immutable record of who owns what \u2014 updated in real-time with sub-second finality.\n\nEvery transfer is a signed transaction. Every cap table entry has a block height. Your investors can verify their ownership from any Solana explorer, any wallet, any time.",
  },
  {
    icon: <ShieldCheck className="h-6 w-6" />,
    iconColor: "#6C3CE1",
    title: "DAO governance via Squads",
    body: "Governance isn\u2019t a dashboard you log into. It\u2019s an M-of-N multisig on Squads Protocol \u2014 the most battle-tested multisig on Solana, securing over $10B in assets.\n\nSet approval thresholds. Add time locks. Define spending limits. Assign roles. Every governance action is a Solana transaction, signed and recorded forever. No black-box votes. No \u201Ctrust us\u201D governance.",
  },
  {
    icon: <KeyRound className="h-6 w-6" />,
    iconColor: "#E8C547",
    title: "Your keys, your entity",
    body: "We don\u2019t touch your keys. No Privy. No embedded wallets. No custody abstraction layer skimming fees between you and your own assets.\n\nConnect your existing wallet. Sign your own transactions. We provide the legal wrapper and the smart contract infrastructure. You maintain absolute control. Because the moment you hand your keys to middleware, your \u201Cdecentralized\u201D entity is just a bank with extra steps.",
  },
  {
    icon: <Globe className="h-6 w-6" />,
    iconColor: "#3B82F6",
    title: "Marshall Islands DAO LLC",
    body: "The Marshall Islands passed the world\u2019s first DAO-specific legislation in 2022. Not a retrofit. Not \u201Cwe\u2019ll figure it out.\u201D A purpose-built legal framework that recognizes smart contracts as legitimate corporate governance instruments.\n\nNo US securities registration. No state-level regulatory patchwork. No annual Delaware franchise tax theater. A sovereign jurisdiction with a clear, stable legal framework designed for exactly what you\u2019re building.",
  },
  {
    icon: <Layers className="h-6 w-6" />,
    iconColor: "#EC4899",
    title: "Series LLC support",
    body: "One parent entity. Unlimited child entities. Each series is liability-isolated \u2014 if one project fails, the others are legally walled off.\n\nLaunch a new product line, spin up a sub-DAO, or isolate a high-risk experiment \u2014 all under a single umbrella entity. Each series gets its own on-chain cap table, its own governance multisig, its own legal identity. One formation fee. Infinite optionality.",
  },
  {
    icon: <FileCode2 className="h-6 w-6" />,
    iconColor: "#F97316",
    title: "Smart contract IS the legal registry",
    body: "Your LLC agreement names a Solana smart contract address as the authoritative membership registry. Not as a \u201Cnice-to-have\u201D digital mirror. As the legal record.\n\nWhen someone buys a membership interest, the SPL token transfer IS the legally binding transfer of ownership. The blockchain is not backing up your cap table. The blockchain is your cap table. Your LLC agreement says so, the Marshall Islands recognizes it, and that\u2019s that.",
  },
];

function FeatureCard({ feature, index }: { feature: Feature; index: number }) {
  const { ref, isInView } = useInView(0.3);
  const reduced = useReducedMotion();

  return (
    <motion.div
      ref={ref}
      initial={reduced ? {} : { opacity: 0, y: 20 }}
      animate={isInView ? { opacity: 1, y: 0 } : {}}
      transition={{ ...transitions.default, delay: index * 0.1 }}
      className="group rounded-xl border border-border bg-bg-card p-8 transition-all duration-200 hover:-translate-y-0.5 hover:border-opacity-40"
      style={
        {
          "--hover-border": feature.iconColor,
        } as React.CSSProperties
      }
      onMouseEnter={(e) => {
        (e.currentTarget as HTMLElement).style.borderColor =
          feature.iconColor + "66";
      }}
      onMouseLeave={(e) => {
        (e.currentTarget as HTMLElement).style.borderColor = "";
      }}
    >
      <div className="mb-4" style={{ color: feature.iconColor }}>
        {feature.icon}
      </div>
      <h3 className="text-lg font-semibold text-text-primary">
        {feature.title}
      </h3>
      <div className="mt-3 space-y-3 text-[15px] leading-relaxed text-text-secondary">
        {feature.body.split("\n\n").map((paragraph, i) => (
          <p key={i}>{paragraph}</p>
        ))}
      </div>
    </motion.div>
  );
}

export function Features() {
  return (
    <section className="bg-bg-primary px-6 py-24 md:py-[120px]">
      <div className="mx-auto max-w-[1100px]">
        <SectionHeader
          eyebrow="FEATURES"
          title="Built different. On purpose."
          subtitle="Every piece of entity.legal exists because the alternatives got it wrong. No custody middleware. No token wrappers. No regulatory theater. Just verifiable legal structure on verifiable infrastructure."
          maxWidth="640px"
        />

        <div className="mt-12 grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
          {features.map((feature, i) => (
            <FeatureCard key={feature.title} feature={feature} index={i} />
          ))}
        </div>
      </div>
    </section>
  );
}
