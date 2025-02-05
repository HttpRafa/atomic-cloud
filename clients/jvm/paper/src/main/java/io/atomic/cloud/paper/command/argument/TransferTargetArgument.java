package io.atomic.cloud.paper.command.argument;

import com.mojang.brigadier.arguments.ArgumentType;
import com.mojang.brigadier.arguments.StringArgumentType;
import com.mojang.brigadier.context.CommandContext;
import com.mojang.brigadier.exceptions.CommandSyntaxException;
import com.mojang.brigadier.exceptions.SimpleCommandExceptionType;
import com.mojang.brigadier.suggestion.Suggestions;
import com.mojang.brigadier.suggestion.SuggestionsBuilder;
import io.atomic.cloud.grpc.server.DeploymentInformation;
import io.atomic.cloud.grpc.server.TransferManagement;
import io.atomic.cloud.grpc.server.UnitInformation;
import io.atomic.cloud.paper.CloudPlugin;
import io.papermc.paper.command.brigadier.MessageComponentSerializer;
import io.papermc.paper.command.brigadier.argument.CustomArgumentType;
import java.util.concurrent.CompletableFuture;
import net.kyori.adventure.text.Component;
import net.kyori.adventure.text.format.NamedTextColor;
import org.jetbrains.annotations.Contract;
import org.jetbrains.annotations.NotNull;

@SuppressWarnings("UnstableApiUsage")
public class TransferTargetArgument
        implements CustomArgumentType.Converted<TransferManagement.TransferTargetValue, String> {

    public static final TransferTargetArgument INSTANCE = new TransferTargetArgument();

    @Override
    public TransferManagement.@NotNull TransferTargetValue convert(@NotNull String value)
            throws CommandSyntaxException {
        if (value.equalsIgnoreCase("fallback")) {
            return TransferManagement.TransferTargetValue.newBuilder()
                    .setTargetType(TransferManagement.TransferTargetValue.TargetType.FALLBACK)
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
            var cached = CloudPlugin.INSTANCE.connection().getUnitsNow();
            if (cached.isEmpty()) throw createException("Fetching available servers...");
            var server = cached.get().getUnitsList().stream()
                    .filter(item -> item.getName().equalsIgnoreCase(identifier))
                    .findFirst();
            if (server.isEmpty()) throw createException("\"" + identifier + "\" does not exist");
            return TransferManagement.TransferTargetValue.newBuilder()
                    .setTargetType(TransferManagement.TransferTargetValue.TargetType.UNIT)
                    .setTarget(server.get().getUuid())
                    .build();
        } else if (type.equalsIgnoreCase("group")) {
            var cached = CloudPlugin.INSTANCE.connection().getDeploymentsNow();
            if (cached.isEmpty()) throw createException("Fetching available groups...");
            var group = cached.get().getDeploymentsList().stream()
                    .filter(item -> item.equalsIgnoreCase(identifier))
                    .findFirst();
            if (group.isEmpty()) throw createException("\"" + identifier + "\" does not exist");
            return TransferManagement.TransferTargetValue.newBuilder()
                    .setTargetType(TransferManagement.TransferTargetValue.TargetType.DEPLOYMENT)
                    .setTarget(group.get())
                    .build();
        }
        throw createException("Unknown transfer target type: " + type);
    }

    @Override
    public <S> @NotNull CompletableFuture<Suggestions> listSuggestions(
            @NotNull CommandContext<S> context, @NotNull SuggestionsBuilder builder) {
        return CloudPlugin.INSTANCE
                .connection()
                .getUnits()
                .thenCombine(CloudPlugin.INSTANCE.connection().getDeployments(), SuggestionsData::new)
                .thenCompose(response -> {
                    response.servers
                            .getUnitsList()
                            .forEach(server -> builder.suggest(
                                    "server:" + server.getName(),
                                    MessageComponentSerializer.message()
                                            .serialize(Component.text(server.getUuid())
                                                    .color(NamedTextColor.BLUE))));
                    response.groups
                            .getDeploymentsList()
                            .forEach(group -> builder.suggest(
                                    "group:" + group,
                                    MessageComponentSerializer.message()
                                            .serialize(
                                                    Component.text(group).color(NamedTextColor.BLUE))));
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

    private record SuggestionsData(
            UnitInformation.UnitListResponse servers, DeploymentInformation.DeploymentListResponse groups) {}
}
