package io.atomic.cloud.api.channel;

import io.atomic.cloud.api.channel.handler.ChannelHandler;

import java.util.concurrent.CompletableFuture;

public interface Channels {

    /**
     * Send a message to a channel
     * @param channel the channel to send the message to
     * @param message the message to send
     * @return a future to be completed once the message has been sent
     */
    CompletableFuture<Void> sendMessage(String channel, String message);

    /**
     * Subscribe to a channel
     * @param channel the channel to subscribe to
     * @return a future to be completed once the subscription has been completed
     */
    CompletableFuture<Void> subscribe(String channel);

    /**
     * Unsubscribe from a channel
     * @param channel the channel to unsubscribe from
     * @return a future to be completed once the unsubscription has been completed
     */
    CompletableFuture<Void> unsubscribe(String channel);

    /**
     * Register a handler for a channel
     * @param channel the channel to register the handler for
     * @param handler the handler to register
     */
    void registerHandler(String channel, ChannelHandler handler);

}