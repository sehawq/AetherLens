"use client";

import { useEffect, useState } from "react";
import { Area, AreaChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { useAetherStore } from "@/store/aether-store";

interface DataPoint {
  time: string;
  pps: number;
}

export default function NetworkTrafficChart() {
  const [data, setData] = useState<DataPoint[]>([]);
  
  useEffect(() => {
    // Initial fill
    const initialData = Array.from({ length: 20 }).map(() => ({
      time: "",
      pps: 0,
    }));
    setData(initialData);

    const interval = setInterval(() => {
      const health = useAetherStore.getState().health;
      const pps = health ? health.packetsPerSecond : 0;
      
      const now = new Date();
      const timeStr = now.toLocaleTimeString('en-US', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' });

      setData(prev => {
        const next = [...prev, { time: timeStr, pps }];
        if (next.length > 20) return next.slice(next.length - 20);
        return next;
      });
    }, 1000);

    return () => clearInterval(interval);
  }, []);

  return (
    <div className="flex h-full flex-col overflow-hidden bg-[#0D1117] relative">
      <div className="flex items-center justify-between border-b border-[#30363D] px-3 py-2">
        <div className="flex items-center gap-2">
          <div className="h-2 w-2 rounded-full bg-[var(--accent-blue)]" />
          <span className="text-xs font-semibold text-[var(--text-secondary)] uppercase tracking-wider">
            Network Throughput
          </span>
        </div>
        <span className="text-[10px] font-mono text-[var(--text-secondary)]">
          LIVE PPS
        </span>
      </div>

      <div className="flex-1 min-h-0 w-full p-2">
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart data={data}>
            <defs>
              <linearGradient id="colorPps" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor="var(--accent-blue)" stopOpacity={0.3}/>
                <stop offset="95%" stopColor="var(--accent-blue)" stopOpacity={0}/>
              </linearGradient>
            </defs>
            <XAxis 
              dataKey="time" 
              hide 
              interval="preserveStartEnd"
            />
            <YAxis 
              hide 
              domain={[0, 'auto']} 
            />
            <Tooltip 
              contentStyle={{ backgroundColor: '#0D1117', borderColor: '#30363D', fontSize: '12px' }}
              itemStyle={{ color: 'var(--accent-blue)' }}
              // eslint-disable-next-line @typescript-eslint/no-explicit-any
              formatter={(value: any) => [`${Number(value).toFixed(1)} PPS`, "Throughput"]}
              labelStyle={{ display: 'none' }}
            />
            <Area 
              type="monotone" 
              dataKey="pps" 
              stroke="var(--accent-blue)" 
              strokeWidth={2}
              fillOpacity={1} 
              fill="url(#colorPps)" 
              isAnimationActive={false}
            />
          </AreaChart>
        </ResponsiveContainer>
      </div>
      
      {/* Stat Overlay */}
      <div className="absolute top-10 right-4 flex flex-col items-end">
        <span className="text-2xl font-bold text-white font-mono">
          {data[data.length - 1]?.pps.toFixed(1) || "0.0"}
        </span>
        <span className="text-[10px] text-[var(--text-secondary)]">PACKETS / SEC</span>
      </div>
    </div>
  );
}
