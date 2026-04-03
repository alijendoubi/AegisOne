export default function Footer() {
  return (
    <footer
      className="border-t py-7"
      style={{ borderColor: "var(--border)", color: "#9fb3c8" }}
    >
      <div className="max-w-[1180px] mx-auto px-5 flex flex-wrap justify-between items-center gap-4">
        <div className="text-sm">
          &copy; {new Date().getFullYear()} AegisOne. All rights reserved.
        </div>
        <nav className="flex items-center gap-4" aria-label="Footer navigation">
          {["Security", "Privacy", "Terms"].map((label) => (
            <a
              key={label}
              href="#"
              className="focus-ring text-sm text-[#9fb3c8] hover:text-[#e6eef7] transition-colors"
            >
              {label}
            </a>
          ))}
        </nav>
      </div>
    </footer>
  );
}
