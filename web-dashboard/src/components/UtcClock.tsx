"use client";

import { useEffect, useState } from "react";
import { Clock } from "lucide-react";

export default function UtcClock() {
  const [time, setTime] = useState("");

  useEffect(() => {
    const update = () => {
      const now = new Date();
      setTime(now.toISOString().replace("T", " ").split(".")[0] + " UTC");
    };
    update();
    const i = setInterval(update, 1000);
    return () => clearInterval(i);
  }, []);

  return (
    <div className="flex items-center gap-2 rounded bg-white/5 px-3 py-1 text-[10px] font-mono text-white/70">
      <Clock className="h-3 w-3 opacity-50" />
      <span>{time}</span>
    </div>
  );
}
