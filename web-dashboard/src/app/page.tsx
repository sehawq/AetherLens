"use client";

import { useSignalR } from "@/lib/signalr";
import ProcessTracker from "@/components/ProcessTracker";
import { useAetherStore } from "@/store/aether-store";
import AnomalyFeed from "@/components/AnomalyFeed";
import LatencyMonitor from "@/components/LatencyMonitor";
import UtcClock from "@/components/UtcClock";
import LiveConsole from "@/components/LiveConsole";
import SystemHealth from "@/components/SystemHealth";
import NetworkTrafficChart from "@/components/NetworkTrafficChart";
import ProtocolPieChart from "@/components/ProtocolPieChart";
import { Activity, Shield, Wifi } from "lucide-react";

export default function WarRoom() {
  useSignalR();

  const connected = useAetherStore((s) => s.connected);

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-[#0a0a0a] text-white selection:bg-[#0096ff]/30 flex-col font-sans">
      {/* Executive Compact Header */}
      <header className="flex h-10 shrink-0 items-center justify-between border-b border-white/5 bg-[#050505]/80 px-4 backdrop-blur-md">
        <div className="flex items-center gap-3">
          <Activity className="h-4 w-4 text-[#0096ff]" />
          <h1 className="text-xs font-bold tracking-[0.2em] font-mono text-white/90 uppercase" style={{ textShadow: "0 0 10px rgba(0, 150, 255, 0.3)" }}>
            AETHERLENS <span className="text-white/30 text-[10px]">v1.0 LAB</span>
          </h1>
        </div>
        
        <div className="flex items-center justify-center gap-4">
          <UtcClock />
          <div className="h-3 w-[1px] bg-white/10" />
          <LatencyMonitor compact />
        </div>

        <div className="flex items-center gap-4">
          <div className="flex items-center gap-2">
            <div
              className={`h-1.5 w-1.5 rounded-full ${
                connected ? "bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.6)]" : "bg-red-500 shadow-[0_0_8px_rgba(239,68,68,0.6)]"
              }`}
            />
            <span className="text-[10px] font-mono text-white/50 tracking-wider">
              {connected ? "SYSTEM ONLINE" : "OFFLINE"}
            </span>
          </div>
        </div>
      </header>

      {/* Main Grid */}
      <main className="grid flex-1 grid-cols-12 gap-1 p-1 bg-[#0a0a0a] min-h-0 overflow-hidden">
        
        {/* Left Column: Process Intelligence */}
        <div className="col-span-3 flex flex-col gap-1 overflow-hidden h-full">
          <div className="flex items-center justify-between px-2 py-1 bg-[#050505] border border-white/5">
             <span className="text-[10px] uppercase text-white/40 font-mono flex items-center gap-2">
                <Wifi className="h-3 w-3" /> Process Monitoring
             </span>
          </div>
          <div className="h-full rounded-sm border border-white/5 bg-[#050505] overflow-hidden">
            <ProcessTracker />
          </div>
        </div>

        {/* Center Column: Live Ops */}
        <div className="col-span-6 flex flex-col gap-1 overflow-hidden h-full">
           {/* Top Row: Charts */}
           <div className="h-32 grid grid-cols-2 gap-1">
              <div className="rounded-sm border border-white/5 bg-[#050505] overflow-hidden">
                <NetworkTrafficChart />
              </div>
              <div className="rounded-sm border border-white/5 bg-[#050505] overflow-hidden">
                <ProtocolPieChart />
              </div>
           </div>

           {/* Middle Row: Console */}
          <div className="flex-1 rounded-sm border border-white/5 bg-[#050505] overflow-hidden flex flex-col min-h-0">
            <LiveConsole />
          </div>

          {/* Bottom Row: Health */}
          <div className="h-48 rounded-sm border border-white/5 bg-[#050505] overflow-hidden shrink-0">
            <SystemHealth />
          </div>
        </div>

        {/* Right Column: Threat Intel */}
        <div className="col-span-3 flex flex-col gap-1 overflow-hidden h-full">
           <div className="flex items-center justify-between px-2 py-1 bg-[#050505] border border-white/5">
             <span className="text-[10px] uppercase text-white/40 font-mono flex items-center gap-2">
                <Shield className="h-3 w-3" /> Threat Feed
             </span>
          </div>
          <div className="flex-1 min-h-0 rounded-sm border border-white/5 bg-[#050505] overflow-hidden">
            <AnomalyFeed />
          </div>
        </div>
      </main>
    </div>
  );
}
