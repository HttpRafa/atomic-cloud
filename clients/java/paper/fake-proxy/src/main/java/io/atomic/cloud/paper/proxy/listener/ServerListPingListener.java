package io.atomic.cloud.paper.proxy.listener;

import com.destroystokyo.paper.event.server.PaperServerListPingEvent;
import io.atomic.cloud.paper.CloudPlugin;
import org.bukkit.event.EventHandler;
import org.bukkit.event.EventPriority;
import org.bukkit.event.Listener;
import org.jetbrains.annotations.NotNull;

public class ServerListPingListener implements Listener {

    @EventHandler(priority = EventPriority.LOWEST)
    public void on(@NotNull PaperServerListPingEvent event) {
        CloudPlugin.INSTANCE
                .clientConnection()
                .userCountNow()
                .ifPresent(uInt32Value -> event.setNumPlayers(uInt32Value.getValue()));
    }
}
