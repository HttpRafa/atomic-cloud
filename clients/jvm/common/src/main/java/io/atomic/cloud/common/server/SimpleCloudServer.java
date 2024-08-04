package io.atomic.cloud.common.server;

import io.atomic.cloud.api.server.CloudServer;
import io.atomic.cloud.common.connection.CloudConnection;
import lombok.RequiredArgsConstructor;

import java.util.concurrent.CompletableFuture;

@RequiredArgsConstructor
public class SimpleCloudServer implements CloudServer {

    protected final CloudConnection connection;

    @Override
    public CompletableFuture<Void> shutdown() {
        return this.connection.requestStop().thenRun(() -> {});
    }

    @Override
    public CompletableFuture<Void> markReady() {
        return this.connection.markReady().thenRun(() -> {});
    }

    @Override
    public CompletableFuture<Void> markNotReady() {
        return this.connection.markNotReady().thenRun(() -> {});
    }

}
