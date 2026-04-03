import DemoTerminal from "./DemoTerminal";

const STATUS_PILLS = [
  { label: "Real-time Monitoring", status: "green" },
  { label: "Kill-Switch", status: "green" },
  { label: "Snapshot Rollback", status: "green" },
  { label: "Explainable AI", status: "green" },
  { label: "Privacy-First", status: "green" },
  { label: "macOS Waitlist", status: "yellow" },
];

export default function Demo() {
  return (
    <section id="demo" className="py-16 reveal">
      <div className="max-w-[1180px] mx-auto px-5 grid grid-cols-1 lg:grid-cols-[1.2fr_0.8fr] gap-[18px]">
        {/* Terminal panel */}
        <div
          className="rounded-card p-[18px]"
          style={{
            background: "var(--panel)",
            border: "1px solid var(--border)",
            boxShadow: "var(--shadow)",
          }}
        >
          <h3 className="text-lg font-bold mb-1">Live Incident Demo</h3>
          <p className="text-sm mb-4" style={{ color: "#9fb3c8" }}>
            Simulated event to showcase the agent&apos;s sequence of actions.
          </p>
          <DemoTerminal />
        </div>

        {/* Status panel */}
        <div
          className="rounded-card p-[18px]"
          style={{
            background: "var(--panel)",
            border: "1px solid var(--border)",
            boxShadow: "var(--shadow)",
          }}
        >
          <h3 className="text-lg font-bold mb-3">Status &amp; Capabilities</h3>
          <div className="flex flex-wrap gap-2.5">
            {STATUS_PILLS.map(({ label, status }) => (
              <span
                key={label}
                className="px-3 py-1.5 rounded-full text-sm font-bold"
                style={{
                  background: "var(--panel)",
                  border: "1px solid var(--border)",
                  color: "#c6d2df",
                }}
              >
                {status === "green" ? "🟢" : "🟡"} {label}
              </span>
            ))}
          </div>
          <p className="mt-3.5 text-sm" style={{ color: "#9fb3c8" }}>
            AegisOne is <strong className="text-[#e6eef7]">defensive-only</strong>. We
            don&apos;t hack back. We focus on protecting your devices and data.
          </p>
        </div>
      </div>
    </section>
  );
}
