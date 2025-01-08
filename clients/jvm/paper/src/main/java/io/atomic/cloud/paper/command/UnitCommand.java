package io.atomic.cloud.paper.command;

import com.mojang.brigadier.Command;
import io.atomic.cloud.paper.command.argument.UnitArgument;
import io.atomic.cloud.paper.permission.Permissions;
import io.papermc.paper.command.brigadier.Commands;
import io.papermc.paper.command.brigadier.argument.ArgumentTypes;
import org.jetbrains.annotations.NotNull;

public class UnitCommand {

    public static void register(@NotNull Commands commands) {
        commands.register(Commands.literal("server")
                .requires(Permissions.SERVER_COMMAND::check)
                        .then(Commands.argument("user", ArgumentTypes.players()).then(Commands.argument("unit", UnitArgument.INSTANCE).executes(context -> {

                            // TODO: Command itself

                            return Command.SINGLE_SUCCESS;
                        })))
                .build());
    }
}
