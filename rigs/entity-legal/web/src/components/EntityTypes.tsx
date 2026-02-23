"use client";

import { motion } from "framer-motion";
import { Coins, HeartHandshake, GitBranch, Building2 } from "lucide-react";
import { transitions } from "@/lib/animations";
import { useInView, useReducedMotion } from "@/lib/hooks";
import { SectionHeader } from "./SectionHeader";
import { Tag } from "./Tag";
import { config } from "@/lib/config";

interface EntityType {
  icon: React.ReactNode;
  accentColor: string;
  title: string;
  subtitle: string;
  body: string;
  tags: string[];
  ctaText: string;
}

const entityTypes: EntityType[] = [
  {
    icon: <Coins className="h-6 w-6" />,
    accentColor: "#6C3CE1",
    title: "DAO LLC",
    subtitle: "For-profit on-chain entities",
    body: "The standard formation for revenue-generating DAOs, on-chain funds, and Web3 startups. Full limited liability protection. Smart contract governance. Membership interests as SPL tokens.\n\nRecognized under the Marshall Islands DAO Act 2022. Can hold assets, enter contracts, sue and be sued. The same legal standing as a traditional LLC \u2014 with on-chain governance baked in.",
    tags: ["DeFi protocols", "Investment DAOs", "Web3 startups", "On-chain funds"],
    ctaText: "Form a DAO LLC",
  },
  {
    icon: <HeartHandshake className="h-6 w-6" />,
    accentColor: "#14F195",
    title: "Non-Profit DAO",
    subtitle: "Mission-driven on-chain organizations",
    body: "For DAOs focused on public goods, open-source development, grants distribution, or community governance. Same on-chain infrastructure. Different tax treatment and legal obligations.\n\nGovernance via token-weighted voting or one-member-one-vote. Treasury managed through Squads multisig. Perfect for protocol foundations, grants DAOs, and open-source collectives.",
    tags: ["Protocol foundations", "Grants DAOs", "Open-source collectives", "Public goods"],
    ctaText: "Form a Non-Profit DAO",
  },
  {
    icon: <GitBranch className="h-6 w-6" />,
    accentColor: "#EC4899",
    title: "Series LLC",
    subtitle: "One umbrella. Unlimited sub-entities.",
    body: "A parent entity with the ability to spawn liability-isolated child entities (series). Each series has its own assets, members, and governance \u2014 legally firewalled from every other series.\n\nOne formation. One registered agent. Unlimited series. Each series gets its own Squads multisig, its own SPL token cap table, its own legal identity. Scale your entity structure without scaling your legal overhead.",
    tags: ["Multi-product DAOs", "Venture studios", "Incubators", "Fund-of-funds"],
    ctaText: "Form a Series LLC",
  },
  {
    icon: <Building2 className="h-6 w-6" />,
    accentColor: "#3B82F6",
    title: "Traditional LLC",
    subtitle: "Marshall Islands LLC without DAO governance",
    body: "For founders who want Marshall Islands jurisdiction without the DAO governance layer. Standard LLC formation with a registered agent, operating agreement, and Certificate of Formation.\n\nStill comes with on-chain cap table option. Still gets Marshall Islands jurisdictional benefits. Just without the mandatory smart contract governance requirement. You can always upgrade to DAO LLC later.",
    tags: ["Solo founders", "Holding companies", "IP entities", "Simple structures"],
    ctaText: "Form a Traditional LLC",
  },
];

function EntityTypeCard({
  entity,
  index,
  onCtaClick,
}: {
  entity: EntityType;
  index: number;
  onCtaClick: () => void;
}) {
  const { ref, isInView } = useInView(0.3);
  const reduced = useReducedMotion();

  return (
    <motion.div
      ref={ref}
      initial={reduced ? {} : { opacity: 0, y: 20 }}
      animate={isInView ? { opacity: 1, y: 0 } : {}}
      transition={{ ...transitions.default, delay: index * 0.1 }}
      className="flex flex-col rounded-xl border border-border bg-bg-card overflow-hidden"
      style={{ borderTopWidth: 4, borderTopColor: entity.accentColor }}
    >
      <div className="flex flex-1 flex-col p-6 md:p-8">
        <div className="mb-4" style={{ color: entity.accentColor }}>
          {entity.icon}
        </div>
        <h3 className="text-xl font-bold text-text-primary">{entity.title}</h3>
        <p className="mt-1 text-sm text-text-secondary">{entity.subtitle}</p>
        <div className="mt-4 flex-1 space-y-3 text-[14px] leading-relaxed text-text-secondary">
          {entity.body.split("\n\n").map((paragraph, i) => (
            <p key={i}>{paragraph}</p>
          ))}
        </div>
        <div className="mt-6 flex flex-wrap gap-2">
          {entity.tags.map((tag) => (
            <Tag key={tag} text={tag} />
          ))}
        </div>
      </div>
      <div className="border-t border-border p-6 md:px-8">
        <button
          onClick={onCtaClick}
          className="text-sm font-semibold text-accent transition-colors hover:text-accent-hover"
        >
          {entity.ctaText} {"\u2192"}
        </button>
      </div>
    </motion.div>
  );
}

export function EntityTypes({ onCtaClick }: { onCtaClick: () => void }) {
  return (
    <section className="bg-bg-primary px-6 py-24 md:py-[120px]">
      <div className="mx-auto max-w-[1100px]">
        <SectionHeader
          eyebrow="ENTITY TYPES"
          title="Pick your structure. We\u2019ll handle the rest."
          subtitle="Every entity type comes with on-chain cap table deployment and Squads multisig governance. The legal structure changes. The infrastructure doesn\u2019t."
          maxWidth="640px"
        />

        <div className="mt-12 grid gap-6 sm:grid-cols-2 lg:grid-cols-4">
          {entityTypes.map((entity, i) => (
            <EntityTypeCard
              key={entity.title}
              entity={entity}
              index={i}
              onCtaClick={onCtaClick}
            />
          ))}
        </div>
      </div>
    </section>
  );
}
