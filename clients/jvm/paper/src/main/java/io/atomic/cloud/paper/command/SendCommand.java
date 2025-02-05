package io.atomic.cloud.paper.command;

import com.mojang.brigadier.Command;
import io.atomic.cloud.grpc.server.TransferManagement;
import io.atomic.cloud.paper.CloudPlugin;
import io.atomic.cloud.paper.command.argument.TransferTargetArgument;
import io.atomic.cloud.paper.permission.Permissions;
import io.papermc.paper.command.brigadier.Commands;
import io.papermc.paper.command.brigadier.argument.ArgumentTypes;
import io.papermc.paper.command.brigadier.argument.resolvers.selector.PlayerSelectorArgumentResolver;
import org.jetbrains.annotations.Contract;
import org.jetbrains.annotations.NotNull;

@SuppressWarnings("UnstableApiUsage")
public class SendCommand {

    public static void register(@NotNull Commands commands) {
        commands.register(Commands.literal("send")
                .requires(Permissions.SEND_COMMAND::check)
                .then(Commands.argument("user", ArgumentTypes.players())
                        .then(Commands.argument("target", TransferTargetArgument.INSTANCE)
                                .executes(context -> {
                                    var sender = context.getSource().getSender();
                                    var connection = CloudPlugin.INSTANCE.connection();

                                    var users = context.getArgument("user", PlayerSelectorArgumentResolver.class)
                                            .resolve(context.getSource());
                                    var userCount = users.size();
                                    var target =
                                            context.getArgument("target", TransferManagement.TransferTargetValue.class);

                                    sender.sendRichMessage("<gray>Transferring <aqua>" + userCount
                                            + " <gray>users to <blue>" + formatTarget(target) + "<dark_gray>...");

                                    connection
                                            .transferUsers(TransferManagement.TransferUsersRequest.newBuilder()
                                                    .addAllUserUuids(users.stream()
                                                            .map(item -> item.getUniqueId()
                                                                    .toString())
                                                            .toList())
                                                    .setTarget(target)
                                                    .build())
                                            .whenComplete((result, throwable) -> {
                                                if (throwable != null) {
                                                    sender.sendRichMessage("<red>Failed to transfer " + userCount
                                                            + " users to " + formatTarget(target) + ": "
                                                            + throwable.getMessage());
                                                } else {
                                                    sender.sendRichMessage("<green>Submitted <aqua>" + userCount
                                                            + " <gray>transfer requests to controller");
                                                }
                                            });
                                    return Command.SINGLE_SUCCESS;
                                })))
                .build());
    }

    @Contract(pure = true)
    private static @NotNull String formatTarget(TransferManagement.@NotNull TransferTargetValue target) {
        switch (target.getTargetType()) {
            case TransferManagement.TransferTargetValue.TargetType.FALLBACK -> {
                return "fallback";
            }
            case TransferManagement.TransferTargetValue.TargetType.UNIT -> {
                return "server:" + target.getTarget();
            }
            case TransferManagement.TransferTargetValue.TargetType.DEPLOYMENT -> {
                return "group:" + target.getTarget();
            }
        }
        return "unknown";
    }
}
