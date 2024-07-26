package de.rafael.atomic.cloud;

import de.rafael.atomic.cloud.api.Cloud;
import de.rafael.atomic.cloud.api.CloudImpl;
import de.rafael.atomic.cloud.common.CloudConnection;
import de.rafael.atomic.cloud.heart.Heart;
import de.rafael.atomic.cloud.listener.ServerLifecycleListener;
import lombok.Getter;
import lombok.Setter;
import org.bukkit.plugin.java.JavaPlugin;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

@Getter
public class CloudPlugin extends JavaPlugin {

  private static final long HEART_BEAT_INTERVAL = 5;

  public static final CloudPlugin INSTANCE = new CloudPlugin();
  public static final Logger LOGGER = LoggerFactory.getLogger("atomic-cloud");

  private final Settings settings = new Settings();
  private Heart heart;
  private CloudConnection connection;

  @Override
  public void onLoad() {
    Cloud.setup(new CloudImpl());

    LOGGER.info("Loading cloud client...");
    connection = CloudConnection.createFromEnv();
    heart = new Heart(HEART_BEAT_INTERVAL);

    registerListeners();

    LOGGER.info("Connecting to controller...");
    connection.connect();
    heart.start();
  }

  @Override
  public void onEnable() {}

  @Override
  public void onDisable() {
    LOGGER.info("Stopping cloud client...");
    heart.stop();
  }

  private void registerListeners() {
    this.getServer().getPluginManager().registerEvents(new ServerLifecycleListener(), this);
  }

  @Getter
  @Setter
  public static class Settings {

    public boolean autoReady = true;
  }
}
