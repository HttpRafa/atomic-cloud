package de.rafael.atomic.cloud.listener;

import de.rafael.atomic.cloud.CloudPlugin;
import org.bukkit.event.Listener;
import org.bukkit.event.server.ServerLoadEvent;

public class ServerLifecycleListener implements Listener {

  private void onServerLoad(ServerLoadEvent event) {
    if (CloudPlugin.INSTANCE.getSettings().isAutoReady()) {
      CloudPlugin.INSTANCE.getConnection().markReady();
    }
    CloudPlugin.INSTANCE.getConnection().markRunning();
  }
}
