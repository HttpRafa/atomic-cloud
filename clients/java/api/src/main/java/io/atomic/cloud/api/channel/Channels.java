package io.atomic.cloud.api.channel;

import io.atomic.cloud.api.channel.subscription.Bytes;
import io.atomic.cloud.api.channel.subscription.Strings;
import java.util.concurrent.CompletableFuture;
import org.jetbrains.annotations.NotNull;

/**
 * The Channels interface provides methods for publishing and subscribing to byte and string
 * messages.
 */
public interface Channels {

    /**
     * Publishes byte data to a specified channel.
     *
     * @param channel the name of the channel to publish to
     * @param data the byte array to be published
     * @return a CompletableFuture that completes with the number of subscribers that received the
     *     message
     */
    CompletableFuture<Integer> publishBytes(String channel, byte[] data);

    /**
     * Subscribes to a specified channel to receive byte messages.
     *
     * @param channel the name of the channel to subscribe to
     * @return a Bytes instance for handling byte messages
     */
    Bytes subscribeToBytes(String channel);

    /**
     * Publishes a string message to a specified channel. This method converts the string message to
     * a byte array and delegates to publishBytes.
     *
     * @param channel the name of the channel to publish to
     * @param message the string message to be published
     * @return a CompletableFuture that completes with the number of subscribers that received the
     *     message
     */
    default CompletableFuture<Integer> publishString(String channel, @NotNull String message) {
        return this.publishBytes(channel, message.getBytes());
    }

    /**
     * Subscribes to a specified channel to receive string messages. This method wraps the Bytes
     * subscription to handle string messages.
     *
     * @param channel the name of the channel to subscribe to
     * @return a Strings instance for handling string messages
     */
    default Strings subscribeToStrings(String channel) {
        return new Strings(this.subscribeToBytes(channel));
    }
}
