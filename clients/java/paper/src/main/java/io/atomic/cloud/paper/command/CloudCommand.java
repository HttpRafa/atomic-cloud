package io.atomic.cloud.paper.command;

import com.mojang.brigadier.Command;
import io.atomic.cloud.paper.CloudPlugin;
import io.atomic.cloud.paper.permission.Permissions;
import io.papermc.paper.command.brigadier.Commands;
import net.kyori.adventure.text.minimessage.tag.resolver.Placeholder;
import org.jetbrains.annotations.NotNull;

@SuppressWarnings("UnstableApiUsage")
public class CloudCommand {

    public static void register(@NotNull Commands commands) {
        commands.register(Commands.literal("cloud")
                .requires(Permissions.CLOUD_COMMAND::check)
                .executes(context -> {
                    var sender = context.getSource().getSender();
                    var connection = CloudPlugin.INSTANCE.connection();

                    connection
                            .getCtrlVer()
                            .thenAcceptBoth(connection.getProtoVer(), (version, protocol) -> CloudPlugin.INSTANCE
                                    .messages()
                                    .infos()
                                    .send(
                                            sender,
                                            Placeholder.unparsed(
                                                    "client",
                                                    CloudPlugin.INSTANCE
                                                            .getPluginMeta()
                                                            .getVersion()),
                                            Placeholder.unparsed("controller", version.getValue()),
                                            Placeholder.unparsed("protocol", String.valueOf(protocol.getValue()))));
                    return Command.SINGLE_SUCCESS;
                })
                .build());
    }
}
