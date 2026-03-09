using System.Collections.Concurrent;
using System.Net;
using System.Text.Json;

namespace AetherLens.Middleware;

public sealed class RateLimitMiddleware
{
    private readonly RequestDelegate _next;

    // Simplified sliding window per IP
    private static readonly ConcurrentDictionary<string, SlidingWindow> _windows = new();
    private static readonly TimeSpan WindowSize = TimeSpan.FromMinutes(1);
    
    // Generous limit for public showcase
    private const int DefaultLimit = 1000;

    public RateLimitMiddleware(RequestDelegate next)
    {
        _next = next;
    }

    public async Task InvokeAsync(HttpContext ctx)
    {
        // skip non-API paths (SignalR, health, etc.)
        if (!ctx.Request.Path.StartsWithSegments("/api"))
        {
            await _next(ctx);
            return;
        }

        // Just use IP for identification
        var ip = ctx.Connection.RemoteIpAddress?.ToString() ?? "unknown";
        var clientKey = $"ip:{ip}";
        var limit = DefaultLimit;

        var window = _windows.GetOrAdd(clientKey, _ => new SlidingWindow());
        var now = DateTimeOffset.UtcNow;

        // Try to consume a request slot
        if (!window.TryConsume(now, limit, WindowSize))
        {
            ctx.Response.StatusCode = (int)HttpStatusCode.TooManyRequests;
            ctx.Response.Headers["Retry-After"] = "60";
            ctx.Response.Headers["X-RateLimit-Limit"] = limit.ToString();
            ctx.Response.ContentType = "application/json";

            await ctx.Response.WriteAsync(
                JsonSerializer.Serialize(new { error = "Rate limit exceeded (Public Demo)", retryAfterSeconds = 60 }));
            return;
        }

        // Add headers for visibility
        var remaining = window.Remaining(now, limit, WindowSize);
        ctx.Response.Headers["X-RateLimit-Limit"] = limit.ToString();
        ctx.Response.Headers["X-RateLimit-Remaining"] = remaining.ToString();

        await _next(ctx);
    }
}

internal sealed class SlidingWindow
{
    private readonly object _lock = new();
    private readonly Queue<DateTimeOffset> _timestamps = new();

    public bool TryConsume(DateTimeOffset now, int limit, TimeSpan window)
    {
        lock (_lock)
        {
            Evict(now, window);
            if (_timestamps.Count >= limit)
            {
                // If full, cannot consume
                return false;
            }

            _timestamps.Enqueue(now);
            return true;
        }
    }

    public int Remaining(DateTimeOffset now, int limit, TimeSpan window)
    {
        lock (_lock)
        {
            Evict(now, window);
            return Math.Max(0, limit - _timestamps.Count);
        }
    }

    private void Evict(DateTimeOffset now, TimeSpan window)
    {
        var cutoff = now - window;
        while (_timestamps.Count > 0 && _timestamps.Peek() < cutoff)
        {
            _timestamps.Dequeue();
        }
    }
}
