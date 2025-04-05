package io.atomic.cloud.common.resource.object;

import io.atomic.cloud.api.client.self.LocalCloudServer;
import io.atomic.cloud.common.connection.client.ClientConnection;
import java.util.concurrent.CompletableFuture;
import lombok.RequiredArgsConstructor;

@RequiredArgsConstructor
public class LocalCloudServerImpl implements LocalCloudServer {

    protected final ClientConnection connection;

    @Override
    public CompletableFuture<Void> shutdown() {
        return this.connection.requestStop().thenRun(() -> {});
    }

    @Override
    public CompletableFuture<Void> ready(boolean ready) {
        return this.connection.ready(ready).thenRun(() -> {});
    }
}
