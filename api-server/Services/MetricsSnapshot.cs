using System.Diagnostics;

namespace AetherLens.Services;

public sealed class MetricsSnapshot
{
    private long _packetCount;
    private long _latencySumTicks;
    private long _maxLatencyTicks;
    private long _windowStart = Stopwatch.GetTimestamp();
    private long _windowPacketCount;
    private int _grpcConnected;

    public bool GrpcConnected => Volatile.Read(ref _grpcConnected) == 1;
    public long TotalPackets => Volatile.Read(ref _packetCount);
    public DateTimeOffset StartTime { get; } = DateTimeOffset.UtcNow;

    public void SetGrpcConnected(bool connected) =>
        Interlocked.Exchange(ref _grpcConnected, connected ? 1 : 0);

    public void RecordPacket(long latencyTicks)
    {
        Interlocked.Increment(ref _packetCount);
        Interlocked.Increment(ref _windowPacketCount);
        Interlocked.Add(ref _latencySumTicks, latencyTicks);

        long current;
        while (latencyTicks > (current = Volatile.Read(ref _maxLatencyTicks)))
        {
            if (Interlocked.CompareExchange(ref _maxLatencyTicks, latencyTicks, current) == current)
                break;
        }
    }

    public (double PacketsPerSecond, double AvgLatencyMs, double MaxLatencyMs) Snapshot()
    {
        var now = Stopwatch.GetTimestamp();
        var windowStart = Interlocked.Exchange(ref _windowStart, now);
        var count = Interlocked.Exchange(ref _windowPacketCount, 0);
        var latencySum = Interlocked.Exchange(ref _latencySumTicks, 0);
        var maxTicks = Interlocked.Exchange(ref _maxLatencyTicks, 0);

        var elapsed = Stopwatch.GetElapsedTime(windowStart, now);
        var tps = elapsed.TotalSeconds > 0 ? count / elapsed.TotalSeconds : 0;
        var avgMs = count > 0 ? Stopwatch.GetElapsedTime(0, latencySum / count).TotalMilliseconds : 0;
        var maxMs = Stopwatch.GetElapsedTime(0, maxTicks).TotalMilliseconds;

        return (tps, avgMs, maxMs);
    }
}
