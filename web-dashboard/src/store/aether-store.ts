import { create } from "zustand";
import { subscribeWithSelector } from "zustand/middleware";
import type { PacketUpdate, ProcessBandwidthUpdate, SuspiciousOutboundConnection, SystemHealthData } from "@/types";

const MAX_PACKETS = 500;
const MAX_PROCESSES = 100;
const MAX_ANOMALIES = 50;

interface AetherState {
  packets: PacketUpdate[];
  processStats: ProcessBandwidthUpdate[];
  anomalies: SuspiciousOutboundConnection[];
  health: SystemHealthData | null;
  connected: boolean;

  addPackets: (packets: PacketUpdate[]) => void;
  upsertProcess: (process: ProcessBandwidthUpdate) => void;
  addAnomaly: (a: SuspiciousOutboundConnection) => void;
  setHealth: (h: SystemHealthData) => void;
  setConnected: (c: boolean) => void;
}

export const useAetherStore = create<AetherState>()(
  subscribeWithSelector((set) => ({
    packets: [],
    processStats: [],
    anomalies: [],
    health: null,
    connected: false,

    addPackets: (packets) =>
      set((state) => {
        const next = [...packets, ...state.packets];
        return { packets: next.slice(0, MAX_PACKETS) };
      }),

    upsertProcess: (process) =>
      set((state) => ({
        processStats: [process, ...state.processStats.filter((p) => p.process !== process.process)].slice(0, MAX_PROCESSES),
      })),

    addAnomaly: (a) =>
      set((state) => ({
        anomalies: [a, ...state.anomalies].slice(0, MAX_ANOMALIES),
      })),

    setHealth: (health) => set({ health }),
    setConnected: (connected) => set({ connected }),
  }))
);
