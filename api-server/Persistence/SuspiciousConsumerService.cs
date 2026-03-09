using System.Threading.Channels;
using AetherLens.Hubs;
using AetherLens.Protos;
using Microsoft.AspNetCore.SignalR;

namespace AetherLens.Persistence;

public sealed class SuspiciousConsumerService : BackgroundService
{
    private readonly ChannelReader<SuspiciousConnectionMessage> _reader;
    private readonly IHubContext<NetworkMonitorHub> _hub;

    public SuspiciousConsumerService(
        ChannelReader<SuspiciousConnectionMessage> reader,
        IHubContext<NetworkMonitorHub> hub)
    {
        _reader = reader;
        _hub = hub;
    }

    protected override async Task ExecuteAsync(CancellationToken stoppingToken)
    {
        while (await _reader.WaitToReadAsync(stoppingToken))
        {
            while (_reader.TryRead(out var item))
            {
                var severity = item.Score >= 0.9 ? "CRITICAL" : item.Score >= 0.8 ? "WARNING" : "INFO";
                var alert = new SuspiciousOutboundConnection(
                    item.Process,
                    item.RemoteAddr,
                    item.Reason,
                    severity,
                    item.Score,
                    (long)item.Timestamp);

                await _hub.Clients.Group("network").SendAsync("ReceiveAnomaly", alert, stoppingToken);
            }
        }
    }
}
