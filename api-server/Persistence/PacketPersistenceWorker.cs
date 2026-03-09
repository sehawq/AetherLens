using System.Threading.Channels;
using AetherLens.Hubs;
using Microsoft.AspNetCore.SignalR;

namespace AetherLens.Persistence;

public sealed class PacketPersistenceWorker : BackgroundService
{
    private readonly ChannelReader<PacketUpdate> _reader;
    private readonly AetherDb _db;
    private readonly ILogger<PacketPersistenceWorker> _logger;

    public PacketPersistenceWorker(
        ChannelReader<PacketUpdate> reader,
        AetherDb db,
        ILogger<PacketPersistenceWorker> logger)
    {
        _reader = reader;
        _db = db;
        _logger = logger;
    }

    protected override async Task ExecuteAsync(CancellationToken stoppingToken)
    {
        var buffer = new List<PacketUpdate>(256);
        while (!stoppingToken.IsCancellationRequested)
        {
            try
            {
                var item = await _reader.ReadAsync(stoppingToken);
                buffer.Add(item);

                while (_reader.TryRead(out var extra) && buffer.Count < 256)
                    buffer.Add(extra);

                if (buffer.Count > 0)
                {
                    await _db.BulkInsertPacketsAsync(buffer);
                    buffer.Clear();
                }
            }
            catch (OperationCanceledException) when (stoppingToken.IsCancellationRequested)
            {
                break;
            }
            catch (Exception ex)
            {
                _logger.LogWarning(ex, "Packet persistence batch failed");
                await Task.Delay(250, stoppingToken);
            }
        }
    }
}
