package io.atomic.cloud.paper.command;

import com.mojang.brigadier.Command;
import io.atomic.cloud.grpc.server.TransferManagement;
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

                    sender.sendRichMessage("Marking server as <red>not ready");
                    connection.markNotReady().thenRun(() -> {
                        sender.sendRichMessage("Requesting to transfer all users to new <blue>servers<dark_gray>...");
                        connection.transferUsers(TransferManagement.TransferUsersRequest.newBuilder()
                                .addAllUserUuids(Bukkit.getOnlinePlayers().stream()
                                        .map(item -> item.getUniqueId().toString())
                                        .toList())
                                .setTarget(TransferManagement.TransferTargetValue.newBuilder()
                                        .setTargetType(TransferManagement.TransferTargetValue.TargetType.FALLBACK)
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
