using System.Text;
using System.Threading.Channels;
using AetherLens;
using AetherLens.Hubs;
using AetherLens.Protos;
using AetherLens.Persistence;
using AetherLens.Middleware;
using AetherLens.Services;
using Microsoft.OpenApi.Models;

var builder = WebApplication.CreateBuilder(args);

var serverUrls = builder.Configuration["Server:Urls"] ?? Environment.GetEnvironmentVariable("ASPNETCORE_URLS");
builder.WebHost.UseUrls(string.IsNullOrWhiteSpace(serverUrls) ? "http://0.0.0.0:5000;http://[::]:5000" : serverUrls);

var channel = Channel.CreateBounded<PacketMessage>(new BoundedChannelOptions(10_000)
{
    FullMode = BoundedChannelFullMode.DropOldest,
    SingleWriter = true,
    SingleReader = true
});

builder.Services.AddSingleton(channel.Reader);
builder.Services.AddSingleton(channel.Writer);

var suspiciousChannel = Channel.CreateBounded<SuspiciousConnectionMessage>(
    new BoundedChannelOptions(4_000)
    {
        FullMode = BoundedChannelFullMode.DropOldest,
        SingleWriter = true,
        SingleReader = true
    });
builder.Services.AddSingleton(suspiciousChannel.Reader);
builder.Services.AddSingleton(suspiciousChannel.Writer);

var persistenceChannel = Channel.CreateBounded<PacketUpdate>(
    new BoundedChannelOptions(50_000)
    {
        FullMode = BoundedChannelFullMode.DropOldest,
        SingleWriter = true,
        SingleReader = true
    });

builder.Services.AddSingleton(persistenceChannel.Reader);
builder.Services.AddSingleton(persistenceChannel.Writer);

var connectionString = builder.Configuration.GetConnectionString("LocalDb")
    ?? "Data Source=aether_lens.db";

builder.Services.AddSingleton(new AetherDb(connectionString));
builder.Services.AddSingleton<MetricsSnapshot>();

builder.Services.AddSignalR()
    .AddMessagePackProtocol();

var corsOrigins = builder.Configuration.GetSection("Cors:AllowedOrigins").Get<string[]>()
    ?? ["http://localhost:3000", "http://localhost:3001"];

static bool IsLocalDashboardOrigin(string origin)
{
    if (!Uri.TryCreate(origin, UriKind.Absolute, out var uri))
        return false;

    var host = uri.Host;
    var isLocalHost =
        string.Equals(host, "localhost", StringComparison.OrdinalIgnoreCase) ||
        host == "127.0.0.1" ||
        host.StartsWith("192.168.", StringComparison.OrdinalIgnoreCase) ||
        host.StartsWith("10.", StringComparison.OrdinalIgnoreCase) ||
        host.StartsWith("172.16.", StringComparison.OrdinalIgnoreCase) ||
        host.StartsWith("172.17.", StringComparison.OrdinalIgnoreCase) ||
        host.StartsWith("172.18.", StringComparison.OrdinalIgnoreCase) ||
        host.StartsWith("172.19.", StringComparison.OrdinalIgnoreCase) ||
        host.StartsWith("172.2", StringComparison.OrdinalIgnoreCase) ||
        host.StartsWith("172.30.", StringComparison.OrdinalIgnoreCase) ||
        host.StartsWith("172.31.", StringComparison.OrdinalIgnoreCase);

    return isLocalHost && (uri.Port == 3000 || uri.Port == 3001);
}

builder.Services.AddCors(options =>
{
    options.AddDefaultPolicy(policy =>
    {
        policy.WithOrigins(corsOrigins)
            .SetIsOriginAllowed(origin =>
                corsOrigins.Contains(origin, StringComparer.OrdinalIgnoreCase) ||
                IsLocalDashboardOrigin(origin))
            .AllowAnyHeader()
            .AllowAnyMethod()
            .AllowCredentials();
    });
});

builder.Services.AddHostedService<GrpcStreamService>();
builder.Services.AddHostedService<SuspiciousGrpcStreamService>();
builder.Services.AddHostedService<PacketConsumerService>();
builder.Services.AddHostedService<SuspiciousConsumerService>();
builder.Services.AddHostedService<PacketPersistenceWorker>();
builder.Services.AddHostedService<SystemMonitorService>();

builder.Services.AddEndpointsApiExplorer();
builder.Services.AddSwaggerGen(options =>
{
    options.SwaggerDoc("v1", new OpenApiInfo
    {
        Title = "AetherLens API",
        Version = "v1",
        Description = "Real-time network traffic analysis engine."
    });
});

var app = builder.Build();

var aetherDb = app.Services.GetRequiredService<AetherDb>();
await aetherDb.InitializeAsync();

app.UseCors();

if (app.Environment.IsDevelopment())
{
    app.UseSwagger();
    app.UseSwaggerUI(options =>
    {
        options.SwaggerEndpoint("/swagger/v1/swagger.json", "AetherLens API v1");
        options.DocumentTitle = "AetherLens API Explorer";
    });
}

app.UseMiddleware<RateLimitMiddleware>();
app.MapHub<NetworkMonitorHub>("/hubs/aether-lens");

Console.WriteLine("[AETHER-LENS] SignalR hub mapped at /hubs/aether-lens");
Console.WriteLine($"[AETHER-LENS] CORS enabled for {string.Join(", ", corsOrigins)}");
Console.WriteLine("[AETHER-LENS] System running in Public Showcase Mode");

await app.RunAsync();
