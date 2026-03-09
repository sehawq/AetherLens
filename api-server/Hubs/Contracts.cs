namespace AetherLens.Hubs;

public readonly record struct PacketUpdate(
    string Process,
    string LocalAddress,
    string RemoteAddress,
    string Protocol,
    long Bytes,
    bool Outbound,
    long Timestamp,
    double LatencyMs);

public readonly record struct ProcessBandwidthUpdate(
    string Process,
    long BytesPerSecond,
    long PacketsPerSecond,
    string Protocol,
    long Timestamp,
    string Tier);

public readonly record struct SuspiciousOutboundConnection(
    string Process,
    string RemoteAddress,
    string Reason,
    string Severity,
    double Score,
    long Timestamp);

public readonly record struct SystemHealth(
    bool GrpcConnected,
    double PacketsPerSecond,
    double AvgLatencyMs,
    double MaxLatencyMs,
    string Uptime);
