package io.atomic.cloud.api;

import java.util.concurrent.CompletableFuture;

public class Cloud {

    private static CloudAPI INSTANCE;

    public static void setup(CloudAPI instance) {
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

    public interface CloudAPI {
        /**
         * Shut down this server instance.
         * This will stop the server and transfer all players to a different server.
         * How the server is shutdown depends on the disk retention policy.
         * If the server is marked as permanent, it will not be deleted.
         * If the server is not marked as permanent, it will be killed and deleted.
         * @return a future to be completed once the server has been shut down
         */
        CompletableFuture<Void> shutdown();

        /**
         * Mark this server as ready
         * @return a future to be completed once the server has been marked as ready
         */
        CompletableFuture<Void> markReady();

        /**
         * Mark this server as not ready
         * @return a future to be completed once the server has been marked as not ready
         */
        CompletableFuture<Void> markNotReady();

        /**
         * The server marks itself ready when it is started. This method disables this behavior.
         * This is useful if you want to control when the server is ready yourself.
         */
        void disableAutoReady();
    }
}
