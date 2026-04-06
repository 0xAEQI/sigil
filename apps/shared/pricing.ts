/** Central pricing config. Imported by both landing page and dashboard app. */

export const TRIAL = {
  days: 3,
  companies: 1,
  agents: 3,
  tokens: "3M",
};

export const PLANS = [
  {
    id: "starter" as const,
    name: "Starter",
    price: 29,
    popular: false,
    tagline: "Ship your first autonomous company.",
    desc: "For individuals getting started with autonomous agents.",
    features: [
      "2 companies",
      "30 agents",
      "50M LLM tokens / month",
      "On-chain cap table",
      "Economy listing",
      "Bring your own LLM key",
    ],
    short: [
      "2 companies",
      "30 agents",
      "50M tokens / month",
      "Email support",
    ],
  },
  {
    id: "growth" as const,
    name: "Growth",
    price: 79,
    popular: true,
    tagline: "Run a portfolio at scale.",
    desc: "For teams running multiple companies with higher volume.",
    features: [
      "Everything in Starter",
      "10 companies",
      "150 agents",
      "500M LLM tokens / month",
      "Priority support",
      "Custom agent templates",
    ],
    short: [
      "10 companies",
      "150 agents",
      "500M tokens / month",
      "Priority support",
      "Custom agent templates",
    ],
  },
] as const;

export type PlanId = (typeof PLANS)[number]["id"];
