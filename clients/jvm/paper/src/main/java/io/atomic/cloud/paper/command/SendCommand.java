package io.atomic.cloud.paper.command;

import com.mojang.brigadier.Command;
import io.atomic.cloud.grpc.unit.TransferManagement;
import io.atomic.cloud.grpc.unit.UnitInformation;
import io.atomic.cloud.paper.CloudPlugin;
import io.atomic.cloud.paper.command.argument.UnitArgument;
import io.atomic.cloud.paper.permission.Permissions;
import io.papermc.paper.command.brigadier.Commands;
import io.papermc.paper.command.brigadier.argument.ArgumentTypes;
import java.util.List;
import java.util.concurrent.CompletableFuture;
import org.bukkit.entity.Player;
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

                            @SuppressWarnings("unchecked")
                            var players = (List<Player>) context.getArgument("user", List.class);
                            var unit = context.getArgument("unit", UnitInformation.SimpleUnitValue.class);
                            var playerCount = players.size();

                            sender.sendRichMessage("Transferring " + playerCount + " to unit " + unit + "...");
                            CompletableFuture.allOf(players.stream()
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
                                            sender.sendRichMessage("Failed to transfer " + playerCount + " to unit "
                                                    + unit + ": " + throwable.getMessage());
                                        } else {
                                            sender.sendRichMessage(
                                                    "Submitted " + playerCount + " transfer requests to controller");
                                        }
                                    });
                            return Command.SINGLE_SUCCESS;
                        })))
                .build());
    }
}
