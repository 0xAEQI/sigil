import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";

const inter = Inter({
  variable: "--font-inter",
  subsets: ["latin"],
  display: "swap",
});

export const metadata: Metadata = {
  title:
    "entity.legal — Marshall Islands DAO LLC Formation with On-Chain Cap Tables on Solana",
  description:
    "Form a Marshall Islands DAO LLC with on-chain cap tables on Solana. SPL token membership interests. Squads Protocol governance. No US regulatory overhead. Your keys, your entity.",
  metadataBase: new URL("https://entity.legal"),
  openGraph: {
    title:
      "entity.legal — Sovereign Entity Formation for the On-Chain Era",
    description:
      "Marshall Islands DAO LLC formation with on-chain cap tables on Solana. Real legal structure. No custody middleware. Your keys, your entity.",
    type: "website",
    url: "https://entity.legal",
    siteName: "entity.legal",
    images: [
      {
        url: "/og-image.png",
        width: 1200,
        height: 630,
        alt: "entity.legal — Sovereign entity formation for the on-chain era",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    site: "@entitylegal",
    title:
      "entity.legal — Sovereign Entity Formation for the On-Chain Era",
    description:
      "Marshall Islands DAO LLC + Solana on-chain cap tables. No US regulatory overhead. Your keys, your entity.",
    images: ["/og-image.png"],
  },
  alternates: {
    canonical: "https://entity.legal",
  },
  robots: {
    index: true,
    follow: true,
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className={inter.variable}>
      <head>
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{
            __html: JSON.stringify({
              "@context": "https://schema.org",
              "@type": "ProfessionalService",
              name: "entity.legal",
              description:
                "Marshall Islands DAO LLC formation with on-chain cap tables on Solana",
              url: "https://entity.legal",
              serviceType: "Business Formation",
              areaServed: "Worldwide",
              brand: {
                "@type": "Brand",
                name: "entity.legal",
                slogan:
                  "Sovereign entity formation for the on-chain era",
              },
              offers: [
                {
                  "@type": "Offer",
                  name: "Starter — DAO LLC Formation",
                  price: "5500",
                  priceCurrency: "USD",
                  description:
                    "Marshall Islands DAO LLC with on-chain cap table on Solana",
                },
                {
                  "@type": "Offer",
                  name: "Pro — DAO LLC + Series LLC",
                  price: "8500",
                  priceCurrency: "USD",
                  description:
                    "DAO LLC with Series LLC capability, custom governance, and compliance automation",
                },
              ],
            }),
          }}
        />
      </head>
      <body className="antialiased">{children}</body>
    </html>
  );
}
