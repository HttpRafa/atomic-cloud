package io.atomic.cloud.api.client.channel.message;

import org.jetbrains.annotations.NotNull;

public record StringMessage(long timestamp, String data) {

    public StringMessage(@NotNull ByteMessage message) {
        this(message.timestamp(), new String(message.data()));
    }
}
