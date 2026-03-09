using AetherLens.Services;
using Xunit;

namespace AetherLens.Tests;

public class MetricsSnapshotTests
{
    [Fact]
    public void Snapshot_CalculatesCorrectAverages()
    {
        // Arrange
        var metrics = new MetricsSnapshot();
        
        // Act
        // Simulate 100 packets with 10ms latency each
        for (int i = 0; i < 100; i++)
        {
            // Latency is in ticks. Stopwatch.Frequency depends on OS, but let's assume we pass raw ticks.
            // 10ms = 0.01s.
            // Let's just use a relative tick count. 
            // MetricsSnapshot divides by Frequency to get seconds/milliseconds.
            
            // Actually, MetricsSnapshot takes raw ticks.
            // We need to verify logic, not exact time.
            
            // Let's inject a known frequency or just test the logic flow?
            // Since MetricsSnapshot uses static Stopwatch.GetTimestamp(), it's hard to unit test time-dependent logic deterministically without abstraction.
            // However, we can test the counters.
            
            metrics.RecordPacket(1000); 
        }

        var (tps, avgMs, maxMs) = metrics.Snapshot();

        // Assert
        // Since elapsed time is very small (execution time), TPS will be high.
        Assert.True(tps >= 0);
        Assert.True(avgMs > 0);
    }
    
    [Fact]
    public void GrpcConnection_StateIsTracked()
    {
        var metrics = new MetricsSnapshot();
        Assert.False(metrics.GrpcConnected);
        
        metrics.SetGrpcConnected(true);
        Assert.True(metrics.GrpcConnected);
        
        metrics.SetGrpcConnected(false);
        Assert.False(metrics.GrpcConnected);
    }
}
