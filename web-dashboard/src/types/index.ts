export interface PacketUpdate {
  process: string;
  localAddress: string;
  remoteAddress: string;
  protocol: string;
  bytes: number;
  outbound: boolean;
  timestamp: number;
  latencyMs: number;
}

export interface ProcessBandwidthUpdate {
  process: string;
  bytesPerSecond: number;
  packetsPerSecond: number;
  protocol: string;
  timestamp: number;
  tier: string;
}

export interface SuspiciousOutboundConnection {
  process: string;
  remoteAddress: string;
  reason: string;
  severity: string;
  score: number;
  timestamp: number;
}

export interface SystemHealthData {
  grpcConnected: boolean;
  packetsPerSecond: number;
  avgLatencyMs: number;
  maxLatencyMs: number;
  uptime: string;
}
