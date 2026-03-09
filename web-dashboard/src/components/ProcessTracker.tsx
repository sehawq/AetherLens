"use client";

import { Activity, ArrowUpRight } from "lucide-react";
import { useAetherStore } from "@/store/aether-store";

function formatBytes(bytes: number) {
  if (bytes >= 1_000_000_000) return `${(bytes / 1_000_000_000).toFixed(2)} GB/s`;
  if (bytes >= 1_000_000) return `${(bytes / 1_000_000).toFixed(2)} MB/s`;
  if (bytes >= 1_000) return `${(bytes / 1_000).toFixed(2)} KB/s`;
  return `${bytes} B/s`;
}

export default function ProcessTracker() {
  const processStats = useAetherStore((s) => s.processStats);

  return (
    <div className="card flex flex-col gap-3 h-full">
      <div className="flex items-center gap-2 shrink-0">
        <Activity className="h-4 w-4 text-[var(--accent-blue)]" />
        <span className="text-sm font-semibold tracking-wide">Process Tracker</span>
      </div>

      <div className="flex-1 overflow-y-auto pr-1 custom-scrollbar min-h-0 space-y-1.5">
        {processStats.length === 0 ? (
          <div className="flex h-full items-center justify-center text-sm text-[var(--text-secondary)] opacity-60">
            Waiting for network process activity...
          </div>
        ) : (
          processStats.map((p) => (
            <div key={`${p.process}-${p.timestamp}`} className="rounded-lg border border-white/10 bg-white/5 px-3 py-2">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <ArrowUpRight className="h-3.5 w-3.5 text-[var(--accent-amber)]" />
                  <span className="font-semibold text-[var(--text-primary)]">{p.process}</span>
                  <span className="text-[10px] text-[var(--text-secondary)] uppercase">{p.protocol}</span>
                </div>
                <span className={`text-[10px] px-1.5 py-0.5 rounded ${
                  p.tier === "HIGH"
                    ? "bg-[var(--accent-red)]/15 text-[var(--accent-red)]"
                    : p.tier === "MEDIUM"
                      ? "bg-[var(--accent-amber)]/15 text-[var(--accent-amber)]"
                      : "bg-[var(--accent-green)]/15 text-[var(--accent-green)]"
                }`}>
                  {p.tier}
                </span>
              </div>
              <div className="mt-1.5 flex items-center justify-between text-xs font-mono text-[var(--text-secondary)]">
                <span>{formatBytes(p.bytesPerSecond)}</span>
                <span>{p.packetsPerSecond} pkts/s</span>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
