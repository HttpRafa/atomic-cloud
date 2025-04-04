package io.atomic.cloud.paper.setting.message.type;

import io.atomic.cloud.paper.setting.message.Messages;
import net.kyori.adventure.audience.Audience;
import net.kyori.adventure.pointer.Pointered;
import net.kyori.adventure.text.minimessage.tag.resolver.TagResolver;
import org.bukkit.configuration.file.FileConfiguration;
import org.jetbrains.annotations.NotNull;

public class SingleLine implements Message {

    private final String line;

    public SingleLine(String path, @NotNull FileConfiguration configuration) {
        this.line = configuration.getString(path, path);
    }

    @Override
    public void send(@NotNull Audience audience, Pointered target, TagResolver... resolvers) {
        audience.sendMessage(Messages.MINI_MESSAGE.deserialize(this.line, target, resolvers));
    }
}
