package io.atomic.cloud.paper.command;

import com.mojang.brigadier.Command;
import io.atomic.cloud.grpc.unit.TransferManagement;
import io.atomic.cloud.grpc.unit.UnitInformation;
import io.atomic.cloud.paper.CloudPlugin;
import io.atomic.cloud.paper.command.argument.UnitArgument;
import io.atomic.cloud.paper.permission.Permissions;
import io.papermc.paper.command.brigadier.Commands;
import io.papermc.paper.command.brigadier.argument.ArgumentTypes;
import io.papermc.paper.command.brigadier.argument.resolvers.selector.PlayerSelectorArgumentResolver;
import java.util.concurrent.CompletableFuture;
import org.jetbrains.annotations.NotNull;

@SuppressWarnings("UnstableApiUsage")
public class SendCommand {

    public static void register(@NotNull Commands commands) {
        commands.register(Commands.literal("send")
                .requires(Permissions.SERVER_COMMAND::check)
                .then(Commands.argument("user", ArgumentTypes.players())
                        .then(Commands.argument("unit", UnitArgument.INSTANCE).executes(context -> {
                            var sender = context.getSource().getSender();
                            var connection = CloudPlugin.INSTANCE.connection();

                            var users = context.getArgument("user", PlayerSelectorArgumentResolver.class)
                                    .resolve(context.getSource());
                            var unit = context.getArgument("unit", UnitInformation.SimpleUnitValue.class);
                            var userCount = users.size();

                            sender.sendRichMessage("<gray>Transferring <aqua>" + userCount
                                    + " <gray>users to unit <blue>" + unit.getName() + "<dark_gray>...");
                            CompletableFuture.allOf(users.stream()
                                            .map((player) -> connection.transferUser(
                                                    TransferManagement.TransferUserRequest.newBuilder()
                                                            .setUserUuid(player.getUniqueId()
                                                                    .toString())
                                                            .setTarget(
                                                                    TransferManagement.TransferTargetValue.newBuilder()
                                                                            .setTargetType(
                                                                                    TransferManagement
                                                                                            .TransferTargetValue
                                                                                            .TargetType.UNIT)
                                                                            .setTarget(unit.getUuid()))
                                                            .build()))
                                            .toArray(CompletableFuture[]::new))
                                    .whenComplete((result, throwable) -> {
                                        if (throwable != null) {
                                            sender.sendRichMessage(
                                                    "<red>Failed to transfer " + userCount + " users to unit "
                                                            + unit.getName() + ": " + throwable.getMessage());
                                        } else {
                                            sender.sendRichMessage("<green>Submitted <aqua>" + userCount
                                                    + " <gray>transfer requests to controller");
                                        }
                                    });
                            return Command.SINGLE_SUCCESS;
                        })))
                .build());
    }
}
