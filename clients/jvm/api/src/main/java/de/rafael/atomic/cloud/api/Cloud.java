package de.rafael.atomic.cloud.api;

import java.util.concurrent.CompletableFuture;

public class Cloud {

    private static CloudInterface INSTANCE;

    public static void setup(CloudInterface instance) {
        if (Cloud.INSTANCE != null) throw new IllegalStateException();
        Cloud.INSTANCE = instance;
    }

    public static CompletableFuture<Void> shutdown() {
        return Cloud.INSTANCE.shutdown();
    }

    public static CompletableFuture<Void> markReady() {
        return Cloud.INSTANCE.markReady();
    }

    public static CompletableFuture<Void> markNotReady() {
        return Cloud.INSTANCE.markNotReady();
    }

    public static void disableAutoReady() {
        Cloud.INSTANCE.disableAutoReady();
    }

    public interface CloudInterface {
        // Server control
        CompletableFuture<Void> shutdown();

        // Network
        CompletableFuture<Void> markReady();

        CompletableFuture<Void> markNotReady();

        // Settings
        void disableAutoReady();
    }
}
