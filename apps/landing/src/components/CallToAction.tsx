import { motion } from "framer-motion";

const fadeUp = (delay = 0) => ({
  initial: { opacity: 0, y: 20 },
  whileInView: { opacity: 1, y: 0 },
  viewport: { once: true, margin: "-80px" } as const,
  transition: { duration: 0.6, ease: "easeOut" as const, delay },
});

export function CallToAction() {
  return (
    <section className="relative z-10 max-w-4xl mx-auto px-8 pt-20 pb-12">
      <div className="text-center mb-32 relative">
        {/* Ambient glow */}
        <div
          className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 w-[400px] h-[200px] pointer-events-none"
          style={{
            background: "radial-gradient(ellipse, rgba(99,102,241,0.05) 0%, transparent 70%)",
          }}
        />

        <motion.h2
          {...fadeUp()}
          className="text-3xl md:text-4xl font-medium text-white/90 leading-snug mb-4 relative"
          style={{ fontFamily: "'Space Grotesk', sans-serif" }}
        >
          From goals to done.
        </motion.h2>
        <motion.p
          {...fadeUp(0.1)}
          className="text-[15px] text-white/30 mb-10"
        >
          Four tables. One loop. A tree that thinks.
        </motion.p>
        <motion.div
          {...fadeUp(0.2)}
          className="flex items-center justify-center gap-6"
        >
          <a
            href="https://app.aeqi.ai"
            className="relative group"
          >
            {/* Glow behind button */}
            <div
              className="absolute -inset-1 rounded-full opacity-0 group-hover:opacity-100 blur-md transition-opacity duration-500"
              style={{
                background: "linear-gradient(135deg, #818cf8, #67e8f9)",
              }}
            />
            <div
              className="relative bg-white text-[#08080C] rounded-full px-8 py-3.5 text-[14px] font-medium hover:bg-white/95 transition-colors"
              style={{ fontFamily: "'Space Grotesk', sans-serif" }}
            >
              Enter
            </div>
          </a>
          <a
            href="https://github.com/0xAEQI/aeqi"
            className="text-white/30 hover:text-white/60 transition-colors text-[14px]"
          >
            View Source
          </a>
        </motion.div>
      </div>

      <div className="h-px bg-white/[0.04] mb-6" />
      <footer
        className="flex items-center justify-between text-[11px] text-white/15 pb-6"
        style={{ fontFamily: "'Space Grotesk', sans-serif" }}
      >
        <span className="tracking-[0.08em]">aeqi.ai</span>
        <div className="flex gap-5">
          <a href="https://github.com/0xAEQI/aeqi" className="hover:text-white/30 transition-colors">GitHub</a>
          <a href="https://github.com/0xAEQI/aeqi/blob/main/docs/architecture.md" className="hover:text-white/30 transition-colors">Docs</a>
          <a href="#" className="hover:text-white/30 transition-colors">Terms</a>
        </div>
      </footer>
    </section>
  );
}
