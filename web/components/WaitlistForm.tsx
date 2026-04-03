"use client";

import { useState } from "react";
import Turnstile from "react-turnstile";

type FormState = "idle" | "loading" | "success" | "error";

export default function WaitlistForm() {
  const [email, setEmail] = useState("");
  const [platform, setPlatform] = useState("Windows");
  const [token, setToken] = useState<string | null>(null);
  const [state, setState] = useState<FormState>("idle");
  const [message, setMessage] = useState("");

  const siteKey = process.env.NEXT_PUBLIC_TURNSTILE_SITE_KEY ?? "";

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();

    if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
      setMessage("Please enter a valid email address.");
      setState("error");
      return;
    }

    if (!token) {
      setMessage("Please complete the security check.");
      setState("error");
      return;
    }

    setState("loading");

    try {
      const res = await fetch("/api/waitlist", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email, platform, token }),
      });

      const data = await res.json();

      if (res.ok) {
        setState("success");
        setMessage("You're on the list. We'll email you shortly.");
        setEmail("");
        setPlatform("Windows");
        setToken(null);
      } else {
        setState("error");
        setMessage(data.error ?? "Something went wrong. Please try again.");
      }
    } catch {
      setState("error");
      setMessage("Network error. Please try again.");
    }
  }

  return (
    <form
      onSubmit={handleSubmit}
      className="grid grid-cols-1 sm:grid-cols-2 gap-[18px] max-w-xl mx-auto"
      noValidate
    >
      {/* Email */}
      <div
        className="rounded-card p-5 text-left"
        style={{ background: "var(--panel)", border: "1px solid var(--border)" }}
      >
        <label
          className="block text-sm font-bold mb-2"
          htmlFor="waitlist-email"
        >
          Email
        </label>
        <input
          id="waitlist-email"
          name="email"
          type="email"
          required
          placeholder="you@company.com"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          className="focus-ring w-full px-3 py-3 rounded-[10px] text-sm text-[#e6eef7] transition-colors"
          style={{
            background: "var(--panel)",
            border: "1px solid var(--border)",
            outline: "none",
          }}
          aria-required="true"
        />
      </div>

      {/* Platform */}
      <div
        className="rounded-card p-5 text-left"
        style={{ background: "var(--panel)", border: "1px solid var(--border)" }}
      >
        <label
          className="block text-sm font-bold mb-2"
          htmlFor="waitlist-platform"
        >
          Platform
        </label>
        <select
          id="waitlist-platform"
          name="platform"
          value={platform}
          onChange={(e) => setPlatform(e.target.value)}
          className="focus-ring w-full px-3 py-3 rounded-[10px] text-sm text-[#e6eef7] transition-colors"
          style={{
            background: "var(--panel)",
            border: "1px solid var(--border)",
            outline: "none",
          }}
        >
          <option value="Windows">Windows</option>
          <option value="macOS">macOS (Waitlist)</option>
          <option value="Linux">Linux (Future)</option>
        </select>
      </div>

      {/* Turnstile */}
      {siteKey && (
        <div className="col-span-full flex justify-center">
          <Turnstile
            sitekey={siteKey}
            onVerify={(t) => setToken(t)}
            onExpire={() => setToken(null)}
            theme="dark"
          />
        </div>
      )}

      {/* Submit */}
      <div className="col-span-full flex justify-center">
        <button
          type="submit"
          disabled={state === "loading"}
          className="focus-ring inline-flex items-center px-6 py-3 rounded-xl text-sm font-bold text-[#e6eef7] transition-transform hover:-translate-y-px disabled:opacity-50 disabled:cursor-not-allowed"
          style={{
            background:
              "linear-gradient(135deg, rgba(0,230,255,0.15), rgba(124,58,237,0.15))",
            border: "1px solid rgba(0,230,255,0.35)",
          }}
        >
          {state === "loading" ? "Adding to waitlist…" : "Request Access"}
        </button>
      </div>

      {/* Toast */}
      {(state === "success" || state === "error") && (
        <div
          className="col-span-full text-center text-sm px-4 py-3 rounded-lg transition-all"
          style={{
            color: state === "success" ? "#10b981" : "#ef4444",
            background:
              state === "success"
                ? "rgba(16,185,129,0.1)"
                : "rgba(239,68,68,0.1)",
            border: `1px solid ${state === "success" ? "#10b981" : "#ef4444"}`,
          }}
          role="alert"
        >
          {message}
        </div>
      )}
    </form>
  );
}
