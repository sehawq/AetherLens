using Dapper;
using AetherLens.Hubs;
using Microsoft.Data.Sqlite;

namespace AetherLens.Persistence;

public sealed class AetherDb : IAsyncDisposable
{
    private readonly string _connectionString;
    private SqliteConnection? _conn;

    public AetherDb(string connectionString) => _connectionString = connectionString;

    public async Task InitializeAsync()
    {
        _conn = new SqliteConnection(_connectionString);
        await _conn.OpenAsync();
        await _conn.ExecuteAsync("PRAGMA journal_mode=WAL;");
        await _conn.ExecuteAsync("PRAGMA synchronous=NORMAL;");

        await _conn.ExecuteAsync("""
            CREATE TABLE IF NOT EXISTS packet_events (
                id             INTEGER PRIMARY KEY AUTOINCREMENT,
                process_name   TEXT NOT NULL,
                local_address  TEXT NOT NULL,
                remote_address TEXT NOT NULL,
                protocol       TEXT NOT NULL,
                bytes          INTEGER NOT NULL,
                outbound       INTEGER NOT NULL,
                event_time_ms  INTEGER NOT NULL,
                latency_ms     REAL NOT NULL
            );
            """);

        await _conn.ExecuteAsync(
            "CREATE INDEX IF NOT EXISTS idx_packet_time ON packet_events (event_time_ms DESC);");
    }

    public async Task BulkInsertPacketsAsync(List<PacketUpdate> batch)
    {
        for (var attempt = 0; attempt < 2; attempt++)
        {
            try
            {
                await EnsureConnectedAsync();
                await using var tx = await _conn!.BeginTransactionAsync();
                const string sql = """
                    INSERT INTO packet_events
                    (process_name, local_address, remote_address, protocol, bytes, outbound, event_time_ms, latency_ms)
                    VALUES (@Process, @LocalAddress, @RemoteAddress, @Protocol, @Bytes, @OutboundNum, @Timestamp, @LatencyMs)
                    """;
                foreach (var item in batch)
                {
                    await _conn.ExecuteAsync(sql, new
                    {
                        item.Process,
                        item.LocalAddress,
                        item.RemoteAddress,
                        item.Protocol,
                        item.Bytes,
                        OutboundNum = item.Outbound ? 1 : 0,
                        item.Timestamp,
                        item.LatencyMs
                    }, tx);
                }
                await tx.CommitAsync();
                return;
            }
            catch when (attempt == 0)
            {
                _conn?.Dispose();
                _conn = null;
            }
        }
    }

    private async Task EnsureConnectedAsync()
    {
        if (_conn is { State: System.Data.ConnectionState.Open })
            return;

        _conn?.Dispose();
        _conn = new SqliteConnection(_connectionString);
        await _conn.OpenAsync();
    }

    public async ValueTask DisposeAsync()
    {
        if (_conn is not null)
            await _conn.DisposeAsync();
    }
}
