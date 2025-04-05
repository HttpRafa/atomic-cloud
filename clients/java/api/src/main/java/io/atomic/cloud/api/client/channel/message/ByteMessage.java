package io.atomic.cloud.api.client.channel.message;

public record ByteMessage(long timestamp, byte[] data) {}
