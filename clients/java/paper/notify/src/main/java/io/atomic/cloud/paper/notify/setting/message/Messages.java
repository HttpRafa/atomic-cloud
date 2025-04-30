package io.atomic.cloud.paper.notify.setting.message;

import io.atomic.cloud.paper.setting.message.type.Message;
import io.atomic.cloud.paper.setting.message.type.SingleLine;
import lombok.Getter;
import org.bukkit.configuration.file.FileConfiguration;

@Getter
public class Messages {

    private final Message serverStarting;
    private final Message serverStopping;

    private final Message serverReady;
    private final Message serverNotReady;

    public Messages(FileConfiguration configuration) {
        this.serverStarting = new SingleLine("messages.server-starting", configuration);
        this.serverStopping = new SingleLine("messages.server-stopping", configuration);

        this.serverReady = new SingleLine("messages.server-ready", configuration);
        this.serverNotReady = new SingleLine("messages.server-not-ready", configuration);
    }
}
