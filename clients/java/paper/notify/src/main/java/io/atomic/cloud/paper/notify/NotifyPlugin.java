package io.atomic.cloud.paper.notify;

import org.bukkit.plugin.java.JavaPlugin;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class NotifyPlugin extends JavaPlugin {

    public static final NotifyPlugin INSTANCE = new NotifyPlugin();
    public static final Logger LOGGER = LoggerFactory.getLogger("ac-notify");
}
