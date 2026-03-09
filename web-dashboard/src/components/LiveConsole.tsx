"use client";

import { useEffect, useRef, useState } from "react";
import { useAetherStore } from "@/store/aether-store";
import { Terminal } from "lucide-react";

interface LogEntry {
  id: string;
  timestamp: string;
  level: "INFO" | "WARN" | "ERROR" | "DEBUG";
  message: string;
}

export default function LiveConsole() {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const scrollRef = useRef<HTMLDivElement>(null);
  
  // Ref to track last processed packet to avoid duplicates if subscription fires redundantly
  const lastPacketRef = useRef<number>(0);

  useEffect(() => {
    const unsubscribePackets = useAetherStore.subscribe(
      (state) => state.packets,
      (packets) => {
        if (packets.length === 0) return;
        const latest = packets[0];
        
        // Simple dedup by timestamp
        if (latest.timestamp === lastPacketRef.current) return;
        lastPacketRef.current = latest.timestamp;

        // Fallback for missing fields due to camelCase issues
        // @ts-expect-error fallback for legacy fields
        const local = latest.localAddress || latest.LocalAddress || "Unknown";
        // @ts-expect-error fallback for legacy fields
        const remote = latest.remoteAddress || latest.RemoteAddress || "Unknown";
        
        addLog("INFO", `[PACKET] ${latest.protocol} ${local} -> ${remote} (${latest.bytes}b)`);
      }
    );

    const unsubscribeAnomalies = useAetherStore.subscribe(
      (state) => state.anomalies,
      (anomalies) => {
        if (anomalies.length === 0) return;
        const latest = anomalies[0];
        addLog("WARN", `[ANOMALY] ${latest.reason} detected for ${latest.process}`);
      }
    );

    const unsubscribeHealth = useAetherStore.subscribe(
      (state) => state.health,
      (health) => {
        if (!health) return;
        if (!health.grpcConnected) {
          // Debounce this log?
          // addLog("ERROR", "[SYSTEM] Core Engine disconnected (gRPC stream lost)");
        }
      }
    );

    const unsubscribeConnection = useAetherStore.subscribe(
      (state) => state.connected,
      (connected) => {
        addLog(connected ? "INFO" : "ERROR", `[SIGNALR] Connection ${connected ? "established" : "lost"}`);
      }
    );

    return () => {
      unsubscribePackets();
      unsubscribeAnomalies();
      unsubscribeHealth();
      unsubscribeConnection();
    };
  }, []);

  const addLog = (level: LogEntry["level"], message: string) => {
    setLogs((prev) => {
      const newLog: LogEntry = {
        id: crypto.randomUUID(),
        timestamp: new Date().toLocaleTimeString('en-US', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit', fractionalSecondDigits: 3 }),
        level,
        message,
      };
      const next = [...prev, newLog];
      if (next.length > 50) return next.slice(next.length - 50);
      return next;
    });
  };

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [logs]);

  return (
    <div 
      className="flex h-full flex-col overflow-hidden bg-[#0D1117] border-[#30363D] relative after:content-[''] after:absolute after:inset-0 after:pointer-events-none after:bg-[linear-gradient(transparent_50%,rgba(0,0,0,0.1)_50%)] after:bg-[length:100%_4px]" 
    >
      <div className="flex items-center gap-2 border-b border-[#30363D] pb-2 p-2 relative z-10">
        <Terminal className="h-4 w-4 text-[var(--accent-green)]" />
        <span className="text-xs font-semibold text-[var(--text-secondary)] tracking-widest uppercase">
          Live Engine Console
        </span>
        <span className="ml-auto flex items-center gap-1.5 rounded bg-[var(--accent-green)]/10 px-2 py-0.5 text-[10px] text-[var(--accent-green)] font-mono">
          <span className="relative flex h-2 w-2">
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-[var(--accent-green)] opacity-75"></span>
            <span className="relative inline-flex rounded-full h-2 w-2 bg-[var(--accent-green)]"></span>
          </span>
          ACTIVE
        </span>
      </div>

      <div
        ref={scrollRef}
        className="flex-1 overflow-y-auto space-y-1 text-[10px] p-2 relative z-10"
        style={{ fontFamily: "'JetBrains Mono', monospace" }}
      >
        {logs.length === 0 ? (
          <div className="text-[var(--text-secondary)] animate-pulse p-2">Waiting for packet stream...</div>
        ) : (
          logs.map((log) => (
            <div key={log.id} className="flex gap-2 leading-tight break-all hover:bg-white/5 px-1 rounded">
              <span className="text-[#8B949E] opacity-75 shrink-0">[{log.timestamp}]</span>
              <span
                className={`shrink-0 font-bold w-12 text-center ${
                  log.level === "INFO"
                    ? "text-[#58A6FF]"
                    : log.level === "WARN"
                    ? "text-[#D29922]"
                    : log.level === "ERROR"
                    ? "text-[#F85149]"
                    : "text-[#8B949E]"
                }`}
              >
                {log.level}
              </span>
              <span className="text-[#C9D1D9]">{log.message}</span>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
