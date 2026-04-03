const QA = [
  {
    q: 'Do you "hack back" against attackers?',
    a: "No. AegisOne is defensive-only: detect, isolate, block C2, kill process trees, and roll back files. We never engage in counter-intrusion.",
  },
  {
    q: "Is my data sent to the cloud?",
    a: "By default, analysis occurs on-device. Optional, opt-in telemetry helps improve detection quality. You're always in control.",
  },
  {
    q: "Which OS do you support?",
    a: "Windows v1 (Beta) is available; macOS v1.1 is on waitlist. Linux and mobile are on the roadmap.",
  },
  {
    q: "Does it work with Microsoft Defender?",
    a: "Yes. AegisOne runs alongside Defender. We can guide Controlled Folder Access setup to reduce noise and conflicts.",
  },
  {
    q: "What happens if rollback snapshots are deleted?",
    a: "We monitor and block common shadow-copy deletion attempts. If snapshots are missing, we still contain the threat and preserve what we can.",
  },
  {
    q: "How do you explain incidents?",
    a: "A local LLM turns structured telemetry into a plain-language summary and checklist. No sensitive contents are sent off device by default.",
  },
];

export default function FAQ() {
  return (
    <section id="faq" className="py-16 reveal">
      <div className="max-w-[1180px] mx-auto px-5">
        <h2 className="text-2xl md:text-3xl font-bold mb-2">FAQ</h2>
        <p className="mb-8" style={{ color: "#9fb3c8" }}>
          If you have other questions, reach out any time.
        </p>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-3.5">
          {QA.map(({ q, a }) => (
            <details
              key={q}
              className="rounded-card group"
              style={{
                background: "var(--panel)",
                border: "1px solid var(--border)",
              }}
            >
              <summary className="flex justify-between items-center w-full p-4 cursor-pointer list-none font-semibold text-[#e6eef7] select-none">
                <span>{q}</span>
                <span
                  className="ml-4 flex-shrink-0 transition-transform group-open:rotate-45 text-[#9fb3c8]"
                  aria-hidden="true"
                >
                  ＋
                </span>
              </summary>
              <p className="px-4 pb-4 text-sm" style={{ color: "#9fb3c8" }}>
                {a}
              </p>
            </details>
          ))}
        </div>
      </div>
    </section>
  );
}
