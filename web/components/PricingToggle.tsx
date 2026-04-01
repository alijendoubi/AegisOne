"use client";

import { useState } from "react";

type Tier = {
  name: string;
  monthly: string;
  annual: string;
  highlight: boolean;
  features: { included: boolean; text: string }[];
  cta: string;
};

export default function PricingToggle({ tiers }: { tiers: Tier[] }) {
  const [annual, setAnnual] = useState(false);

  return (
    <>
      {/* Toggle */}
      <div className="flex items-center justify-center gap-2.5 mb-[18px]">
        <span className="text-sm" style={{ color: "#9fb3c8" }}>
          Monthly
        </span>
        <label className="flex items-center cursor-pointer">
          <span className="sr-only">Toggle annual billing</span>
          <input
            type="checkbox"
            className="billing-toggle focus-ring"
            checked={annual}
            onChange={(e) => setAnnual(e.target.checked)}
            aria-checked={annual}
            role="switch"
          />
        </label>
        <span className="text-sm" style={{ color: "#9fb3c8" }}>
          Annual <span className="text-[#10b981] font-semibold">(save 20%)</span>
        </span>
      </div>

      {/* Tiers */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-[18px]">
        {tiers.map((tier) => (
          <div
            key={tier.name}
            className="rounded-card p-[22px]"
            style={{
              background:
                "linear-gradient(180deg, rgba(255,255,255,0.05), rgba(255,255,255,0.02))",
              border: tier.highlight
                ? "1px solid rgba(0,230,255,0.35)"
                : "1px solid var(--border)",
              boxShadow: "var(--shadow)",
            }}
          >
            <span
              className="text-xs font-semibold uppercase tracking-wider"
              style={{ color: "#9fb3c8" }}
            >
              {tier.name}
            </span>
            <div className="text-[36px] font-extrabold mt-2.5 mb-2.5">
              {annual ? tier.annual : tier.monthly}
              {tier.monthly !== "€0" && (
                <span className="text-base font-normal" style={{ color: "#9fb3c8" }}>
                  /mo
                </span>
              )}
            </div>

            <ul className="list-none p-0 m-0 mt-3">
              {tier.features.map(({ included, text }) => (
                <li key={text} className="flex gap-2.5 items-start my-2 text-sm">
                  <span
                    className="font-black flex-shrink-0 mt-0.5"
                    style={{ color: included ? "#10b981" : "#9fb3c8" }}
                  >
                    {included ? "✔" : "—"}
                  </span>
                  <span style={{ color: included ? "#e6eef7" : "#9fb3c8" }}>
                    {text}
                  </span>
                </li>
              ))}
            </ul>

            <div className="mt-3.5">
              <a
                href="#download"
                className="focus-ring inline-flex items-center px-4 py-2.5 rounded-xl text-sm font-bold text-[#e6eef7] transition-transform hover:-translate-y-px"
                style={
                  tier.highlight
                    ? {
                        background:
                          "linear-gradient(135deg, rgba(0,230,255,0.15), rgba(124,58,237,0.15))",
                        border: "1px solid rgba(0,230,255,0.35)",
                      }
                    : {
                        background: "var(--panel)",
                        border: "1px solid var(--border)",
                      }
                }
              >
                {tier.cta}
              </a>
            </div>
          </div>
        ))}
      </div>
    </>
  );
}
