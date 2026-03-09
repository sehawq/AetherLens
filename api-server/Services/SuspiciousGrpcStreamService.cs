using System.Threading.Channels;
using Grpc.Core;
using Grpc.Net.Client;
using AetherLens.Protos;

namespace AetherLens.Services;

public sealed class SuspiciousGrpcStreamService : BackgroundService
{
    private readonly ChannelWriter<SuspiciousConnectionMessage> _writer;
    private readonly ILogger<SuspiciousGrpcStreamService> _logger;
    private readonly string _grpcTarget;

    public SuspiciousGrpcStreamService(
        ChannelWriter<SuspiciousConnectionMessage> writer,
        ILogger<SuspiciousGrpcStreamService> logger,
        IConfiguration configuration)
    {
        _writer = writer;
        _logger = logger;
        _grpcTarget = configuration.GetValue<string>("Core:GrpcTarget") 
                      ?? "http://127.0.0.1:50051";
    }

    protected override async Task ExecuteAsync(CancellationToken stoppingToken)
    {
        using var grpcChannel = GrpcChannel.ForAddress(_grpcTarget);
        var client = new SuspiciousService.SuspiciousServiceClient(grpcChannel);

        while (!stoppingToken.IsCancellationRequested)
        {
            try
            {
                using var stream = client.SubscribeSuspicious(new SubscribeRequest(), cancellationToken: stoppingToken);
                await foreach (var item in stream.ResponseStream.ReadAllAsync(stoppingToken))
                {
                    await _writer.WriteAsync(item, stoppingToken);
                }
            }
            catch (RpcException ex) when (ex.StatusCode is StatusCode.Unavailable or StatusCode.Internal)
            {
                _logger.LogWarning("Suspicious gRPC stream unavailable: {Detail}", ex.Status.Detail);
                await Task.Delay(TimeSpan.FromSeconds(2), stoppingToken);
            }
            catch (OperationCanceledException) when (stoppingToken.IsCancellationRequested)
            {
                break;
            }
        }
    }
}
