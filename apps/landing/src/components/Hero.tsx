import { motion } from "framer-motion";

const words = [
  { text: "Autonomous", direction: 1 },
  { text: "Execution", direction: -1 },
  { text: "Quantified", direction: 1 },
  { text: "Intelligence", direction: -1 },
];

export function Hero() {
  return (
    <section className="relative min-h-screen flex items-center justify-center overflow-hidden">
      <style>{`
        @keyframes drift-right {
          0%, 100% { transform: translateX(0px); }
          50% { transform: translateX(40px); }
        }
        @keyframes drift-left {
          0%, 100% { transform: translateX(0px); }
          50% { transform: translateX(-40px); }
        }
      `}</style>
      <motion.h1
        className="leading-[1.1] text-left select-none"
        style={{ fontFamily: "'Space Grotesk', sans-serif" }}
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 1, ease: "easeOut" }}
      >
        {words.map((w, i) => (
          <motion.span
            className="block whitespace-nowrap text-4xl md:text-6xl lg:text-[72px] font-bold tracking-tight"
            key={i}
            initial={{ opacity: 0, x: w.direction * -120 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{
              duration: 1.1,
              ease: [0.16, 1, 0.3, 1],
              delay: 0.1 + i * 0.12,
            }}
            style={{
              animation: `${w.direction > 0 ? "drift-right" : "drift-left"} ${6 + i * 1.5}s ease-in-out infinite`,
              animationDelay: `${1.5 + i * 0.4}s`,
            }}
          >
            <span className="text-white">{w.text[0]}</span>
            <span className="text-white/20">{w.text.slice(1)}</span>
          </motion.span>
        ))}
      </motion.h1>
    </section>
  );
}
