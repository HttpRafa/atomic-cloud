package io.atomic.cloud.api.client.self;

import java.util.concurrent.CompletableFuture;

public interface LocalCloudServer {

    /**
     * Shut down this server instance. This will stop the server and transfer all players to a
     * different server. How the server is shutdown depends on the disk retention policy. If the
     * server is marked as permanent, it will not be deleted. If the server is not marked as
     * permanent, it will be killed and deleted.
     *
     * @return a future to be completed once the server has been shut down
     */
    CompletableFuture<Void> shutdown();

    /**
     * Mark this server as ready/not ready
     *
     * @return a future to be completed once the server has been marked as ready/not ready
     */
    CompletableFuture<Void> ready(boolean ready);
}
