using AetherLens.Hubs;
using Microsoft.AspNetCore.SignalR;

namespace AetherLens.Services;

public sealed class SystemMonitorService : BackgroundService
{
    private readonly IHubContext<NetworkMonitorHub> _hub;
    private readonly MetricsSnapshot _metrics;

    public SystemMonitorService(IHubContext<NetworkMonitorHub> hub, MetricsSnapshot metrics)
    {
        _hub = hub;
        _metrics = metrics;
    }

    protected override async Task ExecuteAsync(CancellationToken stoppingToken)
    {
        while (!stoppingToken.IsCancellationRequested)
        {
            await Task.Delay(2000, stoppingToken);

            var (pps, avgMs, maxMs) = _metrics.Snapshot();
            var uptime = DateTimeOffset.UtcNow - _metrics.StartTime;

            var health = new SystemHealth(
                GrpcConnected: _metrics.GrpcConnected,
                PacketsPerSecond: Math.Round(pps, 1),
                AvgLatencyMs: Math.Round(avgMs, 2),
                MaxLatencyMs: Math.Round(maxMs, 2),
                Uptime: $"{(int)uptime.TotalHours:D2}:{uptime.Minutes:D2}:{uptime.Seconds:D2}");

            await _hub.Clients.Group("ops-seha").SendAsync("ReceiveHealth", health, stoppingToken);
        }
    }
}
