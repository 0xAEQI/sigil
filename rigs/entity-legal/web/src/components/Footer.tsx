"use client";

import { useState } from "react";
import { config } from "@/lib/config";

function BrandLogo() {
  return (
    <span className="text-xl font-bold text-text-primary">
      entity<span className="text-accent">.</span>legal
    </span>
  );
}

function SocialLinks() {
  return (
    <div className="mt-6 flex gap-4">
      <a
        href={config.social.twitter}
        target="_blank"
        rel="noopener noreferrer"
        className="text-text-tertiary transition-colors duration-200 hover:text-text-primary"
        aria-label="Follow us on X (Twitter)"
      >
        <svg className="h-5 w-5" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z" />
        </svg>
      </a>
      <a
        href={config.social.discord}
        target="_blank"
        rel="noopener noreferrer"
        className="text-text-tertiary transition-colors duration-200 hover:text-text-primary"
        aria-label="Join our Discord"
      >
        <svg className="h-5 w-5" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path d="M20.317 4.3698a19.7913 19.7913 0 00-4.8851-1.5152.0741.0741 0 00-.0785.0371c-.211.3753-.4447.8648-.6083 1.2495-1.8447-.2762-3.68-.2762-5.4868 0-.1636-.3933-.4058-.8742-.6177-1.2495a.077.077 0 00-.0785-.037 19.7363 19.7363 0 00-4.8852 1.515.0699.0699 0 00-.0321.0277C.5334 9.0458-.319 13.5799.0992 18.0578a.0824.0824 0 00.0312.0561c2.0528 1.5076 4.0413 2.4228 5.9929 3.0294a.0777.0777 0 00.0842-.0276c.4616-.6304.8731-1.2952 1.226-1.9942a.076.076 0 00-.0416-.1057c-.6528-.2476-1.2743-.5495-1.8722-.8923a.077.077 0 01-.0076-.1277c.1258-.0943.2517-.1923.3718-.2914a.0743.0743 0 01.0776-.0105c3.9278 1.7933 8.18 1.7933 12.0614 0a.0739.0739 0 01.0785.0095c.1202.099.246.1981.3728.2924a.077.077 0 01-.0066.1276 12.2986 12.2986 0 01-1.873.8914.0766.0766 0 00-.0407.1067c.3604.698.7719 1.3628 1.225 1.9932a.076.076 0 00.0842.0286c1.961-.6067 3.9495-1.5219 6.0023-3.0294a.077.077 0 00.0313-.0552c.5004-5.177-.8382-9.6739-3.5485-13.6604a.061.061 0 00-.0312-.0286zM8.02 15.3312c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9555-2.4189 2.157-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.9555 2.4189-2.1569 2.4189zm7.9748 0c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9554-2.4189 2.1569-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.946 2.4189-2.1568 2.4189z" />
        </svg>
      </a>
      <a
        href={config.social.telegram}
        target="_blank"
        rel="noopener noreferrer"
        className="text-text-tertiary transition-colors duration-200 hover:text-text-primary"
        aria-label="Join our Telegram"
      >
        <svg className="h-5 w-5" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path d="M11.944 0A12 12 0 0 0 0 12a12 12 0 0 0 12 12 12 12 0 0 0 12-12A12 12 0 0 0 12 0a12 12 0 0 0-.056 0zm4.962 7.224c.1-.002.321.023.465.14a.506.506 0 0 1 .171.325c.016.093.036.306.02.472-.18 1.898-.962 6.502-1.36 8.627-.168.9-.499 1.201-.82 1.23-.696.065-1.225-.46-1.9-.902-1.056-.693-1.653-1.124-2.678-1.8-1.185-.78-.417-1.21.258-1.91.177-.184 3.247-2.977 3.307-3.23.007-.032.014-.15-.056-.212s-.174-.041-.249-.024c-.106.024-1.793 1.14-5.061 3.345-.479.33-.913.49-1.302.48-.428-.008-1.252-.241-1.865-.44-.752-.245-1.349-.374-1.297-.789.027-.216.325-.437.893-.663 3.498-1.524 5.83-2.529 6.998-3.014 3.332-1.386 4.025-1.627 4.476-1.635z" />
        </svg>
      </a>
    </div>
  );
}

function NewsletterForm() {
  const [email, setEmail] = useState("");
  const [submitted, setSubmitted] = useState(false);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    // TODO: integrate with email provider API
    setSubmitted(true);
  };

  if (submitted) {
    return (
      <p className="text-sm text-solana">
        Subscribed. We&rsquo;ll keep you updated.
      </p>
    );
  }

  return (
    <form onSubmit={handleSubmit} className="flex max-w-[500px]">
      <input
        type="email"
        value={email}
        onChange={(e) => setEmail(e.target.value)}
        placeholder="your@email.com"
        required
        className="flex-1 rounded-l-lg border border-border bg-bg-card px-4 py-3.5 text-sm text-text-primary placeholder:text-text-muted focus:border-accent focus:outline-none"
      />
      <button
        type="submit"
        className="rounded-r-lg bg-accent px-6 py-3.5 text-sm font-semibold text-white transition-colors hover:bg-accent-hover"
      >
        Subscribe {"\u2192"}
      </button>
    </form>
  );
}

const productLinks = [
  { label: "DAO LLC Formation", href: "/formation/dao-llc" },
  { label: "Series LLC", href: "/formation/series-llc" },
  { label: "Non-Profit DAO", href: "/formation/non-profit" },
  { label: "Traditional LLC", href: "/formation/traditional" },
  { label: "Pricing", href: "/pricing" },
  { label: "How It Works", href: "/#how-it-works" },
];

const resourceLinks = [
  { label: "Documentation", href: "/docs" },
  { label: "FAQ", href: "/faq" },
  { label: "LLC Agreement Template", href: "/docs/llc-template" },
  {
    label: "Marshall Islands DAO Act",
    href: "/docs/dao-act",
    external: false,
  },
  { label: "Squads Protocol", href: "https://squads.xyz", external: true },
  { label: "Blog", href: "/blog" },
];

const companyLinks = [
  { label: "About", href: "/about" },
  { label: "Contact", href: "/contact" },
  { label: "Privacy Policy", href: "/privacy" },
  { label: "Terms of Service", href: "/terms" },
  { label: "Careers", href: "/careers" },
];

function FooterColumn({
  title,
  links,
}: {
  title: string;
  links: { label: string; href: string; external?: boolean }[];
}) {
  return (
    <div>
      <h4 className="text-[13px] font-semibold uppercase tracking-[2px] text-text-tertiary">
        {title}
      </h4>
      <ul className="mt-4 space-y-0">
        {links.map((link) => (
          <li key={link.label}>
            <a
              href={link.href}
              {...(link.external
                ? { target: "_blank", rel: "noopener noreferrer" }
                : {})}
              className="inline-block py-1 text-sm leading-[2.2] text-text-secondary transition-colors hover:text-text-primary hover:underline"
            >
              {link.label}
              {link.external && (
                <span className="ml-1 text-text-muted" aria-hidden="true">
                  {"\u2197"}
                </span>
              )}
            </a>
          </li>
        ))}
      </ul>
    </div>
  );
}

export function Footer() {
  return (
    <footer className="border-t border-border bg-[#080810] px-6 pb-10 pt-20">
      <div className="mx-auto max-w-[1100px]">
        {/* Grid */}
        <div className="grid gap-12 sm:grid-cols-2 lg:grid-cols-4">
          {/* Brand Column */}
          <div>
            <BrandLogo />
            <p className="mt-3 text-sm leading-relaxed text-text-tertiary">
              Sovereign entity formation
              <br />
              for the on-chain era.
            </p>
            <SocialLinks />
          </div>

          <FooterColumn title="Product" links={productLinks} />
          <FooterColumn title="Resources" links={resourceLinks} />
          <FooterColumn title="Company" links={companyLinks} />
        </div>

        {/* Newsletter */}
        <div className="mt-16 text-center">
          <p className="mb-4 text-sm text-text-secondary">
            Stay updated. No spam. Just formation law, governance design, and
            on-chain entity news.
          </p>
          <div className="flex justify-center">
            <NewsletterForm />
          </div>
        </div>

        {/* Bottom Bar */}
        <div className="mt-10 flex flex-col items-center justify-between gap-4 border-t border-border pt-6 sm:flex-row">
          <p className="text-[13px] text-text-tertiary">
            &copy; 2026 entity.legal. All rights reserved.
          </p>
          <p className="text-[13px] text-text-tertiary">
            Marshall Islands Registered Agent License
          </p>
        </div>
      </div>
    </footer>
  );
}
