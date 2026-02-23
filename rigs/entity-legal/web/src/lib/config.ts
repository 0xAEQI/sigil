// Environment-driven configuration
// These can be overridden via environment variables

export const config = {
  waitlistMode: process.env.NEXT_PUBLIC_WAITLIST_MODE !== "false", // default true
  showTestimonials: process.env.NEXT_PUBLIC_SHOW_TESTIMONIALS === "true", // default false

  // Stats — pulled from env vars so they can be updated without code changes
  stats: {
    entitiesFormed: Number(process.env.NEXT_PUBLIC_ENTITIES_FORMED || "47"),
    capTableValue: process.env.NEXT_PUBLIC_CAP_TABLE_VALUE || "12",
    capTableEntries: Number(
      process.env.NEXT_PUBLIC_CAP_TABLE_ENTRIES || "1847"
    ),
    custodyIncidents: Number(
      process.env.NEXT_PUBLIC_CUSTODY_INCIDENTS || "0"
    ),
    waitlistCount: Number(process.env.NEXT_PUBLIC_WAITLIST_COUNT || "214"),
  },

  social: {
    twitter: "https://x.com/entitylegal",
    discord: "https://discord.gg/entitylegal",
    telegram: "https://t.me/entitylegal",
  },
};
