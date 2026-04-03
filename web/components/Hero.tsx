import TerminalHero from "./TerminalHero";

export default function Hero() {
  return (
    <section className="relative overflow-hidden pt-[72px] pb-8">
      {/* Animated grid */}
      <div className="grid-bg absolute inset-0 pointer-events-none" aria-hidden="true" />

      {/* Halo glow */}
      <div
        className="absolute pointer-events-none"
        style={{
          top: "-200px",
          left: "50%",
          width: "900px",
          height: "900px",
          translate: "-50% 0",
          background:
            "radial-gradient(closest-side, rgba(0,230,255,0.16), transparent 60%)",
          filter: "blur(40px)",
        }}
        aria-hidden="true"
      />

      <div className="max-w-[1180px] mx-auto px-5 relative z-10">
        <div className="grid grid-cols-1 lg:grid-cols-[1.15fr_0.85fr] gap-8 items-center">
          {/* Copy */}
          <div>
            <h1
              className="font-bold leading-tight mb-4"
              style={{ fontSize: "clamp(32px, 4vw, 56px)", margin: "0 0 16px" }}
            >
              AegisOne — Your AI Security Agent
            </h1>
            <p className="text-lg mb-6" style={{ color: "#9fb3c8" }}>
              Stops ransomware in seconds.{" "}
              <strong className="text-[#e6eef7]">Rolls your files back</strong>.
              Explains exactly what happened — all on your device, privacy-first.
            </p>

            {/* Badges */}
            <div className="flex flex-wrap gap-2 mb-6">
              {[
                "Windows v1 — Beta",
                "macOS v1.1 — Waitlist",
                "Defensive-only · No Hack-Back",
              ].map((badge) => (
                <span
                  key={badge}
                  className="px-3 py-1.5 rounded-full text-sm"
                  style={{
                    border: "1px solid var(--border)",
                    color: "#c6d2df",
                    background: "var(--panel)",
                  }}
                >
                  {badge}
                </span>
              ))}
            </div>

            {/* CTA buttons */}
            <div className="flex flex-wrap gap-2.5">
              <a
                href="#download"
                className="focus-ring inline-flex items-center gap-2.5 px-4 py-3 rounded-xl font-bold text-sm text-[#e6eef7] transition-transform hover:-translate-y-px"
                style={{
                  background:
                    "linear-gradient(135deg, rgba(0,230,255,0.15), rgba(124,58,237,0.15))",
                  border: "1px solid rgba(0,230,255,0.35)",
                  boxShadow: "var(--shadow)",
                }}
              >
                Download for Windows
              </a>
              <a
                href="#demo"
                className="focus-ring inline-flex items-center gap-2.5 px-4 py-3 rounded-xl font-bold text-sm text-[#e6eef7] transition-transform hover:-translate-y-px"
                style={{
                  background: "var(--panel)",
                  border: "1px solid var(--border)",
                  boxShadow: "var(--shadow)",
                }}
              >
                Live Demo
              </a>
            </div>
          </div>

          {/* Terminal */}
          <TerminalHero />
        </div>
      </div>
    </section>
  );
}
