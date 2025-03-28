package io.atomic.cloud.paper.command;

import com.mojang.brigadier.Command;
import io.atomic.cloud.paper.CloudPlugin;
import io.atomic.cloud.paper.enums.MessageEnum;
import io.atomic.cloud.paper.permission.Permissions;
import io.papermc.paper.command.brigadier.Commands;
import org.jetbrains.annotations.NotNull;

@SuppressWarnings("UnstableApiUsage")
public class CloudCommand {

    public static void register(@NotNull Commands commands) {
        commands.register(Commands.literal("cloud")
                .requires(Permissions.CLOUD_COMMAND::check)
                .executes(context -> {
                    var sender = context.getSource().getSender();
                    var connection = CloudPlugin.INSTANCE.connection();

                    connection.getCtrlVer().thenAcceptBoth(connection.getProtoVer(), (version, protocol) -> {
                        sender.sendMessage(MessageEnum.INFO_CMD_LINE.of(null));
                        sender.sendMessage(MessageEnum.INFO_CMD_STRING_1.of(null));
                        sender.sendMessage(MessageEnum.INFO_CMD_STRING_2.of(null));
                        sender.sendMessage(MessageEnum.INFO_CMD_STRING_3.of(
                                null, CloudPlugin.INSTANCE.getPluginMeta().getVersion()));
                        sender.sendMessage(MessageEnum.INFO_CMD_STRING_4.of(null, version.getValue()));
                        sender.sendMessage(MessageEnum.INFO_CMD_STRING_5.of(null, String.valueOf(protocol.getValue())));
                        sender.sendMessage(MessageEnum.INFO_CMD_LINE.of(null));
                    });
                    return Command.SINGLE_SUCCESS;
                })
                .build());
    }
}
