package io.atomic.cloud.paper.proxy.setting;

import lombok.Getter;
import lombok.Setter;
import org.bukkit.configuration.file.FileConfiguration;
import org.jetbrains.annotations.NotNull;

@Getter
@Setter
public class Settings {

    private boolean changeOnlinePlayers;

    public Settings(@NotNull FileConfiguration configuration) {
        this.changeOnlinePlayers = configuration.getBoolean("behavior.change-online-players", true);
    }
}
