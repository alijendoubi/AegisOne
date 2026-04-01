"use client";

import { useEffect, useRef, useState } from "react";

const NAV_LINKS = [
  { href: "#features", label: "Features" },
  { href: "#how", label: "How it works" },
  { href: "#demo", label: "Live demo" },
  { href: "#pricing", label: "Pricing" },
  { href: "#faq", label: "FAQ" },
];

export default function Header() {
  const [scrolled, setScrolled] = useState(false);
  const [menuOpen, setMenuOpen] = useState(false);
  const [activeSection, setActiveSection] = useState("");
  const headerRef = useRef<HTMLElement>(null);

  // Shadow on scroll
  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 8);
    window.addEventListener("scroll", onScroll, { passive: true });
    onScroll();
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  // Active nav section via IntersectionObserver
  useEffect(() => {
    const sections = NAV_LINKS.map(({ href }) =>
      document.querySelector(href)
    ).filter(Boolean) as Element[];

    const io = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            setActiveSection("#" + entry.target.id);
          }
        });
      },
      { threshold: 0.5 }
    );
    sections.forEach((s) => io.observe(s));
    return () => io.disconnect();
  }, []);

  const scrollTo = (href: string) => {
    const el = document.querySelector(href);
    if (el) el.scrollIntoView({ behavior: "smooth", block: "start" });
    setMenuOpen(false);
  };

  return (
    <header
      ref={headerRef}
      className={`sticky top-0 z-50 backdrop-saturate-150 backdrop-blur-md border-b transition-shadow duration-200 ${
        scrolled ? "shadow-[0_6px_30px_rgba(0,0,0,0.35)]" : ""
      }`}
      style={{
        background: scrolled
          ? "rgba(11,15,23,0.92)"
          : "linear-gradient(180deg, rgba(11,15,23,0.85), rgba(11,15,23,0.55))",
        borderBottomColor: "var(--border)",
      }}
    >
      <div className="max-w-[1180px] mx-auto px-5 flex items-center justify-between py-3.5">
        {/* Brand */}
        <div className="flex items-center gap-2.5">
          <div
            className="w-9 h-9 rounded-[10px] flex-shrink-0"
            style={{
              background:
                "radial-gradient(100% 100% at 30% 30%, #00e6ff 0%, #7c3aed 60%, #0b0f17 100%)",
              boxShadow:
                "0 0 20px rgba(0,230,255,0.25), inset 0 0 30px rgba(124,58,237,0.25)",
            }}
            aria-hidden="true"
          />
          <b className="font-bold tracking-tight text-[#e6eef7]">AegisOne</b>
          <span
            className="hidden sm:inline-block px-2 py-0.5 rounded-full text-xs font-semibold"
            style={{
              background: "rgba(0,230,255,0.1)",
              color: "#00e6ff",
              border: "1px solid rgba(0,230,255,0.2)",
            }}
          >
            AI Security Agent
          </span>
        </div>

        {/* Desktop nav */}
        <nav className="hidden md:block" aria-label="Main navigation">
          <ul className="flex gap-4 list-none p-0 m-0">
            {NAV_LINKS.map(({ href, label }) => (
              <li key={href}>
                <button
                  onClick={() => scrollTo(href)}
                  className={`focus-ring relative px-1 py-1.5 font-semibold text-sm transition-colors ${
                    activeSection === href
                      ? "text-[#e6eef7]"
                      : "text-[#c6d2df] hover:text-[#e6eef7]"
                  }`}
                  style={{ background: "none", border: "none", cursor: "pointer" }}
                  aria-current={activeSection === href ? "true" : undefined}
                >
                  {label}
                  {activeSection === href && (
                    <span
                      className="absolute left-0 right-0 bottom-[-6px] h-0.5 rounded-sm"
                      style={{
                        background:
                          "linear-gradient(90deg, #00e6ff, #7c3aed)",
                      }}
                    />
                  )}
                </button>
              </li>
            ))}
          </ul>
        </nav>

        {/* CTA */}
        <div className="flex items-center gap-2.5">
          <button
            onClick={() => scrollTo("#download")}
            className="focus-ring hidden sm:inline-flex items-center px-3.5 py-2.5 rounded-xl text-sm font-bold text-[#e6eef7] transition-transform hover:-translate-y-px"
            style={{
              background: "var(--panel)",
              border: "1px solid var(--border)",
            }}
          >
            macOS Waitlist
          </button>
          <button
            onClick={() => scrollTo("#download")}
            className="focus-ring inline-flex items-center px-3.5 py-2.5 rounded-xl text-sm font-bold text-[#e6eef7] transition-transform hover:-translate-y-px"
            style={{
              background:
                "linear-gradient(135deg, rgba(0,230,255,0.15), rgba(124,58,237,0.15))",
              border: "1px solid rgba(0,230,255,0.35)",
            }}
          >
            Download
          </button>

          {/* Mobile toggle */}
          <button
            className="focus-ring md:hidden inline-flex items-center px-3 py-2 rounded-xl text-sm font-bold text-[#e6eef7]"
            style={{ background: "var(--panel)", border: "1px solid var(--border)" }}
            aria-controls="mobile-menu"
            aria-expanded={menuOpen}
            onClick={() => setMenuOpen((o) => !o)}
          >
            Menu
          </button>
        </div>
      </div>

      {/* Mobile menu */}
      {menuOpen && (
        <div
          id="mobile-menu"
          className="md:hidden border-t px-5 py-3 flex flex-col gap-2"
          style={{ borderColor: "var(--border)", background: "rgba(11,15,23,0.95)" }}
        >
          {NAV_LINKS.map(({ href, label }) => (
            <button
              key={href}
              onClick={() => scrollTo(href)}
              className="text-left py-2 font-semibold text-sm text-[#c6d2df] hover:text-[#e6eef7] transition-colors"
              style={{ background: "none", border: "none", cursor: "pointer" }}
            >
              {label}
            </button>
          ))}
        </div>
      )}
    </header>
  );
}
