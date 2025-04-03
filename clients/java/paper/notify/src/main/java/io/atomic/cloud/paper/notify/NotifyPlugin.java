package io.atomic.cloud.paper.notify;

import io.atomic.cloud.paper.CloudPlugin;
import io.atomic.cloud.paper.notify.notify.PowerHandler;
import org.bukkit.plugin.java.JavaPlugin;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class NotifyPlugin extends JavaPlugin {

    public static final NotifyPlugin INSTANCE = new NotifyPlugin();
    public static final Logger LOGGER = LoggerFactory.getLogger("ac-notify");

    private PowerHandler powerHandler;

    @Override
    public void onLoad() {
        this.powerHandler = new PowerHandler(CloudPlugin.INSTANCE.connection());
    }

    @Override
    public void onEnable() {
        // Enable notification system
        this.powerHandler.enable();
    }

    @Override
    public void onDisable() {
        // Cleanup
        this.powerHandler.cleanup();
    }
}
