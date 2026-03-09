"use client";

import { useEffect, useState } from "react";
import { Cell, Pie, PieChart, ResponsiveContainer, Tooltip } from "recharts";
import { useAetherStore } from "@/store/aether-store";

interface ProtocolData {
  name: string;
  value: number;
  color: string;
}

const COLORS: Record<string, string> = {
  TCP: "#0096ff",
  UDP: "#22c55e",
  ICMP: "#eab308",
  OTHER: "#ef4444",
};

export default function ProtocolPieChart() {
  const [data, setData] = useState<ProtocolData[]>([]);

  useEffect(() => {
    const interval = setInterval(() => {
      const packets = useAetherStore.getState().packets;
      
      const counts: Record<string, number> = { TCP: 0, UDP: 0, ICMP: 0, OTHER: 0 };
      
      packets.forEach(p => {
        const proto = p.protocol?.toUpperCase() || "OTHER";
        if (counts[proto] !== undefined) {
          counts[proto]++;
        } else {
          counts["OTHER"]++;
        }
      });

      const chartData = Object.entries(counts)
        .filter(([, value]) => value > 0)
        .map(([name, value]) => ({
          name,
          value,
          color: COLORS[name] || COLORS.OTHER,
        }));

      setData(chartData);
    }, 1000);

    return () => clearInterval(interval);
  }, []);

  return (
    <div className="flex h-full flex-col overflow-hidden bg-[#0D1117] relative">
      <div className="flex items-center justify-between border-b border-[#30363D] px-3 py-2">
        <div className="flex items-center gap-2">
          <div className="h-2 w-2 rounded-full bg-purple-500" />
          <span className="text-xs font-semibold text-[var(--text-secondary)] uppercase tracking-wider">
            Protocol Dist
          </span>
        </div>
      </div>

      <div className="flex-1 min-h-0 w-full relative">
        <ResponsiveContainer width="100%" height="100%">
          <PieChart>
            <Pie
              data={data}
              cx="50%"
              cy="50%"
              innerRadius={25}
              outerRadius={40}
              paddingAngle={5}
              dataKey="value"
              stroke="none"
            >
              {data.map((entry, index) => (
                <Cell key={`cell-${index}`} fill={entry.color} />
              ))}
            </Pie>
            <Tooltip 
              contentStyle={{ backgroundColor: '#0D1117', borderColor: '#30363D', fontSize: '12px' }}
              itemStyle={{ color: '#fff' }}
            />
          </PieChart>
        </ResponsiveContainer>
        
        {/* Center Text */}
        <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
          <span className="text-[10px] text-white/50 font-mono">PROTO</span>
        </div>
      </div>
    </div>
  );
}
