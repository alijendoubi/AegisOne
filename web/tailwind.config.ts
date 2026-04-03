import type { Config } from "tailwindcss";

const config: Config = {
  content: [
    "./app/**/*.{ts,tsx}",
    "./components/**/*.{ts,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        bg: "#0b0f17",
        primary: "#00e6ff",
        accent: "#7c3aed",
        success: "#10b981",
        warning: "#f59e0b",
        danger: "#ef4444",
        "text-base": "#e6eef7",
        muted: "#9fb3c8",
        dim: "#c6d2df",
        panel: "rgba(255,255,255,0.04)",
        border: "rgba(255,255,255,0.08)",
      },
      fontFamily: {
        sans: ["Inter", "system-ui", "-apple-system", "Segoe UI", "Roboto", "Arial", "sans-serif"],
        mono: ["JetBrains Mono", "Menlo", "Monaco", "Consolas", "monospace"],
      },
      borderRadius: {
        card: "16px",
        "card-lg": "22px",
      },
      backgroundImage: {
        "page-gradient":
          "radial-gradient(1200px 600px at 70% -10%, rgba(124,58,237,0.18), transparent 60%), radial-gradient(900px 500px at -10% 20%, rgba(0,230,255,0.16), transparent 60%)",
        "logo-gradient":
          "radial-gradient(100% 100% at 30% 30%, #00e6ff 0%, #7c3aed 60%, #0b0f17 100%)",
        "icon-gradient":
          "radial-gradient(100% 100% at 30% 30%, rgba(0,230,255,0.25), rgba(124,58,237,0.25))",
        "band-gradient":
          "linear-gradient(90deg, rgba(0,230,255,0.12), rgba(124,58,237,0.12))",
        "primary-gradient":
          "linear-gradient(135deg, rgba(0,230,255,0.15), rgba(124,58,237,0.15))",
        "btn-gradient":
          "linear-gradient(135deg, #00e6ff 0%, #7c3aed 100%)",
      },
      animation: {
        "grid-move": "grid-move 22s linear infinite",
        "fade-up": "fade-up 0.5s ease forwards",
      },
      keyframes: {
        "grid-move": {
          from: { transform: "translate3d(0,0,0)" },
          to: { transform: "translate3d(52px,52px,0)" },
        },
        "fade-up": {
          from: { opacity: "0", transform: "translateY(16px)" },
          to: { opacity: "1", transform: "translateY(0)" },
        },
      },
    },
  },
  plugins: [],
};

export default config;
