package io.atomic.cloud.paper.listener;

import io.atomic.cloud.grpc.unit.UserManagement;
import io.atomic.cloud.paper.CloudPlugin;
import org.bukkit.event.EventHandler;
import org.bukkit.event.EventPriority;
import org.bukkit.event.Listener;
import org.bukkit.event.player.PlayerJoinEvent;
import org.bukkit.event.player.PlayerQuitEvent;
import org.jetbrains.annotations.NotNull;

public class PlayerEventsListener implements Listener {

    @EventHandler(priority = EventPriority.LOWEST)
    public void onPlayerJoin(@NotNull PlayerJoinEvent event) {
        var player = event.getPlayer();
        var user = UserManagement.UserConnectedRequest.newBuilder()
                .setName(player.getName())
                .setUuid(player.getUniqueId().toString())
                .build();
        CloudPlugin.INSTANCE.connection().userConnected(user);
    }

    @EventHandler(priority = EventPriority.HIGHEST)
    private void onPlayerLeft(@NotNull PlayerQuitEvent event) {
        var player = event.getPlayer();
        var user = UserManagement.UserDisconnectedRequest.newBuilder()
                .setUuid(player.getUniqueId().toString())
                .build();
        CloudPlugin.INSTANCE.connection().userDisconnected(user);
    }
}
