package io.atomic.cloud.paper.send;

import org.bukkit.plugin.java.JavaPlugin;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class SendPlugin extends JavaPlugin {

    public static final SendPlugin INSTANCE = new SendPlugin();
    public static final Logger LOGGER = LoggerFactory.getLogger("cloud-send");
}
