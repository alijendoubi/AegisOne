import { NextRequest, NextResponse } from "next/server";
import { createServerClient } from "@/lib/supabase";
import { sendWelcomeEmail } from "@/lib/email";

const EMAIL_RE = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
const VALID_PLATFORMS = ["Windows", "macOS", "Linux"];
const TURNSTILE_VERIFY_URL =
  "https://challenges.cloudflare.com/turnstile/v0/siteverify";

async function verifyTurnstile(token: string): Promise<boolean> {
  const secret = process.env.TURNSTILE_SECRET_KEY;
  if (!secret) {
    // Skip verification in dev when secret is not configured
    return process.env.NODE_ENV !== "production";
  }

  const res = await fetch(TURNSTILE_VERIFY_URL, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ secret, response: token }),
  });
  const data = (await res.json()) as { success: boolean };
  return data.success === true;
}

export async function POST(req: NextRequest) {
  // Parse body
  let body: { email?: unknown; platform?: unknown; token?: unknown };
  try {
    body = await req.json();
  } catch {
    return NextResponse.json({ error: "Invalid request body" }, { status: 400 });
  }

  const email = typeof body.email === "string" ? body.email.trim().toLowerCase() : "";
  const platform =
    typeof body.platform === "string" && VALID_PLATFORMS.includes(body.platform)
      ? body.platform
      : "Windows";
  const token = typeof body.token === "string" ? body.token : "";

  // Validate email
  if (!EMAIL_RE.test(email)) {
    return NextResponse.json({ error: "Invalid email address" }, { status: 422 });
  }

  // Verify Turnstile
  const turnstileOk = await verifyTurnstile(token);
  if (!turnstileOk) {
    return NextResponse.json(
      { error: "Security check failed. Please try again." },
      { status: 422 }
    );
  }

  const db = createServerClient();

  // Check for duplicates
  const { data: existing } = await db
    .from("waitlist")
    .select("id")
    .eq("email", email)
    .maybeSingle();

  if (existing) {
    // Silently succeed — don't leak whether an address is registered
    return NextResponse.json({ ok: true });
  }

  // Insert
  const { error: insertError } = await db.from("waitlist").insert({
    email,
    platform,
    source: "landing-page",
    referrer: req.headers.get("referer") ?? null,
  });

  if (insertError) {
    console.error("[supabase] insert error:", insertError);
    return NextResponse.json(
      { error: "Failed to save. Please try again." },
      { status: 500 }
    );
  }

  // Send welcome email (fire-and-forget — don't block the response)
  sendWelcomeEmail(email, platform).catch((err) =>
    console.error("[email] failed:", err)
  );

  return NextResponse.json({ ok: true });
}
