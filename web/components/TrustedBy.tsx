export default function TrustedBy() {
  return (
    <section className="py-8" aria-label="Trusted by">
      <div className="max-w-[1180px] mx-auto px-5">
        <div className="grid grid-cols-3 md:grid-cols-6 gap-3.5 opacity-90">
          {Array.from({ length: 6 }).map((_, i) => (
            <div
              key={i}
              className="h-[50px] rounded-xl flex items-center justify-center text-sm"
              style={{
                border: "1px dashed var(--border)",
                color: "#9fb3c8",
              }}
            >
              LOGO
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
