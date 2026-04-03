"use client";

import { useEffect, useRef, useState } from "react";

const LINES = [
  { text: "event: rapid file writes detected in user Documents/", color: "text-[#e6eef7]" },
  { text: "signal: entropy spike and canary touch → score 0.97", color: "text-[#f59e0b]" },
  { text: "action: terminating process tree pid=4832 (docuprint.exe) ✓", color: "text-[#10b981]" },
  { text: "network: blocking outbound C2 for docuprint.exe ✓", color: "text-[#10b981]" },
  { text: "recovery: restoring 2,412 files from snapshot @13:55 ✓", color: "text-[#10b981]" },
  { text: "explain: incident report generated (timeline + checklist) ✓", color: "text-[#10b981]" },
];

export default function TerminalHero() {
  const [visibleLines, setVisibleLines] = useState<number>(0);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    let idx = 0;
    function tick() {
      if (idx < LINES.length) {
        idx++;
        setVisibleLines(idx);
        timerRef.current = setTimeout(tick, 700);
      }
    }
    timerRef.current = setTimeout(tick, 800);
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, []);

  return (
    <div
      className="rounded-[22px] p-[18px] font-mono min-h-[220px] relative overflow-hidden"
      style={{
        background:
          "linear-gradient(180deg, rgba(255,255,255,0.06), rgba(255,255,255,0.02))",
        border: "1px solid var(--border)",
        boxShadow: "var(--shadow)",
      }}
      aria-label="Live security terminal"
    >
      {/* Traffic light dots */}
      <div className="absolute top-2.5 left-2.5 flex gap-2" aria-hidden="true">
        <span className="w-2.5 h-2.5 rounded-full bg-[#ff5f57] opacity-80" />
        <span className="w-2.5 h-2.5 rounded-full bg-[#febc2e] opacity-80" />
        <span className="w-2.5 h-2.5 rounded-full bg-[#28c840] opacity-80" />
      </div>

      <pre className="mt-5 text-sm leading-relaxed whitespace-pre-wrap m-0">
        <span className="text-[#7dd3fc]">aegisone-agent</span>
        {" v1.0.0  •  on-device AI active\n"}
        {"watching: "}
        <span className="text-[#7dd3fc]">filesystem, processes, network</span>
        {"\npolicy: "}
        <span className="text-[#7dd3fc]">defensive-only</span>
        {"  |  privacy: "}
        <span className="text-[#7dd3fc]">on-device</span>
        {"\n\n› waiting for events…\n"}
        {LINES.slice(0, visibleLines).map((line, i) => (
          <span key={i} className={line.color}>
            {line.text + "\n"}
          </span>
        ))}
      </pre>
    </div>
  );
}
