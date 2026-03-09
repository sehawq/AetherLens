using Microsoft.AspNetCore.SignalR;

namespace AetherLens.Hubs;

public class NetworkMonitorHub : Hub
{
    public override async Task OnConnectedAsync()
    {
        // Public Showcase Mode: Add everyone to all groups
        await Groups.AddToGroupAsync(Context.ConnectionId, "network");
        await Groups.AddToGroupAsync(Context.ConnectionId, "network:pro");
        await Groups.AddToGroupAsync(Context.ConnectionId, "ops-seha"); // For system health metrics

        await base.OnConnectedAsync();
    }
}
