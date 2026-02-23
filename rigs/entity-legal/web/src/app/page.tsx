"use client";

import { useState, useRef, useEffect } from "react";
import { Hero } from "@/components/Hero";
import { ArchitectureStack } from "@/components/ArchitectureStack";
import { Features } from "@/components/Features";
import { HowItWorks } from "@/components/HowItWorks";
import { EntityTypes } from "@/components/EntityTypes";
import { Pricing } from "@/components/Pricing";
import { Trust } from "@/components/Trust";
import { LegalDisclaimers } from "@/components/LegalDisclaimers";
import { Footer } from "@/components/Footer";
import { WaitlistModal } from "@/components/WaitlistModal";
import { FloatingCTA } from "@/components/FloatingCTA";
import { config } from "@/lib/config";

export default function Home() {
  const [waitlistOpen, setWaitlistOpen] = useState(false);
  const [floatingVisible, setFloatingVisible] = useState(false);
  const heroRef = useRef<HTMLElement>(null);

  // Listen for custom event from Hero CTA
  useEffect(() => {
    const handler = () => setWaitlistOpen(true);
    document.addEventListener("open-waitlist", handler);
    return () => document.removeEventListener("open-waitlist", handler);
  }, []);

  // Track hero visibility for floating CTA
  useEffect(() => {
    const heroEl = heroRef.current;
    if (!heroEl) return;

    const observer = new IntersectionObserver(
      ([entry]) => {
        setFloatingVisible(!entry.isIntersecting);
      },
      { threshold: 0.1 }
    );

    observer.observe(heroEl);
    return () => observer.disconnect();
  }, []);

  const openWaitlist = () => {
    if (config.waitlistMode) {
      setWaitlistOpen(true);
    }
  };

  return (
    <main>
      <Hero ref={heroRef} onCtaClick={openWaitlist} />
      <ArchitectureStack />
      <Features />
      <HowItWorks />
      <EntityTypes onCtaClick={openWaitlist} />
      <Pricing onCtaClick={openWaitlist} />
      <Trust />
      <LegalDisclaimers />
      <Footer />

      {/* Floating CTA bar */}
      <FloatingCTA visible={floatingVisible} onCtaClick={openWaitlist} />

      {/* Waitlist Modal */}
      <WaitlistModal
        isOpen={waitlistOpen}
        onClose={() => setWaitlistOpen(false)}
      />
    </main>
  );
}
