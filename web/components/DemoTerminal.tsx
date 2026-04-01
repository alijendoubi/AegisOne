"use client";

import { useRef, useState } from "react";

const SIM_LINES = [
  "[14:03:12] observe  ▶ burst writes in /Users/Alex/Documents",
  "[14:03:12] analyze  ▶ extension/rename pattern matches ransomware",
  "[14:03:13] act      ▶ kill pid=4832 (docuprint.exe)",
  "[14:03:13] isolate  ▶ outbound blocked for process: docuprint.exe",
  "[14:03:14] recover  ▶ restored 2,412 files from snapshot @13:55",
  "[14:03:14] explain  ▶ human-readable report created",
];

const IDLE_TEXT =
  '// click "Run Simulation" to see a short containment & rollback story.';

export default function DemoTerminal() {
  const [output, setOutput] = useState(IDLE_TEXT);
  const [running, setRunning] = useState(false);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  function runDemo() {
    if (intervalRef.current) clearInterval(intervalRef.current);
    setRunning(true);
    setOutput("");
    let i = 0;
    intervalRef.current = setInterval(() => {
      if (i < SIM_LINES.length) {
        setOutput((prev) => prev + SIM_LINES[i] + "\n");
        i++;
      } else {
        clearInterval(intervalRef.current!);
        setRunning(false);
      }
    }, 500);
  }

  function clearDemo() {
    if (intervalRef.current) clearInterval(intervalRef.current);
    setRunning(false);
    setOutput(IDLE_TEXT);
  }

  return (
    <>
      <div
        className="rounded-[22px] p-[18px] font-mono min-h-[160px] relative overflow-hidden mb-3"
        style={{
          background:
            "linear-gradient(180deg, rgba(255,255,255,0.06), rgba(255,255,255,0.02))",
          border: "1px solid var(--border)",
          boxShadow: "var(--shadow)",
        }}
        aria-label="Simulated incident terminal"
        aria-live="polite"
      >
        <div className="absolute top-2.5 left-2.5 flex gap-2" aria-hidden="true">
          <span className="w-2.5 h-2.5 rounded-full bg-[#ff5f57] opacity-80" />
          <span className="w-2.5 h-2.5 rounded-full bg-[#febc2e] opacity-80" />
          <span className="w-2.5 h-2.5 rounded-full bg-[#28c840] opacity-80" />
        </div>
        <pre className="mt-5 text-sm leading-relaxed whitespace-pre-wrap m-0 text-[#e6eef7]">
          {output}
        </pre>
      </div>

      <div className="flex gap-2.5">
        <button
          onClick={runDemo}
          disabled={running}
          className="focus-ring inline-flex items-center px-4 py-2.5 rounded-xl text-sm font-bold text-[#e6eef7] transition-transform hover:-translate-y-px disabled:opacity-50 disabled:cursor-not-allowed"
          style={{
            background:
              "linear-gradient(135deg, rgba(0,230,255,0.15), rgba(124,58,237,0.15))",
            border: "1px solid rgba(0,230,255,0.35)",
          }}
        >
          {running ? "Running…" : "Run Simulation"}
        </button>
        <button
          onClick={clearDemo}
          className="focus-ring inline-flex items-center px-4 py-2.5 rounded-xl text-sm font-bold text-[#e6eef7] transition-transform hover:-translate-y-px"
          style={{
            background: "var(--panel)",
            border: "1px solid var(--border)",
          }}
        >
          Reset
        </button>
      </div>
    </>
  );
}
