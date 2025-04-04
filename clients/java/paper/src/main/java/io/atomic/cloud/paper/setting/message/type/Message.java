package io.atomic.cloud.paper.setting.message.type;

import net.kyori.adventure.audience.Audience;
import net.kyori.adventure.pointer.Pointered;
import net.kyori.adventure.text.minimessage.tag.resolver.TagResolver;
import org.bukkit.command.CommandSender;

public interface Message {

    void send(Audience audience, Pointered target, TagResolver... resolvers);

    default void send(CommandSender sender, TagResolver... resolvers) {
        this.send(sender, sender, resolvers);
    }
}
