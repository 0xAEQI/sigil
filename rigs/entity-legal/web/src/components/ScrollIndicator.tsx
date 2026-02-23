"use client";

import { motion, AnimatePresence } from "framer-motion";
import { ChevronDown } from "lucide-react";
import { useScrollPast } from "@/lib/hooks";

export function ScrollIndicator() {
  const scrolledPast = useScrollPast(100);

  return (
    <AnimatePresence>
      {!scrolledPast && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.3 }}
          className="absolute bottom-8 left-1/2 -translate-x-1/2"
        >
          <motion.div
            animate={{ y: [0, 8, 0] }}
            transition={{
              duration: 2,
              ease: "easeInOut",
              repeat: Infinity,
            }}
          >
            <ChevronDown className="h-6 w-6 text-text-tertiary" />
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
