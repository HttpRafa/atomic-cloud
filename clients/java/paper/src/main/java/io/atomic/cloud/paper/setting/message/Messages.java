package io.atomic.cloud.paper.setting.message;

import io.atomic.cloud.paper.setting.message.type.Message;
import io.atomic.cloud.paper.setting.message.type.MultiLine;
import io.atomic.cloud.paper.setting.message.type.SingleLine;
import lombok.Getter;
import net.kyori.adventure.text.minimessage.MiniMessage;
import org.bukkit.configuration.file.FileConfiguration;

@Getter
public class Messages {

    public static final MiniMessage MINI_MESSAGE = MiniMessage.miniMessage();

    private final Message infos;
    private final Message notReady;
    private final Message transferAllUsers;

    public Messages(FileConfiguration configuration) {
        this.infos = new MultiLine("messages.infos", configuration);
        this.notReady = new SingleLine("messages.not-ready", configuration);
        this.transferAllUsers = new SingleLine("messages.transfer-all-users", configuration);
    }
}
