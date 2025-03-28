package io.atomic.cloud.api.channel.subscription;

import io.atomic.cloud.api.channel.message.StringMessage;
import java.io.Closeable;
import java.io.IOException;
import java.util.function.Consumer;
import lombok.AllArgsConstructor;

/**
 * The Strings class represents a subscription channel that handles string messages. It wraps a
 * Bytes instance to process byte messages and convert them to string messages. This class
 * implements the Closeable interface, allowing the channel to be closed when no longer needed.
 */
@AllArgsConstructor
public class Strings implements Closeable {

    private final Bytes bytes;

    /**
     * Registers a handler to process incoming string messages.
     *
     * @param handler a Consumer that processes StringMessage instances
     */
    public void handler(Consumer<StringMessage> handler) {
        this.bytes.handler(byteMessage -> handler.accept(new StringMessage(byteMessage)));
    }

    /**
     * Registers an error handler to process any errors that occur.
     *
     * @param handler a Consumer that processes Throwable instances
     */
    public void errorHandler(Consumer<Throwable> handler) {
        this.bytes.errorHandler(handler);
    }

    /**
     * Closes the underlying Bytes instance, releasing any resources it holds.
     *
     * @throws IOException if an I/O error occurs
     */
    @Override
    public void close() throws IOException {
        bytes.close();
    }
}
