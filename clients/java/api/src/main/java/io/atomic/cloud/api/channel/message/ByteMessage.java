package io.atomic.cloud.api.channel.message;

public record ByteMessage(long timestamp, byte[] data) {}
