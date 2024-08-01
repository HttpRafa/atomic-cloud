package io.atomic.cloud.paper;

import io.atomic.cloud.api.Cloud;
import io.atomic.cloud.common.connection.CloudConnection;
import io.atomic.cloud.common.health.Heart;
import io.atomic.cloud.paper.api.CloudImpl;
import io.atomic.cloud.paper.listener.PlayerEventsListener;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import lombok.Getter;
import lombok.Setter;
import org.bukkit.plugin.java.JavaPlugin;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

@Getter
public class CloudPlugin extends JavaPlugin {

    private static final long HEART_BEAT_INTERVAL = 5;

    public static final CloudPlugin INSTANCE = new CloudPlugin();
    public static final ScheduledExecutorService SCHEDULER = Executors.newScheduledThreadPool(4);
    public static final Logger LOGGER = LoggerFactory.getLogger("atomic-cloud");

    private final Settings settings = new Settings();
    private Heart heart;
    private CloudConnection connection;

    @Override
    public void onLoad() {
        Cloud.setup(new CloudImpl());

        connection = CloudConnection.createFromEnv();
        heart = new Heart(HEART_BEAT_INTERVAL, connection, SCHEDULER);

        LOGGER.info("Connecting to controller...");
        connection.connect();
        heart.start();
    }

    @Override
    public void onEnable() {
        registerListeners();

        // Mark server as running
        connection.markRunning();
        if (settings.autoReady()) {
            connection.markReady();
        }
    }

    @Override
    public void onDisable() {
        try {
            shutdown().thenRun(heart::stop).join();
        } catch (Throwable throwable) {
            LOGGER.error("Error while shutting down", throwable);
        }
    }

    private void registerListeners() {
        var pluginManager = getServer().getPluginManager();
        pluginManager.registerEvents(new PlayerEventsListener(), this);
    }

    public CompletableFuture<Void> shutdown() {
        return connection.requestStop().thenApply(empty -> null);
    }

    @Getter
    @Setter
    public static class Settings {

        public boolean autoReady = true;
    }
}