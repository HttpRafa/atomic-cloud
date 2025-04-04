package io.atomic.cloud.paper.setting;

import lombok.Getter;
import lombok.Setter;
import org.bukkit.configuration.file.FileConfiguration;
import org.jetbrains.annotations.NotNull;

@Getter
@Setter
public class Settings {

    private boolean autoReady;
    private boolean suicideOnDisable;

    public Settings(@NotNull FileConfiguration configuration) {
        this.autoReady = configuration.getBoolean("behavior.auto-ready", true);
        this.suicideOnDisable = configuration.getBoolean("behavior.suicide-on-disable", true);
    }
}
