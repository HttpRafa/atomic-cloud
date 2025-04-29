package io.atomic.cloud.api.user;

import java.util.concurrent.CompletableFuture;

public interface Users {

    /**
     * Sends a request to the controller to get the number of users currently online
     *
     * @return The number of users on the network
     */
    CompletableFuture<Integer> userCount();
}
