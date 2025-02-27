package io.atomic.cloud.common.objects;

import io.atomic.cloud.api.objects.LocalCloudServer;
import io.atomic.cloud.common.connection.CloudConnection;
import java.util.concurrent.CompletableFuture;
import lombok.RequiredArgsConstructor;

@RequiredArgsConstructor
public class SimpleLocalCloudServer implements LocalCloudServer {

    protected final CloudConnection connection;

    @Override
    public CompletableFuture<Void> shutdown() {
        return this.connection.requestStop().thenRun(() -> {});
    }

    @Override
    public CompletableFuture<Void> setReady(boolean ready) {
        return this.connection.setReady(ready).thenRun(() -> {});
    }
}
