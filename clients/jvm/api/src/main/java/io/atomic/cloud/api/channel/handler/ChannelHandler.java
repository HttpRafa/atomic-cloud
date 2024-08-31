package io.atomic.cloud.api.channel.handler;

public interface ChannelHandler {

    void onMessage(String channel, String message);

    void onError(Throwable throwable);

    void onEnd();
}
