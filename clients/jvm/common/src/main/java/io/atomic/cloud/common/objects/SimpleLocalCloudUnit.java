package io.atomic.cloud.common.objects;

import io.atomic.cloud.api.objects.LocalCloudUnit;
import io.atomic.cloud.common.connection.CloudConnection;
import java.util.concurrent.CompletableFuture;
import lombok.RequiredArgsConstructor;

@RequiredArgsConstructor
public class SimpleLocalCloudUnit implements LocalCloudUnit {

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
