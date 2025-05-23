package io.atomic.cloud.paper.send.command.argument;

import com.mojang.brigadier.arguments.ArgumentType;
import com.mojang.brigadier.arguments.StringArgumentType;
import com.mojang.brigadier.context.CommandContext;
import com.mojang.brigadier.exceptions.CommandSyntaxException;
import com.mojang.brigadier.exceptions.SimpleCommandExceptionType;
import com.mojang.brigadier.suggestion.Suggestions;
import com.mojang.brigadier.suggestion.SuggestionsBuilder;
import io.atomic.cloud.grpc.client.Transfer;
import io.atomic.cloud.grpc.common.CommonGroup;
import io.atomic.cloud.grpc.common.CommonServer;
import io.atomic.cloud.paper.CloudPlugin;
import io.papermc.paper.command.brigadier.MessageComponentSerializer;
import io.papermc.paper.command.brigadier.argument.CustomArgumentType;
import java.util.concurrent.CompletableFuture;
import net.kyori.adventure.text.Component;
import net.kyori.adventure.text.format.NamedTextColor;
import org.jetbrains.annotations.Contract;
import org.jetbrains.annotations.NotNull;

@SuppressWarnings("UnstableApiUsage")
public class TransferTargetArgument implements CustomArgumentType.Converted<Transfer.Target, String> {

    public static final TransferTargetArgument INSTANCE = new TransferTargetArgument();

    @Override
    public Transfer.@NotNull Target convert(@NotNull String value) throws CommandSyntaxException {
        if (value.equalsIgnoreCase("fallback")) {
            return Transfer.Target.newBuilder()
                    .setType(Transfer.Target.Type.FALLBACK)
                    .build();
        }
        var valueSplit = value.split(":");
        if (valueSplit.length != 2) {
            throw createException(
                    "Invalid transfer target value expected <type>:<value> but found something different: " + value);
        }
        var type = valueSplit[0];
        var identifier = valueSplit[1];
        if (type.equalsIgnoreCase("server")) {
            var cached = CloudPlugin.INSTANCE.clientConnection().serversNow();
            if (cached.isEmpty()) throw createException("Fetching available servers...");
            var server = cached.get().getServersList().stream()
                    .filter(item -> item.getName().equalsIgnoreCase(identifier))
                    .findFirst();
            if (server.isEmpty()) throw createException("\"" + identifier + "\" does not exist");
            return Transfer.Target.newBuilder()
                    .setType(Transfer.Target.Type.SERVER)
                    .setTarget(server.get().getId())
                    .build();
        } else if (type.equalsIgnoreCase("group")) {
            var cached = CloudPlugin.INSTANCE.clientConnection().groupsNow();
            if (cached.isEmpty()) throw createException("Fetching available groups...");
            var group = cached.get().getGroupsList().stream()
                    .filter(item -> item.getName().equalsIgnoreCase(identifier))
                    .findFirst();
            if (group.isEmpty()) throw createException("\"" + identifier + "\" does not exist");
            return Transfer.Target.newBuilder()
                    .setType(Transfer.Target.Type.GROUP)
                    .setTarget(group.get().getName())
                    .build();
        }
        throw createException("Unknown transfer target type: " + type);
    }

    @Override
    public <S> @NotNull CompletableFuture<Suggestions> listSuggestions(
            @NotNull CommandContext<S> context, @NotNull SuggestionsBuilder builder) {
        return CloudPlugin.INSTANCE
                .clientConnection()
                .servers()
                .thenCombine(CloudPlugin.INSTANCE.clientConnection().groups(), SuggestionsData::new)
                .thenCompose(response -> {
                    response.servers
                            .getServersList()
                            .forEach(server -> builder.suggest(
                                    "server:" + server.getName(),
                                    MessageComponentSerializer.message()
                                            .serialize(Component.text(server.getId())
                                                    .color(NamedTextColor.BLUE))));
                    response.groups
                            .getGroupsList()
                            .forEach(group -> builder.suggest(
                                    "group:" + group.getName(),
                                    MessageComponentSerializer.message()
                                            .serialize(Component.text(group.getName())
                                                    .color(NamedTextColor.BLUE))));
                    builder.suggest(
                            "fallback",
                            MessageComponentSerializer.message()
                                    .serialize(Component.text(
                                                    "This option will try to transfer all users to a fallback server")
                                            .color(NamedTextColor.BLUE)));
                    return builder.buildFuture();
                });
    }

    @Override
    public @NotNull ArgumentType<String> getNativeType() {
        return StringArgumentType.greedyString();
    }

    @Contract("_ -> new")
    private @NotNull CommandSyntaxException createException(@NotNull String message) {
        return new CommandSyntaxException(new SimpleCommandExceptionType(() -> message), () -> message);
    }

    private record SuggestionsData(CommonServer.List servers, CommonGroup.List groups) {}
}
