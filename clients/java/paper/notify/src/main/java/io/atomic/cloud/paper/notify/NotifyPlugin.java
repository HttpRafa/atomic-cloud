package io.atomic.cloud.paper.notify;

import io.atomic.cloud.paper.CloudPlugin;
import io.atomic.cloud.paper.notify.notify.PowerHandler;
import io.atomic.cloud.paper.notify.setting.message.Messages;
import lombok.Getter;
import org.bukkit.plugin.java.JavaPlugin;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

@Getter
public class NotifyPlugin extends JavaPlugin {

    public static final NotifyPlugin INSTANCE = new NotifyPlugin();
    public static final Logger LOGGER = LoggerFactory.getLogger("ac-notify");

    private Messages messages;

    private PowerHandler powerHandler;

    @Override
    public void onLoad() {
        // Load configuration
        saveDefaultConfig();
        this.messages = new Messages(this.getConfig());

        this.powerHandler = new PowerHandler(CloudPlugin.INSTANCE.clientConnection());
    }

    @Override
    public void onEnable() {
        // Enable the notification system
        this.powerHandler.enable();
    }

    @Override
    public void onDisable() {
        // Cleanup
        this.powerHandler.cleanup();
    }
}
