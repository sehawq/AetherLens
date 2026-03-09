"use client";

import { Activity, Gauge, Clock } from "lucide-react";
import { useAetherStore } from "@/store/aether-store";
import { cn } from "@/lib/utils";

export default function SystemHealth() {
  const health = useAetherStore((s) => s.health);
  const connected = useAetherStore((s) => s.connected);

  const pps = health?.packetsPerSecond ?? 0;
  const avgLatency = health?.avgLatencyMs ?? 0;
  const maxLatency = health?.maxLatencyMs ?? 0;
  const grpcUp = health?.grpcConnected ?? false;
  const uptime = health?.uptime ?? "00:00:00";

  const latencyColor =
    avgLatency <= 100 ? "var(--accent-green)" :
    avgLatency <= 200 ? "var(--accent-amber)" :
    "var(--accent-red)";

  return (
    <div className="flex flex-col gap-2 p-3 h-full justify-between">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Activity className="h-4 w-4 text-[#0096ff]" />
          <span className="text-xs font-bold tracking-widest uppercase text-white/90">System Health</span>
        </div>
        
        <div className="flex items-center gap-3">
           <div className="flex items-center gap-1.5">
            <div className={cn("h-1.5 w-1.5 rounded-full", connected ? "bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.6)]" : "bg-red-500")} />
            <span className="text-[10px] font-mono text-white/50">SignalR</span>
          </div>
          <div className="flex items-center gap-1.5">
             <div className={cn("h-1.5 w-1.5 rounded-full", grpcUp ? "bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.6)]" : "bg-red-500")} />
            <span className="text-[10px] font-mono text-white/50">gRPC</span>
          </div>
        </div>
      </div>

      {/* Metrics grid */}
      <div className="grid grid-cols-2 gap-2 flex-1">
        <MetricCard
          icon={<Gauge className="h-3 w-3" />}
          label="THROUGHPUT"
          value={`${pps.toFixed(1)} PPS`}
          color="#0096ff"
        />
        <MetricCard
          icon={<Activity className="h-3 w-3" />}
          label="AVG LATENCY"
          value={`${avgLatency.toFixed(1)}ms`}
          color={latencyColor}
        />
        <MetricCard
          icon={<Activity className="h-3 w-3" />}
          label="MAX LATENCY"
          value={`${maxLatency.toFixed(1)}ms`}
          color="rgba(255,255,255,0.5)"
        />
        <MetricCard
          icon={<Clock className="h-3 w-3" />}
          label="UPTIME"
          value={uptime}
          color="#a855f7"
        />
      </div>

      {/* Latency bar */}
      <div className="flex flex-col gap-1 mt-1">
        <div className="flex justify-between text-[9px] text-white/30 font-mono">
          <span>LATENCY TARGET: 100ms</span>
          <span style={{ color: latencyColor }}>{avgLatency.toFixed(1)}ms</span>
        </div>
        <div className="h-1 w-full overflow-hidden rounded-full bg-white/5">
          <div
            className="h-full rounded-full transition-all duration-500 shadow-[0_0_10px_currentColor]"
            style={{
              width: `${Math.min((avgLatency / 200) * 100, 100)}%`,
              background: latencyColor,
            }}
          />
        </div>
      </div>
    </div>
  );
}

function MetricCard({
  icon,
  label,
  value,
  color,
}: {
  icon: React.ReactNode;
  label: string;
  value: string;
  color: string;
}) {
  return (
    <div className="flex flex-col justify-center gap-0.5 rounded border border-white/5 bg-white/[0.02] px-3 py-1.5">
      <div className="flex items-center gap-1.5 text-white/40">
        {icon}
        <span className="text-[9px] font-bold tracking-wider">{label}</span>
      </div>
      <span className="font-mono text-xs font-bold tracking-tight" style={{ color, textShadow: `0 0 10px ${color}40` }}>
        {value}
      </span>
    </div>
  );
}
