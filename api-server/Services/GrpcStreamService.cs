using System.Threading.Channels;
using Grpc.Core;
using Grpc.Net.Client;
using AetherLens.Protos;

namespace AetherLens.Services;

public sealed class GrpcStreamService : BackgroundService
{
    private readonly ChannelWriter<PacketMessage> _writer;
    private readonly MetricsSnapshot _metrics;
    private readonly ILogger<GrpcStreamService> _logger;
    private readonly string _grpcTarget;

    public GrpcStreamService(
        ChannelWriter<PacketMessage> writer,
        MetricsSnapshot metrics,
        ILogger<GrpcStreamService> logger,
        IConfiguration configuration)
    {
        _writer = writer;
        _metrics = metrics;
        _logger = logger;
        _grpcTarget = configuration.GetValue<string>("Core:GrpcTarget") 
                      ?? "http://127.0.0.1:50051";
    }

    protected override async Task ExecuteAsync(CancellationToken stoppingToken)
    {
        var handler = new SocketsHttpHandler
        {
            EnableMultipleHttp2Connections = true,
            KeepAlivePingDelay = TimeSpan.FromSeconds(30),
            KeepAlivePingTimeout = TimeSpan.FromSeconds(5),
            InitialHttp2StreamWindowSize = 2 * 1024 * 1024
        };

        using var grpcChannel = GrpcChannel.ForAddress(_grpcTarget, new GrpcChannelOptions
        {
            HttpHandler = handler,
            MaxReceiveMessageSize = 4 * 1024 * 1024
        });
        var client = new PacketService.PacketServiceClient(grpcChannel);

        var delay = TimeSpan.FromSeconds(2);
        var maxDelay = TimeSpan.FromSeconds(30);

        while (!stoppingToken.IsCancellationRequested)
        {
            try
            {
                _logger.LogInformation("Subscribing to gRPC packet stream...");
                using var stream = client.SubscribePackets(new SubscribeRequest(), cancellationToken: stoppingToken);

                _metrics.SetGrpcConnected(true);
                delay = TimeSpan.FromSeconds(2);

                await foreach (var trade in stream.ResponseStream.ReadAllAsync(stoppingToken))
                {
                    await _writer.WriteAsync(trade, stoppingToken);
                }
            }
            catch (RpcException ex) when (ex.StatusCode is StatusCode.Unavailable or StatusCode.Internal)
            {
                _metrics.SetGrpcConnected(false);
                _logger.LogWarning("gRPC {Status}: {Detail} — retrying in {Delay}s",
                    ex.StatusCode, ex.Status.Detail, delay.TotalSeconds);
                await Task.Delay(delay, stoppingToken);
                delay = TimeSpan.FromTicks(Math.Min(delay.Ticks * 2, maxDelay.Ticks));
            }
            catch (OperationCanceledException) when (stoppingToken.IsCancellationRequested)
            {
                break;
            }
        }

        _metrics.SetGrpcConnected(false);
    }
}
