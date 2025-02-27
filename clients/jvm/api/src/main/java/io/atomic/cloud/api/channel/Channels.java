package io.atomic.cloud.api.channel;

import io.atomic.cloud.api.channel.subscription.Bytes;
import io.atomic.cloud.api.channel.subscription.Strings;
import org.jetbrains.annotations.NotNull;

import java.util.concurrent.CompletableFuture;

public interface Channels {

    CompletableFuture<Integer> publishBytes(String channel, byte[] data);
    Bytes subscribeToBytes(String channel);

    default CompletableFuture<Integer> publishString(String channel, @NotNull String message) {
        return this.publishBytes(channel, message.getBytes());
    }
    default Strings subscribeToStrings(String channel) {
        return new Strings(this.subscribeToBytes(channel));
    }

}
