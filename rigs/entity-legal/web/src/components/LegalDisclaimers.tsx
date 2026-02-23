"use client";

import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { ChevronDown } from "lucide-react";
import { transitions } from "@/lib/animations";

interface Disclaimer {
  title: string;
  body: string;
}

const disclaimers: Disclaimer[] = [
  {
    title: "Marshall Islands DAO LLC Act 2022",
    body: "entity.legal facilitates entity formation under the Republic of the Marshall Islands Decentralized Autonomous Organization Act 2022 (52 MIRC Ch. 7). All DAO LLCs formed through entity.legal are registered with the Registrar of Resident Domestic and Authorized Foreign Corporations of the Republic of the Marshall Islands.\n\nThe DAO Act recognizes decentralized autonomous organizations as limited liability companies provided they include specific statements in their certificate of formation and LLC agreement identifying the organization as a DAO. entity.legal ensures all formation documents comply with these requirements.\n\nThe Republic of the Marshall Islands is a sovereign nation. Laws and regulations may change. entity.legal does not guarantee the future regulatory treatment of DAO LLCs in any jurisdiction.",
  },
  {
    title: "Smart contract address as legal record",
    body: "When you form a DAO LLC through entity.legal, your LLC agreement designates a specific Solana smart contract address as the authoritative membership registry. This means:\n\n1. Ownership of membership interests is determined by SPL token holdings at the designated smart contract address.\n2. Transfers of SPL tokens at this address constitute legally binding transfers of membership interests, subject to any transfer restrictions in your LLC agreement.\n3. The on-chain state of the smart contract is the legal record of your cap table.\n\nSmart contracts are software. Software can have bugs. entity.legal does not audit or warranty the Squads Protocol smart contracts. Squads Protocol has been independently audited by OtterSec, Neodyme, and Bramah Systems and is the first formally verified program on Solana. However, no audit eliminates all risk.\n\nYou are responsible for the security of your private keys. entity.legal does not have access to your keys and cannot recover lost keys or reverse transactions.",
  },
  {
    title: "Not legal or tax advice",
    body: "entity.legal is an entity formation service. We are not a law firm. The information on this website and the services we provide do not constitute legal advice, tax advice, investment advice, or any other form of professional advice.\n\nFormation of a Marshall Islands DAO LLC does not exempt you from applicable laws in your jurisdiction of residence or operation. You are responsible for understanding and complying with all laws that apply to you, your organization, and your activities.\n\nWe strongly recommend consulting with qualified legal and tax professionals in your jurisdiction before forming any entity. entity.legal\u2019s services are limited to entity formation, on-chain infrastructure deployment, and annual compliance maintenance.\n\nNothing on this website should be construed as a solicitation or offer to sell securities. Membership interests in a DAO LLC may constitute securities under the laws of certain jurisdictions. Consult a qualified securities attorney before issuing or transferring membership interests.",
  },
  {
    title: "Know Your Customer (KYC) and Anti-Money Laundering (AML)",
    body: "In accordance with Marshall Islands law, founding members of DAO LLCs with 25% or more governance rights must complete Know Your Customer (KYC) verification. This includes providing proof of identity and proof of residential address.\n\nentity.legal collects KYC information solely for the purpose of entity formation compliance. KYC data is processed by our verified KYC provider, encrypted at rest, and never shared with third parties except as required by law.\n\nentity.legal reserves the right to refuse service to any individual or organization that fails KYC verification or that we reasonably believe may be involved in money laundering, terrorism financing, or other illicit activity.",
  },
];

function DisclaimerItem({ disclaimer }: { disclaimer: Disclaimer }) {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <div className="border-b border-border">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="flex w-full items-center justify-between py-5 text-left transition-colors hover:text-accent"
        aria-expanded={isOpen}
      >
        <span className="pr-4 text-base font-medium text-text-primary">
          {disclaimer.title}
        </span>
        <motion.div
          animate={{ rotate: isOpen ? 180 : 0 }}
          transition={transitions.fast}
        >
          <ChevronDown className="h-5 w-5 shrink-0 text-text-tertiary" />
        </motion.div>
      </button>
      <AnimatePresence initial={false}>
        {isOpen && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: "auto", opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={transitions.default}
            className="overflow-hidden"
          >
            <div className="space-y-3 pb-6 text-sm leading-relaxed text-text-secondary">
              {disclaimer.body.split("\n\n").map((paragraph, i) => (
                <p key={i}>{paragraph}</p>
              ))}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

export function LegalDisclaimers() {
  return (
    <section className="border-t border-border bg-bg-secondary px-6 py-20">
      <div className="mx-auto max-w-[800px]">
        <h2 className="text-2xl font-semibold text-text-primary">
          Legal Disclosures
        </h2>
        <div className="mt-8">
          {disclaimers.map((disclaimer) => (
            <DisclaimerItem key={disclaimer.title} disclaimer={disclaimer} />
          ))}
        </div>
      </div>
    </section>
  );
}
