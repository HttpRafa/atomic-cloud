package io.atomic.cloud.paper.command;

import com.mojang.brigadier.Command;
import io.atomic.cloud.paper.CloudPlugin;
import io.atomic.cloud.paper.permission.Permissions;
import io.papermc.paper.command.brigadier.Commands;
import net.kyori.adventure.text.Component;
import net.kyori.adventure.text.format.NamedTextColor;
import org.jetbrains.annotations.NotNull;

public class CloudCommand {

    public static void register(@NotNull Commands commands) {
        commands.register(Commands.literal("cloud")
                .requires(Permissions.CLOUD_COMMAND::check)
                .executes(context -> {
                    var sender = context.getSource().getSender();
                    var connection = CloudPlugin.INSTANCE.connection();

                    connection
                            .getControllerVersion()
                            .thenAcceptBoth(connection.getProtocolVersion(), (version, protocol) -> {
                                sender.sendMessage(Component.text("╔════════════════════", NamedTextColor.GRAY));
                                sender.sendRichMessage("<gray>║ <gradient:#084CFB:#43E8FF>AtomicCloud</gradient>");
                                sender.sendRichMessage(
                                        "<gray>║ <gradient:#084CFB:#43E8FF>Client Version</gradient> <gray>| <gradient:#43E8FF:#0898FB>"
                                                + CloudPlugin.INSTANCE
                                                        .getPluginMeta()
                                                        .getVersion() + "</gradient>");
                                sender.sendRichMessage(
                                        "<gray>║ <gradient:#084CFB:#43E8FF>Controller Version</gradient> <gray>| <gradient:#43E8FF:#0898FB>"
                                                + version.getValue() + "</gradient>");
                                sender.sendRichMessage(
                                        "<gray>║ <gradient:#084CFB:#43E8FF>Controller Protocol Version</gradient> <gray>| <gradient:#43E8FF:#0898FB>"
                                                + protocol.getValue() + "</gradient>");
                                sender.sendMessage(Component.text("╚════════════════════", NamedTextColor.GRAY));
                            });

                    return Command.SINGLE_SUCCESS;
                })
                .build());
    }
}
