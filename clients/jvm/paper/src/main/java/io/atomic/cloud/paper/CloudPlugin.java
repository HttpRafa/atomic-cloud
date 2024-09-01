package io.atomic.cloud.paper;

import io.atomic.cloud.api.Cloud;
import io.atomic.cloud.common.channel.ChannelManager;
import io.atomic.cloud.common.connection.CloudConnection;
import io.atomic.cloud.common.health.Heart;
import io.atomic.cloud.common.server.SimpleCloudServer;
import io.atomic.cloud.paper.api.CloudImpl;
import io.atomic.cloud.paper.listener.PlayerEventsListener;
import io.atomic.cloud.paper.transfer.TransferHandler;
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
    private final ChannelManager channels = new ChannelManager();
    private Heart heart;
    private CloudConnection connection;
    private SimpleCloudServer self;

    private TransferHandler transferHandler;

    @Override
    public void onLoad() {
        Cloud.setup(new CloudImpl());

        this.connection = CloudConnection.createFromEnv();
        this.self = new SimpleCloudServer(this.connection);
        this.heart = new Heart(HEART_BEAT_INTERVAL, connection, SCHEDULER);
        this.transferHandler = new TransferHandler(this.connection);

        LOGGER.info("Connecting to controller...");
        this.connection.connect();
        this.connection.sendReset();
        this.heart.start();
    }

    @Override
    public void onEnable() {
        // Register listeners
        registerListeners();

        // Mark server as running
        this.connection.markRunning();
        if (this.settings.autoReady()) {
            this.connection.markReady();
        }

        // Enable transfer system
        this.transferHandler.enable();
    }

    @Override
    public void onDisable() {
        // Cleanup
        this.channels.cleanup();

        try {
            this.self.shutdown().thenRun(heart::stop).join();
        } catch (Throwable throwable) {
            LOGGER.error("Error while shutting down", throwable);
        }
    }

    private void registerListeners() {
        var pluginManager = getServer().getPluginManager();
        pluginManager.registerEvents(new PlayerEventsListener(), this);
    }

    @Getter
    @Setter
    public static class Settings {

        public boolean autoReady = true;
    }
}
