const FEATURES = [
  {
    icon: "🛡️",
    title: "Kill-Switch Speed",
    description:
      "Terminate ransomware process trees in seconds; isolate host and block C2 during containment.",
  },
  {
    icon: "♻️",
    title: "Snapshot Rollback",
    description:
      "Restore files from safe snapshots (VSS on Windows; APFS on macOS) with one click.",
  },
  {
    icon: "🧠",
    title: "Explainable AI",
    description:
      "A local LLM turns telemetry into a clear incident story: what happened, what changed, what to do next.",
  },
  {
    icon: "🔒",
    title: "Privacy-First",
    description:
      "On-device learning and minimal telemetry by default. Your data stays with you.",
  },
];

export default function Features() {
  return (
    <section id="features" className="py-16 reveal">
      <div className="max-w-[1180px] mx-auto px-5">
        <h2 className="text-2xl md:text-3xl font-bold mb-2">
          Why teams choose AegisOne
        </h2>
        <p className="mb-8" style={{ color: "#9fb3c8" }}>
          Focused excellence: stop encryption, restore clean state, and
          understand the incident — fast.
        </p>

        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-[18px]">
          {FEATURES.map(({ icon, title, description }) => (
            <article
              key={title}
              className="rounded-card p-5"
              style={{
                background: "var(--panel)",
                border: "1px solid var(--border)",
                boxShadow: "var(--shadow)",
              }}
            >
              <div
                className="w-[42px] h-[42px] grid place-items-center rounded-xl mb-2 text-lg"
                style={{
                  background:
                    "radial-gradient(100% 100% at 30% 30%, rgba(0,230,255,0.25), rgba(124,58,237,0.25))",
                  border: "1px solid rgba(255,255,255,0.08)",
                }}
                aria-hidden="true"
              >
                {icon}
              </div>
              <h3 className="text-[18px] font-bold mb-2">{title}</h3>
              <p className="text-sm" style={{ color: "#9fb3c8" }}>
                {description}
              </p>
            </article>
          ))}
        </div>
      </div>
    </section>
  );
}
