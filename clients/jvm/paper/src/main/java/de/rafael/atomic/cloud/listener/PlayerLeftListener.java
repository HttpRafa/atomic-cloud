package de.rafael.atomic.cloud.listener;

import org.bukkit.event.EventHandler;
import org.bukkit.event.EventPriority;
import org.bukkit.event.Listener;
import org.bukkit.event.player.PlayerQuitEvent;

public class PlayerLeftListener implements Listener {

    @EventHandler(priority = EventPriority.HIGHEST)
    private void onPlayerLeft(PlayerQuitEvent event) {}
}
