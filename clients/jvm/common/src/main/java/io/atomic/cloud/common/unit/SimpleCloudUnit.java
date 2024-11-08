package io.atomic.cloud.common.unit;

import io.atomic.cloud.api.unit.CloudUnit;
import io.atomic.cloud.common.connection.CloudConnection;
import java.util.concurrent.CompletableFuture;
import lombok.RequiredArgsConstructor;

@RequiredArgsConstructor
public class SimpleCloudUnit implements CloudUnit {

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
