package io.atomic.cloud.paper.command.argument;

import com.mojang.brigadier.StringReader;
import com.mojang.brigadier.arguments.ArgumentType;
import com.mojang.brigadier.context.CommandContext;
import com.mojang.brigadier.exceptions.CommandSyntaxException;
import com.mojang.brigadier.exceptions.DynamicCommandExceptionType;
import com.mojang.brigadier.suggestion.Suggestions;
import com.mojang.brigadier.suggestion.SuggestionsBuilder;
import io.atomic.cloud.grpc.unit.UnitInformation;
import io.atomic.cloud.paper.CloudPlugin;
import net.minecraft.commands.SharedSuggestionProvider;
import net.minecraft.network.chat.Component;
import org.jetbrains.annotations.NotNull;

import java.util.Arrays;
import java.util.Collection;
import java.util.concurrent.CompletableFuture;

public class UnitArgument implements ArgumentType<UnitInformation.SimpleUnitValue> {

    public static final UnitArgument INSTANCE = new UnitArgument();

    private static final Collection<String> EXAMPLES = Arrays.asList("lobby-1", "bedwars-2x1-1");
    public static final DynamicCommandExceptionType ERROR_STILL_FETCHING = new DynamicCommandExceptionType((_ignored) -> Component.literal("Fetching units..."));
    public static final DynamicCommandExceptionType ERROR_NOT_FOUND = new DynamicCommandExceptionType((unit) -> Component.literal("\"" + unit + "\" does not exist"));

    @Override
    public UnitInformation.SimpleUnitValue parse(@NotNull StringReader reader) throws CommandSyntaxException {
        var unquotedString = reader.readUnquotedString();
        var cached = CloudPlugin.INSTANCE.connection().getUnitsNow();
        if(cached.isEmpty()) throw ERROR_STILL_FETCHING.createWithContext(reader, unquotedString);
        var unit = cached.get().getUnitsList().stream().filter(item -> item.getName().equals(unquotedString)).findFirst();
        if(unit.isEmpty()) throw ERROR_NOT_FOUND.createWithContext(reader, unquotedString);
        return unit.get();
    }

    @Override
    public <S> CompletableFuture<Suggestions> listSuggestions(CommandContext<S> context, SuggestionsBuilder builder) {
        return CloudPlugin.INSTANCE.connection().getUnits().thenCompose(response -> SharedSuggestionProvider.suggest(response.getUnitsList().stream().map(UnitInformation.SimpleUnitValue::getName), builder));
    }

    @Override
    public Collection<String> getExamples() {
        return EXAMPLES;
    }
}
