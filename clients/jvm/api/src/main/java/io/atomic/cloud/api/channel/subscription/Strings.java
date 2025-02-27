package io.atomic.cloud.api.channel.subscription;

import lombok.AllArgsConstructor;

import java.io.Closeable;
import java.io.IOException;

@AllArgsConstructor
public class Strings implements Closeable {

    private final Bytes bytes;

    @Override
    public void close() throws IOException {
        bytes.close();
    }
}