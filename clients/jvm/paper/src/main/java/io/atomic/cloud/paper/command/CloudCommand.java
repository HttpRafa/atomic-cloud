package io.atomic.cloud.paper.command;

import com.google.protobuf.StringValue;
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

                    CloudPlugin.INSTANCE
                            .connection()
                            .getControllerVersion()
                            .thenApply(StringValue::getValue)
                            .thenAccept(version -> {
                                sender.sendMessage(Component.text("╔════════════════════", NamedTextColor.GRAY));
                                sender.sendRichMessage("<gray>║ <gradient:#084CFB:#43E8FF>AtomicCloud</gradient>");
                                sender.sendRichMessage(
                                        "<gray>║ <gradient:#084CFB:#43E8FF>Client Version</gradient> <gray>| <gradient:#43E8FF:#0898FB>"
                                                + CloudPlugin.INSTANCE
                                                        .getPluginMeta()
                                                        .getVersion() + "</gradient>");
                                sender.sendRichMessage(
                                        "<gray>║ <gradient:#084CFB:#43E8FF>Controller Version</gradient> <gray>| <gradient:#43E8FF:#0898FB>"
                                                + version + "</gradient>");
                                sender.sendMessage(Component.text("╚════════════════════", NamedTextColor.GRAY));
                            });

                    return Command.SINGLE_SUCCESS;
                })
                .build());
    }
}
