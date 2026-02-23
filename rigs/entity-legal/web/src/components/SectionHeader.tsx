"use client";

import { motion } from "framer-motion";
import { fadeUp, transitions } from "@/lib/animations";
import { useInView, useReducedMotion } from "@/lib/hooks";

interface SectionHeaderProps {
  eyebrow?: string;
  title: string;
  subtitle?: string;
  maxWidth?: string;
}

export function SectionHeader({
  eyebrow,
  title,
  subtitle,
  maxWidth = "640px",
}: SectionHeaderProps) {
  const { ref, isInView } = useInView(0.2);
  const reduced = useReducedMotion();

  return (
    <div ref={ref} className="mb-16">
      {eyebrow && (
        <motion.p
          initial={reduced ? false : fadeUp.initial}
          animate={isInView ? fadeUp.animate : fadeUp.initial}
          transition={transitions.default}
          className="text-[13px] font-medium uppercase tracking-[4px] text-accent"
        >
          {eyebrow}
        </motion.p>
      )}
      <motion.h2
        initial={reduced ? false : fadeUp.initial}
        animate={isInView ? fadeUp.animate : fadeUp.initial}
        transition={{ ...transitions.default, delay: 0.1 }}
        className="mt-3 text-[28px] font-bold leading-tight tracking-tight text-text-primary md:text-[36px]"
      >
        {title}
      </motion.h2>
      {subtitle && (
        <motion.p
          initial={reduced ? false : fadeUp.initial}
          animate={isInView ? fadeUp.animate : fadeUp.initial}
          transition={{ ...transitions.default, delay: 0.2 }}
          className="mt-4 text-base leading-relaxed text-text-secondary"
          style={{ maxWidth }}
        >
          {subtitle}
        </motion.p>
      )}
    </div>
  );
}
