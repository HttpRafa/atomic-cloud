package io.atomic.cloud.paper.command;

import com.mojang.brigadier.Command;
import io.atomic.cloud.grpc.client.Transfer;
import io.atomic.cloud.paper.CloudPlugin;
import io.atomic.cloud.paper.enums.MessageEnum;
import io.atomic.cloud.paper.permission.Permissions;
import io.papermc.paper.command.brigadier.Commands;
import java.util.concurrent.TimeUnit;
import org.bukkit.Bukkit;
import org.jetbrains.annotations.NotNull;

@SuppressWarnings("UnstableApiUsage")
public class DisposeCommand {

    public static void register(@NotNull Commands commands) {
        commands.register(Commands.literal("dispose")
                .requires(Permissions.DISPOSE_COMMAND::check)
                .executes(context -> {
                    var sender = context.getSource().getSender();
                    var connection = CloudPlugin.INSTANCE.connection();

                    sender.sendMessage(MessageEnum.SERVER_NOT_READY.of(MessageEnum.PREFIX));
                    connection.setReady(false).thenRun(() -> {
                        sender.sendMessage(MessageEnum.TRANSFER_USER_ALL.of(MessageEnum.PREFIX));
                        connection.transferUsers(Transfer.TransferReq.newBuilder()
                                .addAllIds(Bukkit.getOnlinePlayers().stream()
                                        .map(item -> item.getUniqueId().toString())
                                        .toList())
                                .setTarget(Transfer.Target.newBuilder()
                                        .setType(Transfer.Target.Type.FALLBACK)
                                        .build())
                                .build());
                        CloudPlugin.SCHEDULER.schedule(
                                () -> {
                                    // Check if players are on the server
                                    if (Bukkit.getOnlinePlayers().isEmpty()) {
                                        Bukkit.shutdown();
                                    }
                                },
                                4,
                                TimeUnit.SECONDS);
                    });
                    return Command.SINGLE_SUCCESS;
                })
                .build());
    }
}
