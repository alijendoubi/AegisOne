const STEPS = [
  {
    title: "Detect",
    description:
      "Behavioral signals + canary files + script inspection identify encryption bursts and recovery tampering.",
  },
  {
    title: "Act",
    description:
      "Kill malicious processes, block outbound, and roll back impacted files from safe snapshots.",
  },
  {
    title: "Explain",
    description:
      "Generate a plain-language incident report with a timeline, actions taken, and recommended follow-ups.",
  },
];

export default function HowItWorks() {
  return (
    <section id="how" className="py-16 reveal">
      <div className="max-w-[1180px] mx-auto px-5">
        <h2 className="text-2xl md:text-3xl font-bold mb-2">How it works</h2>
        <p className="mb-8" style={{ color: "#9fb3c8" }}>
          Detect → Act → Explain. Anchored to known adversary behaviors
          (ATT&amp;CK T1486, T1490, T1041).
        </p>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-[18px] [counter-reset:step]">
          {STEPS.map(({ title, description }, idx) => (
            <div
              key={title}
              className="rounded-card p-5"
              style={{
                background: "var(--panel)",
                border: "1px solid var(--border)",
                boxShadow: "var(--shadow)",
              }}
            >
              <div className="flex items-center gap-2 mb-2">
                <span
                  className="inline-grid place-items-center w-[26px] h-[26px] rounded-full text-xs font-extrabold flex-shrink-0"
                  style={{
                    color: "#00e6ff",
                    border: "1px solid var(--border)",
                    background: "var(--panel)",
                  }}
                >
                  {idx + 1}
                </span>
                <h3 className="text-[18px] font-bold">{title}</h3>
              </div>
              <p className="text-sm" style={{ color: "#9fb3c8" }}>
                {description}
              </p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
