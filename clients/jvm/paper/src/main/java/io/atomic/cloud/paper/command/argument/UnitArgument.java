package io.atomic.cloud.paper.command.argument;

import com.mojang.brigadier.arguments.ArgumentType;
import com.mojang.brigadier.arguments.StringArgumentType;
import com.mojang.brigadier.context.CommandContext;
import com.mojang.brigadier.exceptions.CommandSyntaxException;
import com.mojang.brigadier.exceptions.SimpleCommandExceptionType;
import com.mojang.brigadier.suggestion.Suggestions;
import com.mojang.brigadier.suggestion.SuggestionsBuilder;
import io.atomic.cloud.grpc.unit.UnitInformation;
import io.atomic.cloud.paper.CloudPlugin;
import io.papermc.paper.command.brigadier.MessageComponentSerializer;
import io.papermc.paper.command.brigadier.argument.CustomArgumentType;
import java.util.Arrays;
import java.util.Collection;
import java.util.concurrent.CompletableFuture;
import net.kyori.adventure.text.Component;
import net.kyori.adventure.text.format.NamedTextColor;
import org.jetbrains.annotations.Contract;
import org.jetbrains.annotations.NotNull;

@SuppressWarnings("UnstableApiUsage")
public class UnitArgument implements CustomArgumentType.Converted<UnitInformation.SimpleUnitValue, String> {

    public static final UnitArgument INSTANCE = new UnitArgument();
    private static final Collection<String> EXAMPLES = Arrays.asList("lobby-1", "bedwars-2x1-1");

    @Override
    public UnitInformation.@NotNull SimpleUnitValue convert(@NotNull String value) throws CommandSyntaxException {
        var cached = CloudPlugin.INSTANCE.connection().getUnitsNow();
        if (cached.isEmpty()) throw createException("Fetching available units...");
        var unit = cached.get().getUnitsList().stream()
                .filter(item -> item.getName().equals(value))
                .findFirst();
        if (unit.isEmpty()) throw createException("\"" + value + "\" does not exist");
        return unit.get();
    }

    @Override
    public <S> @NotNull CompletableFuture<Suggestions> listSuggestions(
            @NotNull CommandContext<S> context, @NotNull SuggestionsBuilder builder) {
        return CloudPlugin.INSTANCE.connection().getUnits().thenCompose(response -> {
            response.getUnitsList()
                    .forEach(unit -> builder.suggest(
                            unit.getName(),
                            MessageComponentSerializer.message()
                                    .serialize(Component.text(unit.getUuid()).color(NamedTextColor.BLUE))));
            return builder.buildFuture();
        });
    }

    @Override
    public @NotNull ArgumentType<String> getNativeType() {
        return StringArgumentType.word();
    }

    @Override
    public @NotNull Collection<String> getExamples() {
        return EXAMPLES;
    }

    @Contract("_ -> new")
    private @NotNull CommandSyntaxException createException(@NotNull String message) {
        return new CommandSyntaxException(new SimpleCommandExceptionType(() -> message), () -> message);
    }
}
