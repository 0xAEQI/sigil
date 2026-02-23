"use client";

import { motion } from "framer-motion";
import { Check, X } from "lucide-react";
import { transitions } from "@/lib/animations";
import { useInView, useReducedMotion } from "@/lib/hooks";
import { SectionHeader } from "./SectionHeader";
import { Badge } from "./Badge";

interface PricingTier {
  title: string;
  price: string;
  period: string;
  renewal: string;
  description: string;
  features: string[];
  excluded?: string[];
  highlighted?: boolean;
  badge?: string;
  ctaText: string;
}

const tiers: PricingTier[] = [
  {
    title: "Starter",
    price: "$5,500",
    period: "one-time formation",
    renewal: "$1,800/year",
    description:
      "For solo founders and small teams who need a clean legal structure.",
    features: [
      "Marshall Islands DAO LLC formation",
      "LLC agreement with smart contract designation",
      "Registered agent (year 1 included)",
      "Certificate of Formation",
      "SPL token cap table deployment (Solana mainnet)",
      "Squads multisig setup (up to 3 signers)",
      "KYC processing for up to 3 founding members",
      "Formation documents delivered digitally",
      "Email support",
    ],
    excluded: [
      "Series LLC capability",
      "Custom governance design",
      "Priority support",
      "Compliance calendar management",
    ],
    ctaText: "Get Started",
  },
  {
    title: "Pro",
    price: "$8,500",
    period: "one-time formation",
    renewal: "$2,800/year",
    description:
      "For growing DAOs that need governance flexibility and compliance automation.",
    highlighted: true,
    badge: "MOST POPULAR",
    features: [
      "Everything in Starter, plus:",
      "Series LLC capability (up to 5 series at formation)",
      "Custom governance design consultation (1 session)",
      "Squads multisig setup (up to 7 signers)",
      "KYC processing for up to 10 founding members",
      "Compliance calendar with automated reminders",
      "Annual report preparation",
      "Priority email + Telegram support",
      "One LLC agreement amendment per year",
    ],
    ctaText: "Get Started",
  },
  {
    title: "Enterprise",
    price: "Custom",
    period: "contact us",
    renewal: "Custom",
    description:
      "For protocols, funds, and organizations with complex multi-entity structures.",
    features: [
      "Everything in Pro, plus:",
      "Unlimited series at formation",
      "Unlimited signers",
      "Unlimited founding member KYC",
      "Custom LLC agreement drafting",
      "Dedicated account manager",
      "Governance architecture consultation",
      "Multi-entity structure planning",
      "API access for cap table queries",
      "SLA-backed support (24h response)",
      "Legal opinion letter",
    ],
    ctaText: "Contact Us",
  },
];

function PricingCard({
  tier,
  index,
  onCtaClick,
}: {
  tier: PricingTier;
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
      transition={{ ...transitions.default, delay: index * 0.15 }}
      className={`relative flex flex-col rounded-xl border bg-bg-card p-8 ${
        tier.highlighted
          ? "border-accent scale-[1.02] shadow-lg shadow-accent/10"
          : "border-border"
      }`}
    >
      {tier.badge && (
        <div className="absolute -top-3 left-8">
          <Badge text={tier.badge} />
        </div>
      )}

      <div className="mb-6">
        <h3 className="text-xl font-bold text-text-primary">{tier.title}</h3>
        <div className="mt-3">
          <span className="text-4xl font-bold text-text-primary tabular-nums">
            {tier.price}
          </span>
          <span className="ml-2 text-sm text-text-tertiary">{tier.period}</span>
        </div>
        <p className="mt-1 text-sm text-text-tertiary">
          Annual renewal: {tier.renewal}{tier.renewal !== "Custom" ? " (after year 1)" : ""}
        </p>
        <p className="mt-3 text-sm leading-relaxed text-text-secondary">
          {tier.description}
        </p>
      </div>

      <div className="flex-1">
        <ul className="space-y-3">
          {tier.features.map((feature, i) => (
            <li
              key={i}
              className="flex items-start gap-3 text-sm text-text-secondary"
            >
              <Check className="mt-0.5 h-4 w-4 shrink-0 text-solana" />
              <span>{feature}</span>
            </li>
          ))}
        </ul>

        {tier.excluded && tier.excluded.length > 0 && (
          <ul className="mt-4 space-y-3 border-t border-border pt-4">
            {tier.excluded.map((item, i) => (
              <li
                key={i}
                className="flex items-start gap-3 text-sm text-text-muted"
              >
                <X className="mt-0.5 h-4 w-4 shrink-0" />
                <span>{item}</span>
              </li>
            ))}
          </ul>
        )}
      </div>

      <button
        onClick={onCtaClick}
        className={`mt-8 w-full rounded-lg py-3.5 text-sm font-semibold transition-colors duration-200 ${
          tier.highlighted
            ? "bg-accent text-white hover:bg-accent-hover"
            : "border border-border bg-transparent text-text-primary hover:border-accent hover:text-accent"
        }`}
      >
        {tier.ctaText} {"\u2192"}
      </button>
    </motion.div>
  );
}

export function Pricing({ onCtaClick }: { onCtaClick: () => void }) {
  return (
    <section id="pricing" className="bg-bg-secondary px-6 py-24 md:py-[120px]">
      <div className="mx-auto max-w-[1100px]">
        <SectionHeader
          eyebrow="PRICING"
          title="Transparent pricing. No retainer. No hourly."
          subtitle="One fee. Everything included. We don\u2019t charge you to ask questions, and we don\u2019t nickel-and-dime you for document revisions. The price is the price."
          maxWidth="640px"
        />

        <div className="mt-12 grid gap-6 md:grid-cols-3">
          {tiers.map((tier, i) => (
            <PricingCard
              key={tier.title}
              tier={tier}
              index={i}
              onCtaClick={onCtaClick}
            />
          ))}
        </div>

        {/* Comparison Table */}
        <ComparisonTable />

        {/* Pricing footnote */}
        <p className="mt-10 text-center text-[13px] text-text-tertiary">
          All prices in USD. Government filing fees included in formation cost.
          Annual renewal includes registered agent, registered office, and
          government annual fees. KYC processing fees included. No hidden fees
          {"\u2014"} ever.
        </p>
      </div>
    </section>
  );
}

function ComparisonTable() {
  const { ref, isInView } = useInView(0.2);
  const reduced = useReducedMotion();

  const headers = [
    "",
    "entity.legal (MI)",
    "Wyoming DAO LLC",
    "Delaware LLC",
    "Cayman Foundation",
  ];

  const rows = [
    {
      label: "Formation Cost",
      values: [
        { text: "$5,500\u2013$8,500", type: "neutral" as const },
        { text: "$5,000\u2013$25,000+", type: "neutral" as const },
        { text: "$3,000\u2013$15,000", type: "neutral" as const },
        { text: "$18,500\u2013$35,000+", type: "neutral" as const },
      ],
    },
    {
      label: "Annual Renewal",
      values: [
        { text: "$1,800\u2013$2,800", type: "neutral" as const },
        { text: "$500\u2013$2,000", type: "neutral" as const },
        { text: "$1,500\u2013$5,000", type: "neutral" as const },
        { text: "$8,000\u2013$15,000", type: "neutral" as const },
      ],
    },
    {
      label: "DAO-Specific Law",
      values: [
        { text: "Yes (DAO Act 2022)", type: "good" as const },
        { text: "Partial (2021 law)", type: "warn" as const },
        { text: "No", type: "neutral" as const },
        { text: "No", type: "neutral" as const },
      ],
    },
    {
      label: "Smart Contract as Legal Registry",
      values: [
        { text: "Yes", type: "good" as const },
        { text: "No", type: "neutral" as const },
        { text: "No", type: "neutral" as const },
        { text: "No", type: "neutral" as const },
      ],
    },
    {
      label: "On-Chain Cap Table",
      values: [
        { text: "Included", type: "good" as const },
        { text: "DIY", type: "neutral" as const },
        { text: "DIY", type: "neutral" as const },
        { text: "DIY", type: "neutral" as const },
      ],
    },
    {
      label: "Multisig Governance",
      values: [
        { text: "Included", type: "good" as const },
        { text: "DIY", type: "neutral" as const },
        { text: "DIY", type: "neutral" as const },
        { text: "DIY", type: "neutral" as const },
      ],
    },
    {
      label: "US Regulatory Exposure",
      values: [
        { text: "None", type: "good" as const },
        { text: "Full", type: "warn" as const },
        { text: "Full", type: "warn" as const },
        { text: "Partial", type: "warn" as const },
      ],
    },
    {
      label: "Securities Registration",
      values: [
        { text: "Not required", type: "good" as const },
        { text: "Likely required", type: "warn" as const },
        { text: "Likely required", type: "warn" as const },
        { text: "Not required", type: "good" as const },
      ],
    },
    {
      label: "Formation Time",
      values: [
        { text: "10\u201315 business days", type: "neutral" as const },
        { text: "5\u201310 business days", type: "neutral" as const },
        { text: "3\u20137 business days", type: "neutral" as const },
        { text: "30\u201360 business days", type: "neutral" as const },
      ],
    },
    {
      label: "KYC Requirement",
      values: [
        { text: "25%+ members", type: "good" as const },
        { text: "All members", type: "neutral" as const },
        { text: "All members", type: "neutral" as const },
        { text: "Directors + UBOs", type: "neutral" as const },
      ],
    },
    {
      label: "Series LLC Support",
      values: [
        { text: "Yes", type: "good" as const },
        { text: "Yes", type: "good" as const },
        { text: "Yes", type: "good" as const },
        { text: "No", type: "neutral" as const },
      ],
    },
  ];

  const colorMap = {
    good: "text-solana",
    warn: "text-gold",
    neutral: "text-text-secondary",
  };

  return (
    <motion.div
      ref={ref}
      initial={reduced ? {} : { opacity: 0, y: 20 }}
      animate={isInView ? { opacity: 1, y: 0 } : {}}
      transition={transitions.default}
      className="mt-16 overflow-x-auto"
    >
      <table className="w-full min-w-[700px] border-collapse rounded-xl bg-bg-card">
        <thead>
          <tr>
            {headers.map((header, i) => (
              <th
                key={i}
                className={`border-b border-border p-4 text-left text-sm font-semibold ${
                  i === 1
                    ? "border-l-2 border-l-accent text-accent"
                    : "text-text-secondary"
                } ${i === 0 ? "w-[200px]" : ""}`}
              >
                {header}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((row, rowIdx) => (
            <tr
              key={rowIdx}
              className="border-b border-border/50 transition-colors hover:bg-bg-elevated/30"
            >
              <td className="p-4 text-sm font-medium text-text-primary">
                {row.label}
              </td>
              {row.values.map((val, colIdx) => (
                <td
                  key={colIdx}
                  className={`p-4 text-sm ${colorMap[val.type]} ${
                    colIdx === 0 ? "border-l-2 border-l-accent/30" : ""
                  }`}
                >
                  {val.text}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </motion.div>
  );
}
