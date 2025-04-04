package io.atomic.cloud.paper.send;

import io.atomic.cloud.paper.send.setting.message.Messages;
import lombok.Getter;
import org.bukkit.plugin.java.JavaPlugin;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

@Getter
public class SendPlugin extends JavaPlugin {

    public static final SendPlugin INSTANCE = new SendPlugin();
    public static final Logger LOGGER = LoggerFactory.getLogger("ac-send");

    private Messages messages;

    @Override
    public void onLoad() {
        // Load configuration
        saveDefaultConfig();
        this.messages = new Messages(this.getConfig());
    }
}
