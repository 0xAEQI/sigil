import { motion } from "framer-motion";

const fadeUp = (delay = 0) => ({
  initial: { opacity: 0, y: 24 } as const,
  whileInView: { opacity: 1, y: 0 } as const,
  viewport: { once: true, margin: "-60px" } as const,
  transition: { duration: 0.6, ease: "easeOut" as const, delay },
});

const steps = [
  {
    num: "01",
    word: "Ask",
    color: "#818cf8",
    desc: "Define your goal. AEQI decomposes it into executable tasks and assigns them across agents.",
  },
  {
    num: "02",
    word: "Execute",
    color: "#67e8f9",
    desc: "Agents share persistent memory, avoid duplicate work, and build in parallel across your stack.",
  },
  {
    num: "03",
    word: "Question",
    color: "#c084fc",
    desc: "Every output is validated against your original intent. Nothing ships unchecked.",
  },
  {
    num: "04",
    word: "Improve",
    color: "#34d399",
    desc: "Results feed back into the system. Each cycle sharpens the process and compounds leverage.",
  },
];

export function Process() {
  return (
    <section className="relative z-10 max-w-5xl mx-auto px-8 py-28">
      <motion.div {...fadeUp()} className="text-center mb-20">
        <p
          className="text-[11px] uppercase tracking-[0.25em] text-white/15 mb-6"
          style={{ fontFamily: "'Space Grotesk', sans-serif" }}
        >
          The Loop
        </p>
        <p className="text-[15px] text-white/30 max-w-md mx-auto leading-relaxed">
          Persistent agent orchestration that decomposes, executes,
          validates, and improves — autonomously.
        </p>
      </motion.div>

      {/* Loop indicator */}
      <motion.div {...fadeUp(0.05)} className="flex justify-center mb-12">
        <svg
          viewBox="0 0 200 8"
          className="w-48 overflow-visible"
          fill="none"
        >
          <motion.path
            d="M 10 4 L 190 4"
            stroke="url(#loop-gradient)"
            strokeWidth="0.5"
            strokeDasharray="4 4"
            initial={{ pathLength: 0 }}
            whileInView={{ pathLength: 1 }}
            viewport={{ once: true }}
            transition={{ duration: 1.5, ease: "easeOut" }}
          />
          <defs>
            <linearGradient id="loop-gradient" x1="0%" y1="0%" x2="100%" y2="0%">
              <stop offset="0%" stopColor="#818cf8" stopOpacity="0.4" />
              <stop offset="33%" stopColor="#67e8f9" stopOpacity="0.4" />
              <stop offset="66%" stopColor="#c084fc" stopOpacity="0.4" />
              <stop offset="100%" stopColor="#34d399" stopOpacity="0.4" />
            </linearGradient>
          </defs>
        </svg>
      </motion.div>

      <div className="grid grid-cols-1 md:grid-cols-4 gap-10 md:gap-6">
        {steps.map((step, i) => (
          <motion.div key={i} {...fadeUp(0.08 * i)} className="group">
            <div
              className="border-t pt-6 transition-colors duration-500"
              style={{ borderColor: `${step.color}15` }}
            >
              <span
                className="text-[11px] font-mono block mb-3 transition-colors duration-500"
                style={{ color: `${step.color}50` }}
              >
                {step.num}
              </span>
              <h3
                className="text-[18px] font-semibold mb-3 transition-colors duration-300"
                style={{
                  fontFamily: "'Space Grotesk', sans-serif",
                  color: `${step.color}cc`,
                }}
              >
                {step.word}
              </h3>
              <p className="text-[13px] leading-relaxed text-white/25">
                {step.desc}
              </p>
            </div>
          </motion.div>
        ))}
      </div>

      {/* Loop-back arrow */}
      <motion.div
        {...fadeUp(0.4)}
        className="flex justify-center mt-10"
      >
        <svg viewBox="0 0 120 20" className="w-24 overflow-visible" fill="none">
          <motion.path
            d="M 100 5 C 110 5, 115 10, 110 15 C 105 20, 15 20, 10 15 C 5 10, 10 5, 20 5"
            stroke="rgba(255,255,255,0.06)"
            strokeWidth="0.5"
            strokeDasharray="3 3"
            initial={{ pathLength: 0 }}
            whileInView={{ pathLength: 1 }}
            viewport={{ once: true }}
            transition={{ duration: 1.5, delay: 0.5, ease: "easeOut" }}
          />
          {/* Arrow head */}
          <motion.polygon
            points="17,3 22,5 17,7"
            fill="rgba(255,255,255,0.08)"
            initial={{ opacity: 0 }}
            whileInView={{ opacity: 1 }}
            viewport={{ once: true }}
            transition={{ delay: 2, duration: 0.3 }}
          />
        </svg>
      </motion.div>
    </section>
  );
}
