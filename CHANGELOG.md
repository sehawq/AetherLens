# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-03-09

### Added
- **Core Engine (Rust):**
  - High-performance packet capture using `libpnet`.
  - Anomaly detection for port scans and suspicious outbound connections.
  - gRPC streaming server for real-time event broadcasting.
- **Backend (.NET 8):**
  - Centralized API for aggregating metrics.
  - SignalR hub with MessagePack protocol support.
  - SQLite persistence for long-term data storage.
- **Dashboard (Next.js 14):**
  - Real-time "War Room" visualization.
  - Live charts for throughput, latency, and protocol distribution.
  - Process-level bandwidth monitoring.
- **Infrastructure:**
  - Docker Compose support for demo environments.
  - GitHub Actions CI pipeline for Rust, .NET, and Node.js.

### Changed
- Rebranded project from "Vistoris Guardian" to "AetherLens".
- Updated all namespaces and package names to reflect the new identity.
- Optimized Npcap SDK integration for Windows builds.
