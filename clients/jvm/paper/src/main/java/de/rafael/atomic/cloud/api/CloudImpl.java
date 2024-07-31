package de.rafael.atomic.cloud.api;

import de.rafael.atomic.cloud.CloudPlugin;
import java.util.concurrent.CompletableFuture;

public class CloudImpl implements Cloud.CloudAPI {

    @Override
    public CompletableFuture<Void> shutdown() {
        return CloudPlugin.INSTANCE.shutdown();
    }

    @Override
    public void disableAutoReady() {
        CloudPlugin.INSTANCE.settings().autoReady(false);
    }

    @Override
    public CompletableFuture<Void> markReady() {
        return CloudPlugin.INSTANCE.connection().markReady().thenApply(empty -> null);
    }

    @Override
    public CompletableFuture<Void> markNotReady() {
        return CloudPlugin.INSTANCE.connection().markNotReady().thenApply(empty -> null);
    }

}
