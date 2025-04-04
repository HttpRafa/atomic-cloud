package io.atomic.cloud.paper.send.command;

import com.mojang.brigadier.Command;
import io.atomic.cloud.grpc.client.Transfer;
import io.atomic.cloud.paper.CloudPlugin;
import io.atomic.cloud.paper.send.SendPlugin;
import io.atomic.cloud.paper.send.command.argument.TransferTargetArgument;
import io.atomic.cloud.paper.send.permission.Permissions;
import io.papermc.paper.command.brigadier.Commands;
import io.papermc.paper.command.brigadier.argument.ArgumentTypes;
import io.papermc.paper.command.brigadier.argument.resolvers.selector.PlayerSelectorArgumentResolver;
import net.kyori.adventure.text.minimessage.tag.resolver.Placeholder;
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
                                    var connection = CloudPlugin.INSTANCE.clientConnection();

                                    var users = context.getArgument("user", PlayerSelectorArgumentResolver.class)
                                            .resolve(context.getSource());
                                    var userCount = users.size();
                                    var target = context.getArgument("target", Transfer.Target.class);

                                    SendPlugin.INSTANCE
                                            .messages()
                                            .transferringUsers()
                                            .send(
                                                    sender,
                                                    Placeholder.unparsed("count", String.valueOf(userCount)),
                                                    Placeholder.unparsed("target", formatTarget(target)));
                                    connection
                                            .transferUsers(Transfer.TransferReq.newBuilder()
                                                    .addAllIds(users.stream()
                                                            .map(item -> item.getUniqueId()
                                                                    .toString())
                                                            .toList())
                                                    .setTarget(target)
                                                    .build())
                                            .whenComplete((result, throwable) -> {
                                                if (throwable != null) {
                                                    SendPlugin.INSTANCE
                                                            .messages()
                                                            .transferFailed()
                                                            .send(
                                                                    sender,
                                                                    Placeholder.unparsed(
                                                                            "count", String.valueOf(userCount)),
                                                                    Placeholder.unparsed(
                                                                            "target", formatTarget(target)),
                                                                    Placeholder.unparsed(
                                                                            "error", throwable.getMessage()));
                                                } else {
                                                    SendPlugin.INSTANCE
                                                            .messages()
                                                            .transferSuccessful()
                                                            .send(
                                                                    sender,
                                                                    Placeholder.unparsed(
                                                                            "count", String.valueOf(userCount)));
                                                }
                                            });
                                    return Command.SINGLE_SUCCESS;
                                })))
                .build());
    }

    @Contract(pure = true)
    private static @NotNull String formatTarget(Transfer.@NotNull Target target) {
        switch (target.getType()) {
            case Transfer.Target.Type.FALLBACK -> {
                return "fallback";
            }
            case Transfer.Target.Type.SERVER -> {
                return "server:" + target.getTarget();
            }
            case Transfer.Target.Type.GROUP -> {
                return "group:" + target.getTarget();
            }
        }
        return "unknown";
    }
}
