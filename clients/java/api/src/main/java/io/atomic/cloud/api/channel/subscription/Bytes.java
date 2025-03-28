package io.atomic.cloud.api.channel.subscription;

import io.atomic.cloud.api.channel.message.ByteMessage;
import java.io.Closeable;
import java.util.function.Consumer;

/**
 * The Bytes interface represents a subscription channel that handles byte messages. It extends the
 * Closeable interface, allowing the channel to be closed when no longer needed.
 */
public interface Bytes extends Closeable {

    /**
     * Registers a handler to process incoming byte messages.
     *
     * @param handler a Consumer that processes ByteMessage instances
     */
    void handler(Consumer<ByteMessage> handler);

    /**
     * Registers an error handler to process any errors that occur.
     *
     * @param handler a Consumer that processes Throwable instances
     */
    void errorHandler(Consumer<Throwable> handler);
}
