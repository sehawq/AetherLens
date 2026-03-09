# AetherLens Architecture

AetherLens is a high-performance, real-time network traffic analysis platform designed to visualize network flows and detect security anomalies.

## Overview

The system operates as a distributed microservices architecture, leveraging the strengths of three different ecosystems:

1.  **Rust (Core Engine):** Low-level packet capture, parsing, and analysis.
2.  **.NET 8 (Backend Service):** High-throughput data aggregation, state management, and WebSocket broadcasting.
3.  **Next.js 14 (Frontend):** Reactive, high-frequency data visualization.

```mermaid
graph TD
    subgraph "Core Layer (Rust)"
        Sniffer[Packet Sniffer (pnet)]
        Engine[Analysis Engine]
        gRPC_Server[gRPC Server (Tonic)]
        
        Sniffer -->|Raw Packets| Engine
        Engine -->|Anomalies & Stats| gRPC_Server
    end

    subgraph "Service Layer (.NET 8)"
        gRPC_Client[gRPC Client]
        Processor[Event Processor]
        SignalR[SignalR Hub]
        
        gRPC_Server -->|Stream (aether.proto)| gRPC_Client
        gRPC_Client --> Processor
        Processor -->|Real-time Events| SignalR
    end

    subgraph "Presentation Layer (Next.js)"
        Dashboard[Live Dashboard]
        Charts[Traffic Visualization]
        
        SignalR -->|WebSockets (MessagePack)| Dashboard
        Dashboard --> Charts
    end
```

## 1. Core Engine (Rust)

The core engine (`aether_core`) is responsible for interfacing with the network adapter using `libpnet` (Windows Npcap).

### Key Components:
- **Packet Sniffer:** Captures raw Ethernet frames in a zero-copy manner where possible.
- **Protocol Analyzer:** Dissects headers (Ethernet, IPv4/6, TCP/UDP) to extract metadata like source/dest IP, ports, and protocol type.
- **Anomaly Detection:**
  - **Port Scans:** Tracks connection attempts to multiple ports on a single host within a short window.
  - **Suspicious Outbound:** Flags connections to known bad ports or unusual protocols.
- **gRPC Server:** Streams structured `PacketMessage` and `SuspiciousConnectionMessage` events to the backend.

**Technology Stack:** `Tokio`, `Tonic`, `Prost`, `Pnet`, `DashMap`.

## 2. Backend Service (.NET 8)

The backend (`AetherLens.Api`) acts as the central nervous system. It aggregates high-frequency streams from the Rust core and broadcasts summarized updates to connected clients.

### Key Components:
- **gRPC Client:** Consumes the stream from the Rust core.
- **Metrics Snapshotting:** Uses `Interlocked` operations to maintain thread-safe counters for packet rates and latency without locking.
- **SignalR Hub:** Broadcasts updates to web clients.
  - Uses **MessagePack** protocol for binary serialization, significantly reducing payload size compared to JSON.
- **Persistence (Optional):** Can log critical events to a local SQLite database (`AetherDb`).

**Technology Stack:** `ASP.NET Core 8`, `SignalR`, `Grpc.Net.Client`, `Dapper`, `SQLite`.

## 3. Web Dashboard (Next.js)

The dashboard (`web-dashboard`) provides a "War Room" style interface for monitoring network activity.

### Key Components:
- **Zustand Store:** Manages application state (connected status, packet lists, anomalies) efficiently.
- **SignalR Client:** Connects to the backend hub and handles incoming binary streams.
- **Recharts & Framer Motion:** Renders smooth, animated charts and lists that update at 60fps.
- **Tailwind CSS:** Provides a responsive, dark-mode-first UI.

**Technology Stack:** `Next.js 14`, `React 19`, `TypeScript`, `Tailwind CSS`, `Recharts`.

## Data Flow

1.  **Capture:** Rust core captures a packet.
2.  **Analysis:** Metadata is extracted and checked against rules.
3.  **Stream:** Metadata is sent via gRPC stream to .NET backend.
4.  **Aggregation:** Backend aggregates metrics (e.g., packets/sec) over 1-second windows.
5.  **Broadcast:** Backend pushes updates to frontend via SignalR (WebSockets).
6.  **Render:** Frontend updates the UI components.

## Building from Source

See [CONTRIBUTING.md](CONTRIBUTING.md) for build instructions.
