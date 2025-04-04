package io.atomic.cloud.common.resource.object;

import io.atomic.cloud.api.resource.object.LocalCloudServer;
import io.atomic.cloud.common.connection.client.ClientConnection;
import java.util.concurrent.CompletableFuture;
import lombok.RequiredArgsConstructor;

@RequiredArgsConstructor
public class SimpleLocalCloudServer implements LocalCloudServer {

    protected final ClientConnection connection;

    @Override
    public CompletableFuture<Void> shutdown() {
        return this.connection.requestStop().thenRun(() -> {});
    }

    @Override
    public CompletableFuture<Void> setReady(boolean ready) {
        return this.connection.ready(ready).thenRun(() -> {});
    }
}
