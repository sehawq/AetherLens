"use client";

import { AnimatePresence, motion } from "framer-motion";
import { AlertTriangle } from "lucide-react";
import { useAetherStore } from "@/store/aether-store";
import { timeAgo } from "@/lib/utils";

const SEVERITY_STYLES: Record<string, { badge: string; border: string; flash: string }> = {
  CRITICAL: {
    badge: "bg-[var(--accent-red)] text-white",
    border: "border border-[var(--accent-red)]/30 bg-[var(--accent-red)]/8",
    flash: "animate-[pulse_1.5s_ease-in-out_infinite]",
  },
  WARNING: {
    badge: "bg-[var(--accent-amber)] text-black",
    border: "border border-[var(--accent-amber)]/20 bg-[var(--bg-card-hover)]",
    flash: "",
  },
};

const TYPE_LABELS: Record<string, string> = {
  "Outbound Telnet attempt": "TELNET",
  "Outbound SMB connection": "SMB",
  "Outbound RDP session": "RDP",
  "Outbound IRC channel": "IRC",
};

const MAX_VISIBLE = 15;

export default function AnomalyFeed() {
  const anomalies = useAetherStore((s) => s.anomalies);
  const visible = anomalies.slice(0, MAX_VISIBLE);

  return (
    <div className="card flex flex-col gap-3 overflow-hidden h-full">
      <div className="flex items-center gap-2 shrink-0">
        <AlertTriangle className="h-4 w-4 text-[var(--accent-amber)]" />
        <span className="text-sm font-semibold tracking-wide">Suspicious Outbound Connections</span>
        {anomalies.length > 0 && (
          <span className="ml-auto rounded bg-[var(--bg-card-hover)] px-1.5 py-0.5 text-[10px] font-mono text-[var(--text-secondary)]">
            {anomalies.length}
          </span>
        )}
      </div>

      <div className="flex-1 overflow-y-auto custom-scrollbar flex flex-col gap-1.5 min-h-0">
        <AnimatePresence initial={false}>
          {visible.length === 0 ? (
            <div className="flex h-20 items-center justify-center text-sm text-[var(--text-secondary)]">
              No anomalies detected
            </div>
          ) : (
            visible.map((a, i) => {
              const sev = SEVERITY_STYLES[a.severity] ?? SEVERITY_STYLES.WARNING;
              const key = `${a.timestamp}-${a.reason}-${i}`;

              return (
                <motion.div
                  key={key}
                  initial={{ opacity: 0, x: -20, height: 0 }}
                  animate={{ opacity: 1, x: 0, height: "auto" }}
                  exit={{ opacity: 0, height: 0 }}
                  transition={{ duration: 0.3, ease: "easeOut" }}
                  className={`flex items-center justify-between rounded-lg px-3 py-2 text-xs ${sev.border} ${sev.flash}`}
                >
                  <div className="flex items-center gap-2">
                    <span className={`rounded px-1.5 py-0.5 text-[10px] font-bold ${sev.badge}`}>
                      {a.severity === "CRITICAL" ? "CRIT" : "WARN"}
                    </span>
                    <span className="font-semibold text-[var(--text-primary)]">{a.process}</span>
                    <span className="text-[var(--text-secondary)]">
                      {TYPE_LABELS[a.reason] ?? a.reason}
                    </span>
                  </div>
                  <div className="flex items-center gap-3">
                    <span className="font-mono text-[var(--text-primary)]">
                      {a.remoteAddress}
                    </span>
                    <span className="font-mono text-[10px] text-[var(--accent-blue)]">
                      score:{a.score.toFixed(2)}
                    </span>
                    <span className="text-[var(--text-secondary)]">{timeAgo(a.timestamp)}</span>
                  </div>
                </motion.div>
              );
            })
          )}
        </AnimatePresence>
      </div>
    </div>
  );
}
