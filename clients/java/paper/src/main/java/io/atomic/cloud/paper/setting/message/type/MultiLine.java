package io.atomic.cloud.paper.setting.message.type;

import io.atomic.cloud.paper.setting.message.Messages;
import net.kyori.adventure.audience.Audience;
import net.kyori.adventure.pointer.Pointered;
import net.kyori.adventure.text.minimessage.tag.resolver.TagResolver;
import org.bukkit.configuration.file.FileConfiguration;
import org.jetbrains.annotations.NotNull;

public class MultiLine implements Message {

    private final String[] lines;

    public MultiLine(String path, @NotNull FileConfiguration configuration) {
        this.lines = configuration.getStringList(path).toArray(new String[0]);
    }

    @Override
    public void send(Audience audience, Pointered target, TagResolver... resolvers) {
        for (String line : this.lines) {
            audience.sendMessage(Messages.MINI_MESSAGE.deserialize(line, target, resolvers));
        }
    }
}
