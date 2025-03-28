package io.atomic.cloud.paper;

import io.atomic.cloud.api.Cloud;
import io.atomic.cloud.common.channel.ChannelManager;
import io.atomic.cloud.common.connection.CloudConnection;
import io.atomic.cloud.common.health.Heart;
import io.atomic.cloud.common.resource.ResourceManager;
import io.atomic.cloud.common.resource.object.SimpleLocalCloudServer;
import io.atomic.cloud.common.transfer.TransferManager;
import io.atomic.cloud.paper.api.CloudImpl;
import io.atomic.cloud.paper.listener.PlayerEventsListener;
import io.atomic.cloud.paper.transfer.TransferHandler;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import lombok.Getter;
import lombok.Setter;
import lombok.SneakyThrows;
import org.bukkit.plugin.java.JavaPlugin;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

@Getter
public class CloudPlugin extends JavaPlugin {

    private static final long HEART_BEAT_INTERVAL = 5;

    public static final CloudPlugin INSTANCE = new CloudPlugin();
    public static final ScheduledExecutorService SCHEDULER = Executors.newScheduledThreadPool(4);
    public static final Logger LOGGER = LoggerFactory.getLogger("ac-core");

    private final Settings settings = new Settings();
    private ChannelManager channels;
    private TransferManager transfers;
    private ResourceManager resources;
    private Heart heart;
    private CloudConnection connection;
    private SimpleLocalCloudServer self;

    private TransferHandler transferHandler;

    @SneakyThrows
    @Override
    public void onLoad() {
        Cloud.setup(new CloudImpl());

        this.connection = CloudConnection.createFromEnv();
        this.self = new SimpleLocalCloudServer(this.connection);
        this.heart = new Heart(HEART_BEAT_INTERVAL, connection, SCHEDULER);
        this.channels = new ChannelManager(this.connection);
        this.transfers = new TransferManager(this.connection);
        this.resources = new ResourceManager(this.connection);
        this.transferHandler = new TransferHandler(this.connection);

        LOGGER.info("Connecting to controller...");
        this.connection.connect();
        this.heart.start();
    }

    @Override
    public void onEnable() {
        // Register listeners
        registerListeners();

        // Mark server as running
        this.connection.setRunning();
        if (this.settings.autoReady()) {
            this.connection.setReady(true);
        }

        // Enable transfer system
        this.transferHandler.enable();
    }

    @Override
    public void onDisable() {
        // Cleanup
        this.transferHandler.cleanup();

        try {
            if (this.settings.suicideOnDisable()) {
                this.self.shutdown().thenRun(heart::stop).join();
            } else {
                this.heart.stop();
            }
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
        public boolean suicideOnDisable = true;
    }
}
