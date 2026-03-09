using System.Collections.Concurrent;
using System.Diagnostics;
using System.Threading.Channels;
using AetherLens.Hubs;
using AetherLens.Protos;
using AetherLens.Services;
using Microsoft.AspNetCore.SignalR;

namespace AetherLens.Persistence;

public sealed class PacketConsumerService : BackgroundService
{
    private readonly ChannelReader<PacketMessage> _reader;
    private readonly ChannelWriter<PacketUpdate> _persistenceWriter;
    private readonly IHubContext<NetworkMonitorHub> _hub;
    private readonly MetricsSnapshot _metrics;

    public PacketConsumerService(
        ChannelReader<PacketMessage> reader,
        ChannelWriter<PacketUpdate> persistenceWriter,
        IHubContext<NetworkMonitorHub> hub,
        MetricsSnapshot metrics)
    {
        _reader = reader;
        _persistenceWriter = persistenceWriter;
        _hub = hub;
        _metrics = metrics;
    }

    protected override async Task ExecuteAsync(CancellationToken stoppingToken)
    {
        var packetQueue = new ConcurrentQueue<PacketUpdate>();
        var processCounters = new ConcurrentDictionary<string, (long Bytes, long Packets, string Protocol)>();
        var packetTask = RunPacketLoop(packetQueue, processCounters, stoppingToken);
        var processTask = RunProcessLoop(processCounters, stoppingToken);
        await Task.WhenAll(packetTask, processTask);
    }

    private async Task RunPacketLoop(
        ConcurrentQueue<PacketUpdate> queue,
        ConcurrentDictionary<string, (long Bytes, long Packets, string Protocol)> counters,
        CancellationToken ct)
    {
        while (await _reader.WaitToReadAsync(ct))
        {
            while (_reader.TryRead(out var packet))
            {
                var now = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds();
                var latencyMs = Math.Max(0, now - (long)packet.Timestamp);
                var latencyTicks = latencyMs * Stopwatch.Frequency / 1000;
                _metrics.RecordPacket(latencyTicks);

                var update = new PacketUpdate(
                    packet.Process,
                    packet.LocalAddr,
                    packet.RemoteAddr,
                    packet.Protocol,
                    (long)packet.Bytes,
                    packet.Outbound,
                    (long)packet.Timestamp,
                    latencyMs);

                _persistenceWriter.TryWrite(update);
                queue.Enqueue(update);

                counters.AddOrUpdate(
                    packet.Process,
                    _ => ((long)packet.Bytes, 1, packet.Protocol),
                    (_, current) => (current.Bytes + (long)packet.Bytes, current.Packets + 1, packet.Protocol));
            }

            if (queue.IsEmpty)
                continue;

            var batch = new List<PacketUpdate>(64);
            while (queue.TryDequeue(out var item))
                batch.Add(item);

            if (batch.Count > 0)
                await _hub.Clients.Group("network").SendAsync("ReceivePackets", batch.ToArray(), ct);
        }
    }

    private async Task RunProcessLoop(
        ConcurrentDictionary<string, (long Bytes, long Packets, string Protocol)> counters,
        CancellationToken ct)
    {
        using var timer = new PeriodicTimer(TimeSpan.FromSeconds(1));
        while (await timer.WaitForNextTickAsync(ct))
        {
            var now = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds();
            foreach (var (process, metric) in counters)
            {
                counters.TryRemove(process, out _);
                var tier = metric.Bytes > 10_000_000 ? "HIGH" : metric.Bytes > 1_000_000 ? "MEDIUM" : "LOW";
                var update = new ProcessBandwidthUpdate(process, metric.Bytes, metric.Packets, metric.Protocol, now, tier);
                await _hub.Clients.Group("network").SendAsync("ReceiveProcessStats", update, ct);
            }
        }
    }
}
