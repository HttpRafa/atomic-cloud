package io.atomic.cloud.paper.send.setting.message;

import io.atomic.cloud.paper.setting.message.type.Message;
import io.atomic.cloud.paper.setting.message.type.SingleLine;
import lombok.Getter;
import org.bukkit.configuration.file.FileConfiguration;

@Getter
public class Messages {

    private final Message transferringUsers;
    private final Message transferFailed;
    private final Message transferSuccessful;

    public Messages(FileConfiguration configuration) {
        this.transferringUsers = new SingleLine("messages.transferring-users", configuration);
        this.transferFailed = new SingleLine("messages.transfer-failed", configuration);
        this.transferSuccessful = new SingleLine("messages.transfer-successful", configuration);
    }
}
