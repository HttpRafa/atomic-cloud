package io.atomic.cloud.paper.command;

import com.mojang.brigadier.Command;
import io.atomic.cloud.grpc.client.Transfer;
import io.atomic.cloud.paper.CloudPlugin;
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

                    CloudPlugin.INSTANCE.messages().notReady().send(sender);
                    connection.setReady(false).thenRun(() -> {
                        CloudPlugin.INSTANCE.messages().transferAllUsers().send(sender);
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
                                    // Check if players are on the
                                    // server
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
