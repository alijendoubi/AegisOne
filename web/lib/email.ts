import { Resend } from "resend";

const resend = new Resend(process.env.RESEND_API_KEY);

const FROM = `${process.env.RESEND_FROM_NAME ?? "AegisOne"} <${
  process.env.RESEND_FROM_EMAIL ?? "noreply@aegisone.com"
}>`;

const SITE_URL = process.env.NEXT_PUBLIC_SITE_URL ?? "https://aegisone.com";

export async function sendWelcomeEmail(
  email: string,
  platform: string
): Promise<boolean> {
  const firstName = email.split("@")[0] ?? "there";
  const name = firstName.length > 1 ? capitalise(firstName) : "there";

  const { error } = await resend.emails.send({
    from: FROM,
    to: email,
    subject: "You're on the AegisOne waitlist",
    html: buildHtml(name, platform),
    text: buildText(name, platform),
  });

  if (error) {
    console.error("[resend] failed to send welcome email:", error);
    return false;
  }
  return true;
}

function capitalise(s: string) {
  return s.charAt(0).toUpperCase() + s.slice(1).toLowerCase();
}

function buildHtml(name: string, platform: string) {
  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Welcome to AegisOne</title>
  <style>
    body { font-family: Inter, -apple-system, sans-serif; line-height:1.6; color:#e6eef7; background:#0b0f17; margin:0; padding:20px; }
    .wrap { max-width:600px; margin:0 auto; background:linear-gradient(180deg,rgba(255,255,255,.06),rgba(255,255,255,.02)); border:1px solid rgba(255,255,255,.08); border-radius:16px; overflow:hidden; }
    .hd { padding:40px 40px 20px; text-align:center; background:linear-gradient(135deg,rgba(0,230,255,.15),rgba(124,58,237,.15)); }
    .logo { width:48px; height:48px; margin:0 auto 16px; border-radius:12px; background:radial-gradient(100% 100% at 30% 30%,#00e6ff 0%,#7c3aed 60%,#0b0f17 100%); }
    .body { padding:30px 40px 40px; }
    h1 { color:#00e6ff; font-size:28px; margin-bottom:16px; font-weight:700; }
    .feature { margin:20px 0; padding:16px; background:rgba(255,255,255,.02); border:1px solid rgba(255,255,255,.04); border-radius:8px; }
    .cta { margin:30px 0; text-align:center; }
    .btn { display:inline-block; padding:14px 28px; background:linear-gradient(135deg,#00e6ff 0%,#7c3aed 100%); color:#fff; text-decoration:none; border-radius:8px; font-weight:600; }
    .ft { padding:20px 40px; border-top:1px solid rgba(255,255,255,.08); font-size:14px; color:#9fb3c8; text-align:center; }
  </style>
</head>
<body>
  <div class="wrap">
    <div class="hd">
      <div class="logo"></div>
      <h1>Welcome to AegisOne</h1>
      <span style="display:inline-block;padding:6px 12px;background:rgba(0,230,255,.15);color:#00e6ff;border-radius:20px;font-size:12px;font-weight:600">AI Security Agent</span>
    </div>
    <div class="body">
      <p>Hi ${name},</p>
      <p>Thanks for joining the <strong>AegisOne waitlist</strong>! You're part of the next generation of cybersecurity.</p>
      <div class="feature"><strong>🛡️ Kill-Switch Speed</strong> — Stop ransomware process trees in seconds</div>
      <div class="feature"><strong>♻️ Snapshot Rollback</strong> — Restore files instantly from safe snapshots</div>
      <div class="feature"><strong>🧠 Explainable AI</strong> — Get plain-language incident reports</div>
      <div class="feature"><strong>🔒 Privacy-First</strong> — Everything runs on your device</div>
      <p><strong>What's next?</strong></p>
      <ul>
        <li>We'll keep you updated on our progress</li>
        <li>You'll get early access when ${platform} support is ready</li>
        <li>Be first to know about beta releases</li>
      </ul>
      <div class="cta">
        <a href="${SITE_URL}" class="btn">Visit AegisOne</a>
      </div>
      <p><em>AegisOne is defensive-only. We never hack back.</em></p>
    </div>
    <div class="ft">AegisOne &bull; AI Security Agent<br />You can unsubscribe from these emails at any time.</div>
  </div>
</body>
</html>`;
}

function buildText(name: string, platform: string) {
  return `Welcome to AegisOne!

Hi ${name},

Thanks for joining the AegisOne waitlist! You're part of the next generation of cybersecurity.

Features:
- Kill-Switch Speed: Stop ransomware process trees in seconds
- Snapshot Rollback: Restore files instantly from safe snapshots
- Explainable AI: Get plain-language incident reports
- Privacy-First: Everything runs on your device

What's next?
- We'll keep you updated on our progress
- You'll get early access when ${platform} support is ready
- Be first to know about beta releases

Visit us: ${SITE_URL}

AegisOne is defensive-only. We never hack back.

— The AegisOne Team
`;
}
