"use client";

import { useEffect, useRef, useCallback } from "react";
import { HubConnectionBuilder, HubConnectionState, LogLevel, type HubConnection } from "@microsoft/signalr";
import { MessagePackHubProtocol } from "@microsoft/signalr-protocol-msgpack";
import { useAetherStore } from "@/store/aether-store";
import { getHubUrl } from "@/lib/runtime-endpoints";
import type { PacketUpdate, ProcessBandwidthUpdate, SuspiciousOutboundConnection, SystemHealthData } from "@/types";

const HUB_URL = getHubUrl();
const RECONNECT_DELAYS = [0, 1000, 2000, 5000, 10000, 15000, 30000];

let _connection: HubConnection | null = null;

function camelizeKey(key: string): string {
  return key.charAt(0).toLowerCase() + key.slice(1);
}

function camelize<T>(obj: unknown): T {
  if (Array.isArray(obj)) return obj.map((item) => camelize(item)) as T;
  if (obj !== null && typeof obj === "object") {
    const result: Record<string, unknown> = {};
    for (const [k, v] of Object.entries(obj as Record<string, unknown>)) {
      result[camelizeKey(k)] = camelize(v);
    }
    return result as T;
  }
  return obj as T;
}

export async function stopSignalRConnection() {
  if (!_connection) return;
  const connection = _connection;
  _connection = null;
  if (connection.state === HubConnectionState.Disconnected) return;
  try {
    await connection.stop();
  } catch {}
}

export function useSignalR() {
  const connectionRef = useRef<HubConnection | null>(null);

  const buildConnection = useCallback(() => {
    return new HubConnectionBuilder()
      .withUrl(HUB_URL)
      .withHubProtocol(new MessagePackHubProtocol())
      .withAutomaticReconnect(RECONNECT_DELAYS)
      .configureLogging(LogLevel.None)
      .build();
  }, []);

  useEffect(() => {
    // Prevent starting a new connection if one is already active or connecting
    if (_connection && _connection.state !== HubConnectionState.Disconnected) {
      return;
    }

    const connection = buildConnection();
    connectionRef.current = connection;
    _connection = connection;

    connection.on("ReceivePackets", (raw: unknown) => {
      useAetherStore.getState().addPackets(camelize<PacketUpdate[]>(raw));
    });

    connection.on("ReceiveProcessStats", (raw: unknown) => {
      useAetherStore.getState().upsertProcess(camelize<ProcessBandwidthUpdate>(raw));
    });

    connection.on("ReceiveHealth", (raw: unknown) => {
      useAetherStore.getState().setHealth(camelize<SystemHealthData>(raw));
    });

    connection.on("ReceiveAnomaly", (raw: unknown) => {
      useAetherStore.getState().addAnomaly(camelize<SuspiciousOutboundConnection>(raw));
    });

    connection.onreconnecting(() => {
      useAetherStore.getState().setConnected(false);
    });

    connection.onreconnected(async () => {
      useAetherStore.getState().setConnected(true);
    });

    connection.onclose(() => {
      useAetherStore.getState().setConnected(false);
    });

    async function start() {
      if (connection.state === HubConnectionState.Disconnected) {
        try {
          await connection.start();
          useAetherStore.getState().setConnected(true);
        } catch (err: unknown) {
          const msg = err instanceof Error ? err.message : String(err);
          if (!msg.includes("stopped during negotiation")) {
            console.error("[signalr] initial connection failed:", err);
          }
        }
      }
    }

    start();

    return () => {
      const localConnection = connectionRef.current;
      connectionRef.current = null;
      if (!localConnection) return;
      if (_connection === localConnection) {
        void stopSignalRConnection();
        return;
      }
      if (localConnection.state !== HubConnectionState.Disconnected) {
        void localConnection.stop();
      }
    };
  }, [buildConnection]);
}
