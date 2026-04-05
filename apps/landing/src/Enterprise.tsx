import { motion } from "framer-motion";

const fade = (delay = 0) => ({
  initial: { opacity: 0, y: 8 } as const,
  animate: { opacity: 1, y: 0 } as const,
  transition: { duration: 0.7, ease: [0.25, 0.1, 0.25, 1] as const, delay },
});

const fadeView = (delay = 0) => ({
  initial: { opacity: 0, y: 16 } as const,
  whileInView: { opacity: 1, y: 0 } as const,
  viewport: { once: true, margin: "-40px" } as const,
  transition: { duration: 0.7, ease: [0.25, 0.1, 0.25, 1] as const, delay },
});

/* ─── Nav ─── */
function Nav() {
  return (
    <motion.nav
      className="fixed top-0 left-0 right-0 z-50 flex justify-center pt-4 px-4"
      initial={{ opacity: 0, y: -20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.6, delay: 0.15, ease: [0.25, 0.1, 0.25, 1] }}
    >
      <div className="w-full max-w-3xl backdrop-blur-2xl bg-white/60 border border-black/[0.06] rounded-2xl shadow-lg shadow-black/[0.03] px-5 h-12 flex items-center justify-between">
        <a href="/" className="text-[16px] font-semibold tracking-tight text-black/60 hover:text-black/80 transition-colors">
          aeqi
        </a>
        <div className="flex items-center gap-1">
          <a href="/economy" className="text-[13px] text-black/40 hover:text-black/70 hover:bg-black/[0.04] rounded-lg px-3 py-1.5 transition-all">
            Economy
          </a>
          <a href="/enterprise" className="text-[13px] text-black/70 font-medium hover:bg-black/[0.04] rounded-lg px-3 py-1.5 transition-all">
            Enterprise
          </a>
          <div className="w-px h-5 bg-black/[0.08] mx-1.5" />
          <a href="https://app.aeqi.ai/login" className="text-[13px] text-black/40 hover:text-black/70 hover:bg-black/[0.04] rounded-lg px-3 py-1.5 transition-all">
            Log in
          </a>
          <a
            href="https://app.aeqi.ai/signup"
            className="bg-black text-white rounded-xl px-4 py-1.5 text-[13px] font-medium hover:bg-black/85 transition-all hover:shadow-md hover:shadow-black/10 active:scale-[0.97]"
          >
            Sign up
          </a>
        </div>
      </div>
    </motion.nav>
  );
}

/* ─── Pricing ─── */
function Pricing() {
  return (
    <section className="flex-1 flex items-center justify-center px-6 pt-32 pb-20">
      <div className="max-w-4xl mx-auto w-full">
        <motion.div className="text-center mb-20" {...fade(0.1)}>
          <h1 className="text-[28px] md:text-[36px] font-semibold tracking-tight text-black/80 leading-snug">
            Simple pricing.
            <br />
            <span className="text-black/40">Scale when you're ready.</span>
          </h1>
        </motion.div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {/* Starter */}
          <motion.div
            className="rounded-2xl border border-black/[0.06] bg-white p-8 flex flex-col"
            {...fade(0.2)}
          >
            <p className="text-[11px] uppercase tracking-[0.2em] text-black/30 mb-6">Starter</p>
            <div className="mb-1">
              <span className="text-[36px] font-semibold tracking-tight text-black/80">$20</span>
              <span className="text-[15px] text-black/30 ml-1">/mo</span>
            </div>
            <p className="text-[13px] text-black/25 mb-8">For getting started with your first idea.</p>
            <div className="space-y-3.5 text-[14px] text-black/45 mb-10">
              <div className="flex items-center gap-2.5">
                <span className="text-black/15">+</span>
                <span>Up to 2 companies</span>
              </div>
              <div className="flex items-center gap-2.5">
                <span className="text-black/15">+</span>
                <span>Unlimited agents</span>
              </div>
              <div className="flex items-center gap-2.5">
                <span className="text-black/15">+</span>
                <span>500K LLM tokens included</span>
              </div>
              <div className="flex items-center gap-2.5">
                <span className="text-black/15">+</span>
                <span>On-chain cap table</span>
              </div>
              <div className="flex items-center gap-2.5">
                <span className="text-black/15">+</span>
                <span>Economy listing</span>
              </div>
              <div className="flex items-center gap-2.5">
                <span className="text-black/15">+</span>
                <span>Bring your own LLM key</span>
              </div>
            </div>
            <a
              href="https://app.aeqi.ai/signup"
              className="mt-auto inline-block text-center bg-black text-white rounded-xl px-6 py-3 text-[14px] font-medium hover:bg-black/85 transition-all hover:shadow-md hover:shadow-black/10 active:scale-[0.97]"
            >
              Launch a Company
            </a>
          </motion.div>

          {/* Growth */}
          <motion.div
            className="rounded-2xl border border-black/[0.12] bg-white p-8 flex flex-col ring-1 ring-black/[0.04]"
            {...fade(0.3)}
          >
            <p className="text-[11px] uppercase tracking-[0.2em] text-black/30 mb-6">Growth</p>
            <div className="mb-1">
              <span className="text-[36px] font-semibold tracking-tight text-black/80">$100</span>
              <span className="text-[15px] text-black/30 ml-1">/mo</span>
            </div>
            <p className="text-[13px] text-black/25 mb-8">For running a portfolio of companies.</p>
            <div className="space-y-3.5 text-[14px] text-black/45 mb-10">
              <div className="flex items-center gap-2.5">
                <span className="text-black/15">+</span>
                <span>Unlimited companies</span>
              </div>
              <div className="flex items-center gap-2.5">
                <span className="text-black/15">+</span>
                <span>Unlimited agents</span>
              </div>
              <div className="flex items-center gap-2.5">
                <span className="text-black/15">+</span>
                <span>5M LLM tokens included</span>
              </div>
              <div className="flex items-center gap-2.5">
                <span className="text-black/15">+</span>
                <span>Portfolio dashboard</span>
              </div>
              <div className="flex items-center gap-2.5">
                <span className="text-black/15">+</span>
                <span>Priority support</span>
              </div>
              <div className="flex items-center gap-2.5">
                <span className="text-black/15">+</span>
                <span>Everything in Starter</span>
              </div>
            </div>
            <a
              href="https://app.aeqi.ai/signup"
              className="mt-auto inline-block text-center bg-black text-white rounded-xl px-6 py-3 text-[14px] font-medium hover:bg-black/85 transition-all hover:shadow-md hover:shadow-black/10 active:scale-[0.97]"
            >
              Get Started
            </a>
          </motion.div>

          {/* Enterprise */}
          <motion.div
            className="rounded-2xl border border-black/[0.06] bg-white p-8 flex flex-col"
            {...fade(0.4)}
          >
            <p className="text-[11px] uppercase tracking-[0.2em] text-black/30 mb-6">Enterprise</p>
            <div className="mb-1">
              <span className="text-[36px] font-semibold tracking-tight text-black/80">Custom</span>
            </div>
            <p className="text-[13px] text-black/25 mb-8">For teams that need more control.</p>
            <div className="space-y-3.5 text-[14px] text-black/45 mb-10">
              <div className="flex items-center gap-2.5">
                <span className="text-black/15">+</span>
                <span>Dedicated infrastructure</span>
              </div>
              <div className="flex items-center gap-2.5">
                <span className="text-black/15">+</span>
                <span>Unlimited LLM tokens</span>
              </div>
              <div className="flex items-center gap-2.5">
                <span className="text-black/15">+</span>
                <span>Custom integrations</span>
              </div>
              <div className="flex items-center gap-2.5">
                <span className="text-black/15">+</span>
                <span>SLA and dedicated support</span>
              </div>
              <div className="flex items-center gap-2.5">
                <span className="text-black/15">+</span>
                <span>White-glove onboarding</span>
              </div>
              <div className="flex items-center gap-2.5">
                <span className="text-black/15">+</span>
                <span>Everything in Growth</span>
              </div>
            </div>
            <a
              href="https://cal.com/aeqi/enterprise"
              className="mt-auto inline-block text-center border border-black/[0.1] text-black/60 rounded-xl px-6 py-3 text-[14px] font-medium hover:bg-black/[0.03] hover:border-black/[0.15] transition-all active:scale-[0.97]"
            >
              Book a Demo
            </a>
          </motion.div>
        </div>

        {/* Token note */}
        <motion.p
          className="text-center text-[13px] text-black/20 mt-12"
          {...fadeView(0.1)}
        >
          All plans support bring-your-own OpenRouter key. Extra tokens available as top-ups.
        </motion.p>
      </div>
    </section>
  );
}

/* ─── Footer ─── */
function Footer() {
  return (
    <footer className="border-t border-black/[0.04]">
      <div className="max-w-4xl mx-auto px-6 py-14 w-full">
        <div className="grid grid-cols-2 md:grid-cols-3 gap-10 md:gap-14">
          <motion.div {...fadeView(0.05)}>
            <p className="text-[11px] uppercase tracking-[0.2em] text-black/20 mb-4">Product</p>
            <div className="space-y-2.5 text-[13px]">
              <a href="https://app.aeqi.ai" className="block text-black/35 hover:text-black/60 transition-colors">Launch a Company</a>
              <a href="/enterprise" className="block text-black/35 hover:text-black/60 transition-colors">Enterprise</a>
              <a href="https://github.com/0xAEQI/aeqi/blob/main/docs/architecture.md" className="block text-black/35 hover:text-black/60 transition-colors">Docs</a>
            </div>
          </motion.div>

          <motion.div {...fadeView(0.1)}>
            <p className="text-[11px] uppercase tracking-[0.2em] text-black/20 mb-4">Community</p>
            <div className="space-y-2.5 text-[13px]">
              <a href="https://github.com/0xAEQI/aeqi" className="block text-black/35 hover:text-black/60 transition-colors">GitHub</a>
              <a href="https://x.com/0xAEQI" className="block text-black/35 hover:text-black/60 transition-colors">X</a>
            </div>
          </motion.div>

          <motion.div {...fadeView(0.15)}>
            <p className="text-[11px] uppercase tracking-[0.2em] text-black/20 mb-4">Legal</p>
            <div className="space-y-2.5 text-[13px]">
              <a href="/terms" className="block text-black/35 hover:text-black/60 transition-colors">Terms</a>
              <a href="/privacy" className="block text-black/35 hover:text-black/60 transition-colors">Privacy</a>
            </div>
          </motion.div>
        </div>

        <motion.div {...fadeView(0.2)} className="mt-14 pt-6 border-t border-black/[0.04] flex items-center justify-between">
          <a href="/" className="text-[22px] font-bold tracking-tighter text-black/25 leading-none hover:text-black/40 transition-colors">æ</a>
          <p className="text-[12px] text-black/20">
            &copy; {new Date().getFullYear()} aeqi.ai
          </p>
        </motion.div>
      </div>
    </footer>
  );
}

export default function Enterprise() {
  return (
    <div className="min-h-screen flex flex-col bg-white">
      <Nav />
      <Pricing />
      <div className="bg-[#fafafa]">
        <Footer />
      </div>
    </div>
  );
}
