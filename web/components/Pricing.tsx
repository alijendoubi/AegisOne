import PricingToggle from "./PricingToggle";

const TIERS = [
  {
    name: "Free",
    monthly: "€0",
    annual: "€0",
    highlight: false,
    features: [
      { included: true, text: "Real-time detection" },
      { included: true, text: "Manual isolate & kill" },
      { included: true, text: "Incident explainers" },
      { included: false, text: "Snapshot rollback" },
    ],
    cta: "Get Started",
  },
  {
    name: "Pro",
    monthly: "€8",
    annual: "€6",
    highlight: true,
    features: [
      { included: true, text: "Auto-respond & isolate" },
      { included: true, text: "Snapshot rollback" },
      { included: true, text: "Advanced explainers" },
      { included: true, text: "Email alerts" },
    ],
    cta: "Start Pro",
  },
  {
    name: "Business",
    monthly: "€12",
    annual: "€9",
    highlight: false,
    features: [
      { included: true, text: "Multi-device policies" },
      { included: true, text: "Slack/Email alerts" },
      { included: true, text: "Priority support" },
      { included: true, text: "macOS + Windows" },
    ],
    cta: "Contact Sales",
  },
];

export default function Pricing() {
  return (
    <section id="pricing" className="py-16 reveal">
      <div className="max-w-[1180px] mx-auto px-5">
        <h2 className="text-2xl md:text-3xl font-bold mb-2">
          Simple, transparent pricing
        </h2>
        <p className="mb-3" style={{ color: "#9fb3c8" }}>
          Start free. Upgrade when you need automated response and rollback.
        </p>

        {/* Client-side billing toggle + price display */}
        <PricingToggle tiers={TIERS} />
      </div>
    </section>
  );
}
