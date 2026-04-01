import WaitlistForm from "./WaitlistForm";

export default function WaitlistSection() {
  return (
    <section id="download" className="py-16 reveal">
      <div className="max-w-[1180px] mx-auto px-5">
        <div
          className="rounded-[22px] p-7 text-center"
          style={{
            background:
              "linear-gradient(90deg, rgba(0,230,255,0.12), rgba(124,58,237,0.12))",
            border: "1px solid rgba(255,255,255,0.1)",
            boxShadow: "var(--shadow)",
          }}
        >
          <h3 className="text-2xl font-bold mt-0 mb-2">
            Put an AI Security Agent on every device.
          </h3>
          <p className="mb-8" style={{ color: "#9fb3c8" }}>
            Stop ransomware in seconds. Roll back safely. Understand exactly
            what happened.
          </p>
          <WaitlistForm />
        </div>
      </div>
    </section>
  );
}
