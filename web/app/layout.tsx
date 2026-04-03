import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "AegisOne — Your AI Security Agent",
  description:
    "AegisOne is your on-device AI Security Agent. It stops ransomware in seconds, rolls your files back, and explains incidents in plain language — privacy-first.",
  openGraph: {
    title: "AegisOne — Your AI Security Agent",
    description:
      "Stops ransomware in seconds. Rolls files back. Explains what happened. On your device.",
    type: "website",
  },
  other: {
    "application-name": "AegisOne",
  },
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <head>
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{
            __html: JSON.stringify({
              "@context": "https://schema.org",
              "@type": "SoftwareApplication",
              name: "AegisOne — AI Security Agent",
              applicationCategory: "SecurityApplication",
              operatingSystem: "Windows 10+, macOS 12+",
              description:
                "Stops ransomware in seconds, rolls files back, and explains what happened — on-device AI, privacy-first.",
              offers: { "@type": "Offer", price: "0", priceCurrency: "EUR" },
            }),
          }}
        />
      </head>
      <body>{children}</body>
    </html>
  );
}
