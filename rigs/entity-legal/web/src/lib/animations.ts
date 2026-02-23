export const transitions = {
  fast: { duration: 0.15, ease: "easeOut" as const },
  default: { duration: 0.3, ease: "easeOut" as const },
  slow: { duration: 0.6, ease: [0.16, 1, 0.3, 1] as const },
  spring: { type: "spring" as const, stiffness: 300, damping: 30 },
};

export const fadeUp = {
  initial: { opacity: 0, y: 20 },
  animate: { opacity: 1, y: 0 },
};

export const fadeIn = {
  initial: { opacity: 0 },
  animate: { opacity: 1 },
};

export const staggerContainer = {
  animate: {
    transition: {
      staggerChildren: 0.1,
    },
  },
};

export const staggerItem = {
  initial: { opacity: 0, y: 20 },
  animate: {
    opacity: 1,
    y: 0,
    transition: transitions.default,
  },
};
