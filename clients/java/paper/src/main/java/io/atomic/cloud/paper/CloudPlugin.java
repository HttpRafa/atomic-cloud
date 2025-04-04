package io.atomic.cloud.paper;

import io.atomic.cloud.api.Cloud;
import io.atomic.cloud.common.channel.ChannelManager;
import io.atomic.cloud.common.connection.client.ClientConnection;
import io.atomic.cloud.common.health.Heart;
import io.atomic.cloud.common.resource.ResourceManager;
import io.atomic.cloud.common.resource.object.SimpleLocalCloudServer;
import io.atomic.cloud.common.transfer.TransferManager;
import io.atomic.cloud.paper.api.CloudImpl;
import io.atomic.cloud.paper.listener.PlayerEventsListener;
import io.atomic.cloud.paper.setting.Settings;
import io.atomic.cloud.paper.setting.message.Messages;
import io.atomic.cloud.paper.transfer.TransferHandler;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import lombok.Getter;
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

    private Settings settings;
    private Messages messages;

    private ChannelManager channels;
    private TransferManager transfers;
    private ResourceManager resources;
    private Heart heart;
    private ClientConnection clientConnection;
    private SimpleLocalCloudServer self;

    private TransferHandler transferHandler;

    @SneakyThrows
    @Override
    public void onLoad() {
        Cloud.setup(new CloudImpl());

        // Load configuration
        saveDefaultConfig();
        this.settings = new Settings(this.getConfig());
        this.messages = new Messages(this.getConfig());

        this.clientConnection = ClientConnection.createFromEnv();
        this.self = new SimpleLocalCloudServer(this.clientConnection);
        this.heart = new Heart(HEART_BEAT_INTERVAL, clientConnection, SCHEDULER);
        this.channels = new ChannelManager(this.clientConnection);
        this.transfers = new TransferManager(this.clientConnection);
        this.resources = new ResourceManager(this.clientConnection);
        this.transferHandler = new TransferHandler(this.clientConnection);

        LOGGER.info("Connecting to controller...");
        this.clientConnection.connect();
        this.heart.start();
    }

    @Override
    public void onEnable() {
        // Register listeners
        registerListeners();

        // Mark server as running
        this.clientConnection.running();
        if (this.settings.autoReady()) {
            this.clientConnection.ready(true);
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
}
